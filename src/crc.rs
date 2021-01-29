use core::fmt;
use std::error::Error;

// CRC table generation and Mode S checksumming ported from
// https://github.com/wiedehopf/readsb/blob/177545fbff8cc2be9d7e9b9b109c5c1046c2642b/crc.c

const MODES_GENERATOR_POLY: u32 = 0xfff409;

lazy_static! {
    static ref CRC_TABLE: [u32; 256] = {
        let mut result = [0u32; 256];
        for i in 0..256 {
            let mut c = i << 16;
            for _j in 0..8 {
                if c & 0x800000 != 0 {
                    c = (c << 1) ^ MODES_GENERATOR_POLY;
                } else {
                    c <<= 1;
                }
            }
            result[i as usize] = c & 0x00ffffff;
        }
        result
    };
}

#[derive(Debug, Clone)]
pub struct CrcError(String);

impl fmt::Display for CrcError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for CrcError {}

pub fn mode_s_crc(input: &[u8], num_bytes: u8) -> Result<u32, CrcError> {
    let mut rem = 0u32;
    if num_bytes < 3 {
        return Err(CrcError(String::from(
            "num_bytes must be >= 3 to calculate Mode S CRC",
        )));
    }
    let num_bytes = num_bytes as usize;
    for byte in input.iter().take(num_bytes - 3) {
        let idx = (*byte as u32) ^ ((rem & 0xff0000) >> 16);
        rem = (rem << 8) ^ CRC_TABLE[idx as usize];
        rem &= 0xffffff;
    }
    rem = rem
        ^ ((input[num_bytes - 3] as u32) << 16)
        ^ ((input[num_bytes - 2] as u32) << 8)
        ^ (input[num_bytes - 1] as u32);
    Ok(rem)
}
