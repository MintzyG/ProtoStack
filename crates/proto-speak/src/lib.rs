//! Shared `#![no_std]` binary protocol for ProtoStack.
//!
//! `proto-speak` defines:
//! - The 12-byte packet header + postcard payload + CRC-16/CCITT wire format.
//! - 64-byte WebHID report chunking and reassembly.
//! - Typed control messages (`HELLO_INFO`, `PREVIEW_STATE`, etc.).
//!
//! Because it uses no heap allocation and no OS APIs, the exact same code
//! compiles for the Teensy 4.1 firmware, unit tests, and browser WASM.

#![no_std]

pub mod hid;
pub mod messages;
pub mod packet;

pub use hid::{
    HidChunker, HidReassembler, HID_CHUNK_DATA_LEN, HID_HEADER_LEN, HID_REPORT_ID, HID_REPORT_SIZE,
};
pub use messages::{HelloInfo, HelloRequest, McuKind, PreviewAck, PreviewState};
pub use packet::{
    decode_packet, decode_payload, encode_packet, encode_payload, DecodedPacket, Domain, Flags,
    Header, NodeId, Opcode, CRC_LEN, HEADER_LEN, MAX_PACKET_SIZE, PROTOCOL_VERSION,
};

/// Errors returned while encoding, decoding, or reassembling ProtoSpeak frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// Output buffer does not have enough space for the encoded bytes.
    BufferTooSmall,
    /// Input packet is shorter than the minimum header + CRC size, or length mismatches.
    TruncatedPacket,
    /// Header version nibble is not supported by this build.
    UnsupportedVersion(u8),
    /// Header domain nibble is not a known domain.
    InvalidDomain(u8),
    /// Computed CRC-16 does not match the trailing CRC in the packet.
    BadCrc {
        /// CRC computed locally over header + payload.
        expected: u16,
        /// CRC bytes received at the end of the packet.
        actual: u16,
    },
    /// Postcard serialization or deserialization failed.
    Codec,
    /// Received a HID report with a report ID other than `HID_REPORT_ID`.
    InvalidReportId(u8),
    /// Received a HID chunk with `total_chunks == 0` or `chunk_index >= total_chunks`.
    InvalidChunkHeader,
    /// Received a HID chunk whose index did not match the expected sequence.
    OutOfOrderChunk {
        /// Next chunk index the reassembler expected.
        expected: u8,
        /// Chunk index present in the incoming HID report.
        actual: u8,
    },
    /// Incoming HID message exceeds the reassembler's fixed buffer capacity.
    PacketTooLarge,
}
