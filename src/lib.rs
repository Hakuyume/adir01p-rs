pub mod presets;

use rusb::{DeviceHandle, GlobalContext, UsbContext};
use std::cmp;
use std::str::Utf8Error;
use std::time::Duration;

const VENDOR_ID: u16 = 0x22ea;
const PRODUCT_ID: u16 = 0x003a;
const IFACE: u8 = 3;
const ENDPOINT_IN: u8 = 0x84;
const ENDPOINT_OUT: u8 = 0x04;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Rusb(#[from] rusb::Error),
    #[error(transparent)]
    Utf8(#[from] Utf8Error),
    #[error("no device found")]
    NoDevice,
    #[error("unexpected code (0x{actual:02x} != 0x{expected:02x})")]
    UnexpectedCode { actual: u8, expected: u8 },
}

pub struct Device<T>
where
    T: UsbContext,
{
    handle: DeviceHandle<T>,
    timeout: Duration,
}

pub fn open(timeout: Duration) -> Result<Device<GlobalContext>, Error> {
    Device::open(&GlobalContext::default(), timeout)
}

impl<T> Device<T>
where
    T: UsbContext,
{
    pub fn recv(&mut self, freq: u16) -> Result<Recv<'_, T>, Error> {
        self.recv_start(freq)?;
        Ok(Recv { device: self })
    }

    pub fn send(&mut self, freq: u16, bits: &[Bit]) -> Result<(), Error> {
        self.write(bits)?;
        self.transmit(freq, bits.len())?;
        Ok(())
    }

    fn open(context: &T, timeout: Duration) -> Result<Self, Error> {
        let device = context
            .devices()?
            .iter()
            .find_map(|device| {
                device
                    .device_descriptor()
                    .map(|device_descriptor| {
                        (device_descriptor.vendor_id() == VENDOR_ID
                            || device_descriptor.product_id() == PRODUCT_ID)
                            .then_some(device)
                    })
                    .transpose()
            })
            .ok_or(Error::NoDevice)??;
        let mut handle = device.open()?;
        if handle.kernel_driver_active(IFACE)? {
            handle.detach_kernel_driver(IFACE)?;
        }
        handle.claim_interface(IFACE)?;
        Ok(Self { handle, timeout })
    }

    fn communicate(&mut self, request: &[u8]) -> Result<[u8; 64], Error> {
        self.handle
            .write_interrupt(ENDPOINT_OUT, request, self.timeout)?;
        let mut response = [0; 64];
        self.handle
            .read_interrupt(ENDPOINT_IN, &mut response, self.timeout)?;
        if response[0] == request[0] {
            Ok(response)
        } else {
            Err(Error::UnexpectedCode {
                actual: response[0],
                expected: request[0],
            })
        }
    }

    fn recv_start(&mut self, freq: u16) -> Result<(), Error> {
        let mut request = [0; 8];
        request[0] = 0x31;
        request[1..3].copy_from_slice(&freq.to_be_bytes());
        self.communicate(&request)?;
        Ok(())
    }

    fn recv_stop(&mut self) -> Result<(), Error> {
        self.communicate(&[0x32])?;
        Ok(())
    }

    fn read(&mut self) -> Result<Vec<Bit>, Error> {
        let mut bits = Vec::new();
        loop {
            let response = self.communicate(&[0x33])?;
            let total = u16::from_be_bytes(response[1..3].try_into().unwrap()) as usize;
            let offset = u16::from_be_bytes(response[3..5].try_into().unwrap()) as usize;
            let len = response[5] as usize;
            let data = &response[6..];
            if total > 0 && len > 0 {
                bits.resize(total, Bit { on: 0, off: 0 });
                for (i, chunk) in data.chunks_exact(4).take(len).enumerate() {
                    bits[offset + i] = Bit {
                        on: u16::from_be_bytes(chunk[..2].try_into().unwrap()),
                        off: u16::from_be_bytes(chunk[2..].try_into().unwrap()),
                    };
                }
            } else {
                break;
            }
        }
        Ok(bits)
    }

    fn write(&mut self, bits: &[Bit]) -> Result<(), Error> {
        let total = bits.len();
        let mut bits = bits.iter();
        let mut offset = 0;
        loop {
            let mut request = [0; 64];
            let len = cmp::min(request[6..].len() / 4, bits.len());
            request[0] = 0x34;
            request[1..3].copy_from_slice(&(total as u16).to_be_bytes());
            request[3..5].copy_from_slice(&(offset as u16).to_be_bytes());
            request[5] = len as _;
            for (chunk, bit) in request[6..].chunks_exact_mut(4).zip(bits.by_ref()) {
                chunk[..2].copy_from_slice(&bit.on.to_be_bytes());
                chunk[2..].copy_from_slice(&bit.off.to_be_bytes());
            }
            self.communicate(&request)?;
            if request[5] > 0 {
                offset += len;
            } else {
                break;
            }
        }
        Ok(())
    }

    fn transmit(&mut self, freq: u16, len: usize) -> Result<(), Error> {
        let mut request = [0; 5];
        request[0] = 0x35;
        request[1..3].copy_from_slice(&freq.to_be_bytes());
        request[3..5].copy_from_slice(&(len as u16).to_be_bytes());
        self.communicate(&request)?;
        Ok(())
    }

    pub fn firmware_version(&mut self) -> Result<String, Error> {
        let response = self.communicate(&[0x56])?;
        let version = &response[1..];
        Ok(std::str::from_utf8(version.split(|&c| c == 0).next().unwrap())?.to_owned())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Bit {
    pub on: u16,
    pub off: u16,
}

pub struct Recv<'a, T>
where
    T: UsbContext,
{
    device: &'a mut Device<T>,
}

impl<T> Recv<'_, T>
where
    T: UsbContext,
{
    pub fn finish(self) -> Result<Vec<Bit>, Error> {
        self.device.recv_stop()?;
        self.device.read()
    }
}
