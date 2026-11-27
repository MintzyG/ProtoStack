//! CRC-16/CCITT-FALSE checksum used on ProtoSpeak frames.

use crc::{Crc, CRC_16_IBM_3740};

/// CRC-16/IBM-3740 (also known as CRC-16/CCITT-FALSE: poly `0x1021`, init `0xFFFF`,
/// no reflection, no final XOR, check value `0x29B1`).
const CRC16: Crc<u16> = Crc::<u16>::new(&CRC_16_IBM_3740);

/// Computes CRC-16/CCITT-FALSE over `bytes`.
pub fn crc16_ccitt(bytes: &[u8]) -> u16 {
    CRC16.checksum(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_check_vector() {
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
