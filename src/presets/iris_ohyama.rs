use crate::Bit;

pub fn freq() -> u16 {
    38_000
}

fn bits(bytes: [u8; 5], repeat: usize) -> Vec<Bit> {
    const T: u16 = 20;
    let mut bits = vec![Bit {
        on: 4 * T,
        off: 2 * T,
    }];
    for _ in 0..repeat {
        bits.push(Bit {
            on: 11 * T,
            off: 2 * T,
        });
        for byte in bytes {
            for k in (0..8).rev() {
                if byte & (1 << k) == 0 {
                    bits.push(Bit { on: T, off: T });
                } else {
                    bits.push(Bit { on: 3 * T, off: T });
                }
            }
        }
        bits.push(Bit { on: T, off: 18 * T });
    }
    bits
}

pub mod cl_rl1 {
    pub use super::freq;
    use crate::Bit;

    // 切/入/常夜灯
    pub fn off_on_night_light_ch1(repeat: usize) -> Vec<Bit> {
        super::bits(
            [0b11000000, 0b10000001, 0b01000000, 0b00000000, 0b01011011],
            repeat,
        )
    }
}

pub mod ledhcl_r1 {
    pub use super::freq;
    use crate::Bit;

    // 電源
    pub fn power_ch1(repeat: usize) -> Vec<Bit> {
        super::bits(
            [0b10000000, 0b10001000, 0b00000000, 0b00000000, 0b01010010],
            repeat,
        )
    }

    // 調光
    pub fn dimming_ch1(repeat: usize) -> Vec<Bit> {
        super::bits(
            [0b10000000, 0b10000100, 0b00000000, 0b00000000, 0b01011110],
            repeat,
        )
    }

    // 常夜灯
    pub fn night_light_ch1(repeat: usize) -> Vec<Bit> {
        super::bits(
            [0b10000000, 0b10000010, 0b00000000, 0b00000000, 0b01011000],
            repeat,
        )
    }
}
