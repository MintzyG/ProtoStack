//! CRC-16/CCITT-FALSE checksum used on ProtoSpeak frames.

/// Computes CRC-16/CCITT-FALSE (polynomial `0x1021`, initial value `0xFFFF`,
/// no final XOR, MSB-first).
pub fn crc16_ccitt(bytes: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &byte in bytes {
        crc ^= (byte as u16) << 8;
        for _ in 0..8 {
            if (crc & 0x8000) != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_check_vector() {
        // Standard CRC-16/CCITT-FALSE check value for ASCII "123456789" is 0x29B1.
        assert_eq!(crc16_ccitt(b"123456789"), 0x29B1);
    }

    #[test]
    fn empty_input_returns_initial_seed() {
        assert_eq!(crc16_ccitt(&[]), 0xFFFF);
    }

    #[test]
    fn single_bit_flip_changes_checksum() {
        let a = crc16_ccitt(&[0x10, 0x00, 0x01, 0x00]);
        let b = crc16_ccitt(&[0x10, 0x00, 0x01, 0x01]);
        assert_ne!(a, b);
    }
}
