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

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum CrcError {
    InputTooShort,
}

impl fmt::Display for CrcError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CrcError::InputTooShort => "length in bytes must be >= 3 to calculate CRC",
            }
        )
    }
}

impl Error for CrcError {}

pub(crate) fn get_crc_remainder(input: &[u8]) -> Result<u32, CrcError> {
    let mut rem = 0u32;
    let num_bytes = input.len();
    if num_bytes < 3 {
        return Err(CrcError::InputTooShort);
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc_valid() {
        let message_with_crc = b"\x8D\x48\x40\xD6\x20\x2C\xC3\x71\xC3\x2C\xE0\x57\x60\x98";
        assert_eq!(get_crc_remainder(message_with_crc).unwrap(), 0);
        let message_without_crc = &message_with_crc[0..message_with_crc.len() - 3];
        let crc = &message_with_crc[message_with_crc.len() - 3..message_with_crc.len()];
        assert_eq!(
            &get_crc_remainder([message_without_crc, b"\x00\x00\x00"].concat().as_slice())
                .unwrap()
                .to_be_bytes()[1..4],
            crc
        );
    }

    #[test]
    fn crc_invalid() {
        let message_invalid_crc = b"\x8E\x48\x40\xD6\x20\x2C\xC3\x71\xC3\x2C\xE0\x57\x60\x98";
        assert_eq!(get_crc_remainder(message_invalid_crc), Ok(15242120));
    }

    #[test]
    fn crc_input_too_short() {
        assert_eq!(get_crc_remainder(b"\x60\x98"), Err(CrcError::InputTooShort));
    }
}
