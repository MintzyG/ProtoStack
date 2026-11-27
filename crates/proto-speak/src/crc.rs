//! CRC-16/CCITT-FALSE checksum used on ProtoSpeak frames.

use crc::{Crc, CRC_16_IBM_3740};

/// CRC-16/IBM-3740 (also known as CRC-16/CCITT-FALSE: poly `0x1021`, init `0xFFFF`,
/// no reflection, no final XOR, check value `0x29B1`).
const CRC16: Crc<u16> = Crc::<u16>::new(&CRC_16_IBM_3740);

/// Computes CRC-16/CCITT-FALSE over `bytes`.
pub fn crc16_ccitt(bytes: &[u8]) -> u16 {
    CRC16.checksum(bytes)
}
