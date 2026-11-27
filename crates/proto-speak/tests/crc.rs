//! Tests for CRC-16/CCITT-FALSE checksums.

use proto_speak::crc16_ccitt;

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
