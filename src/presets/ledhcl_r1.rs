//! アイリスオーヤマ LEDHCL-R1

use crate::Bit;

pub fn freq() -> u16 {
    38_000
}

// 電源
pub fn power_ch1(repeat: usize) -> Vec<Bit> {
    bits(
        [0b10000000, 0b10001000, 0b00000000, 0b00000000, 0b01010010],
        repeat,
    )
}

// 調光
pub fn dimming_ch1(repeat: usize) -> Vec<Bit> {
    bits(
        [0b10000000, 0b10000100, 0b00000000, 0b00000000, 0b01011110],
        repeat,
    )
}

// 常夜灯
pub fn night_light_ch1(repeat: usize) -> Vec<Bit> {
    bits(
        [0b10000000, 0b10000010, 0b00000000, 0b00000000, 0b01011000],
        repeat,
    )
}

fn bits(bytes: [u8; 5], repeat: usize) -> Vec<Bit> {
    const T: u16 = 20;
    let mut bits = Vec::new();
    bits.push(Bit {
        on: 4 * T,
        off: 2 * T,
    });
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
