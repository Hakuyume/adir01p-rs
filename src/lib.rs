use rusb::{DeviceHandle, GlobalContext, UsbContext};
use std::string::FromUtf8Error;
use std::time::Duration;

const VENDOR_ID: u16 = 0x22ea;
const PRODUCT_ID: u16 = 0x003a;
const IFACE: u8 = 3;
const ENDPOINT_IN: u8 = 0x84;
const ENDPOINT_OUT: u8 = 0x04;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    FromUtf8(#[from] FromUtf8Error),
    #[error(transparent)]
    Rusb(#[from] rusb::Error),
    #[error("no device found")]
    NoDevice,
}

pub fn open() -> Result<Adir01p<GlobalContext>, Error> {
    Adir01p::with_context(&GlobalContext::default())
}

pub struct Adir01p<T>
where
    T: UsbContext,
{
    handle: DeviceHandle<T>,
}

impl<T> Adir01p<T>
where
    T: UsbContext,
{
    fn with_context(context: &T) -> Result<Self, Error> {
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
        Ok(Self { handle })
    }

    pub fn firmware_version(&self, timeout: Duration) -> Result<String, Error> {
        self.handle
            .write_interrupt(ENDPOINT_OUT, &[0x56], timeout)?;
        let mut buf = vec![0; 64];
        self.handle.read_interrupt(ENDPOINT_IN, &mut buf, timeout)?;
        if let Some(len) = buf.iter().position(|&c| c == 0) {
            buf.truncate(len);
        }
        Ok(String::from_utf8(buf)?)
    }
}
