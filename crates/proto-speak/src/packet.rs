//! 12-byte ProtoSpeak packet envelope, encoding, and CRC verification.
//!
//! Wire layout (little-endian):
//! ```text
//! Byte 0:       [Ver (high 4 bits) | Domain (low 4 bits)]
//! Byte 1:       Flags bitmask
//! Bytes 2..4:   Opcode (u16)
//! Bytes 4..6:   Source NodeId (u16)
//! Bytes 6..8:   Destination NodeId (u16)
//! Bytes 8..10:  Sequence number (u16)
//! Bytes 10..12: Payload length in bytes (u16)
//! Bytes 12..12+len: Postcard-encoded payload
//! Last 2 bytes: CRC-16/CCITT-FALSE over bytes 0..12+len
//! ```

use serde::{de::DeserializeOwned, Serialize};

use crate::{crc16_ccitt, Error};

/// Current ProtoSpeak wire format version (fits in 4 bits).
pub const PROTOCOL_VERSION: u8 = 1;

/// Fixed size of the packet header in bytes.
pub const HEADER_LEN: usize = 12;

/// Size of the trailing CRC-16 checksum in bytes.
pub const CRC_LEN: usize = 2;

/// Default maximum packet size for v0.1 control frames (fits in 5 HID chunks).
pub const MAX_PACKET_SIZE: usize = 256;

/// Communication domain stored in the lower 4 bits of header byte 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Domain {
    /// Host-to-device USB WebHID / BLE configuration control (v0.1+).
    Control = 0,
    /// Intra-suit wired or wireless part bus (v1.2+).
    SuitMesh = 1,
    /// Opt-in proximity beacons between different suits (v2.0+).
    SocialMesh = 2,
}

impl Domain {
    fn from_u8(value: u8) -> Result<Self, Error> {
        match value {
            0 => Ok(Self::Control),
            1 => Ok(Self::SuitMesh),
            2 => Ok(Self::SocialMesh),
            other => Err(Error::InvalidDomain(other)),
        }
    }
}

/// Bitmask flags in header byte 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Flags(pub u8);

impl Flags {
    /// No flags set.
    pub const NONE: Self = Self(0);
    /// Sender requests an acknowledgement packet.
    pub const ACK_REQ: Self = Self(1 << 0);
    /// Packet is an acknowledgement / response to an earlier request.
    pub const IS_ACK: Self = Self(1 << 1);
    /// Payload is encrypted (reserved for wireless Suit Mesh).
    pub const ENCRYPTED: Self = Self(1 << 2);
    /// Packet is fragmented at the application layer.
    pub const FRAGMENTED: Self = Self(1 << 3);
    /// High-priority safety frame (e.g. emergency fan/dim command).
    pub const PRIORITY_SAFETY: Self = Self(1 << 4);

    /// Combines two flag masks.
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Returns true if all bits in `flag` are set.
    pub const fn contains(self, flag: Self) -> bool {
        (self.0 & flag.0) == flag.0
    }
}

/// Well-known node addresses in the ProtoSpeak header.
pub struct NodeId;

impl NodeId {
    /// Browser / PC host configurator.
    pub const HOST: u16 = 0x0000;
    /// Main helmet controller (Teensy 4.1).
    pub const HEAD: u16 = 0x0001;
    /// All nodes on the bus.
    pub const BROADCAST: u16 = 0xFFFF;
}

/// Control domain opcodes (see `docs/03-protoio-protocol-spec.md`).
pub struct Opcode;

impl Opcode {
    /// Handshake: firmware version, MCU type, capabilities, display size.
    pub const HELLO_INFO: u16 = 0x0001;
    /// Live RAM preview: switch face, tint, brightness, virtual button state.
    pub const PREVIEW_STATE: u16 = 0x0010;
}

/// Parsed 12-byte ProtoSpeak packet header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header {
    /// Protocol version nibble (`PROTOCOL_VERSION`).
    pub version: u8,
    /// Communication domain (`Control`, `SuitMesh`, `SocialMesh`).
    pub domain: Domain,
    /// Packet flags bitmask.
    pub flags: Flags,
    /// Operation code (`Opcode::*`).
    pub opcode: u16,
    /// Source node ID (`NodeId::*`).
    pub src: u16,
    /// Destination node ID (`NodeId::*`).
    pub dst: u16,
    /// Monotonic sequence number for matching requests and responses.
    pub seq: u16,
    /// Payload length in bytes (does not include header or CRC).
    pub payload_len: u16,
}

impl Header {
    /// Creates a Domain 0 (`Control`) header with `PROTOCOL_VERSION`.
    pub const fn control(opcode: u16, src: u16, dst: u16, seq: u16, flags: Flags) -> Self {
        Self {
            version: PROTOCOL_VERSION,
            domain: Domain::Control,
            flags,
            opcode,
            src,
            dst,
            seq,
            payload_len: 0,
        }
    }

    /// Encodes the header into 12 little-endian bytes.
    pub fn to_bytes(&self) -> [u8; HEADER_LEN] {
        let mut out = [0u8; HEADER_LEN];
        out[0] = ((self.version & 0x0F) << 4) | (self.domain as u8 & 0x0F);
        out[1] = self.flags.0;
        out[2..4].copy_from_slice(&self.opcode.to_le_bytes());
        out[4..6].copy_from_slice(&self.src.to_le_bytes());
        out[6..8].copy_from_slice(&self.dst.to_le_bytes());
        out[8..10].copy_from_slice(&self.seq.to_le_bytes());
        out[10..12].copy_from_slice(&self.payload_len.to_le_bytes());
        out
    }

    /// Decodes 12 little-endian bytes into a validated header.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() < HEADER_LEN {
            return Err(Error::TruncatedPacket);
        }

        let version = (bytes[0] >> 4) & 0x0F;
        if version != PROTOCOL_VERSION {
            return Err(Error::UnsupportedVersion(version));
        }
        let domain = Domain::from_u8(bytes[0] & 0x0F)?;

        Ok(Self {
            version,
            domain,
            flags: Flags(bytes[1]),
            opcode: u16::from_le_bytes([bytes[2], bytes[3]]),
            src: u16::from_le_bytes([bytes[4], bytes[5]]),
            dst: u16::from_le_bytes([bytes[6], bytes[7]]),
            seq: u16::from_le_bytes([bytes[8], bytes[9]]),
            payload_len: u16::from_le_bytes([bytes[10], bytes[11]]),
        })
    }
}

/// Borrowed view of a verified ProtoSpeak packet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecodedPacket<'a> {
    /// Parsed 12-byte header.
    pub header: Header,
    /// Raw postcard payload bytes (exact length `header.payload_len`).
    pub payload: &'a [u8],
}

/// Encodes a header and raw payload slice into `out`, appending CRC-16.
/// Returns the total written byte count (`HEADER_LEN + payload.len() + CRC_LEN`).
pub fn encode_packet(mut header: Header, payload: &[u8], out: &mut [u8]) -> Result<usize, Error> {
    let payload_len = u16::try_from(payload.len()).map_err(|_| Error::BufferTooSmall)?;
    let total_len = HEADER_LEN
        .checked_add(payload.len())
        .and_then(|n| n.checked_add(CRC_LEN))
        .ok_or(Error::BufferTooSmall)?;

    if out.len() < total_len {
        return Err(Error::BufferTooSmall);
    }

    header.payload_len = payload_len;
    out[..HEADER_LEN].copy_from_slice(&header.to_bytes());
    out[HEADER_LEN..HEADER_LEN + payload.len()].copy_from_slice(payload);

    let crc = crc16_ccitt(&out[..HEADER_LEN + payload.len()]);
    out[HEADER_LEN + payload.len()..total_len].copy_from_slice(&crc.to_le_bytes());

    Ok(total_len)
}

/// Serializes `message` with `postcard` and wraps it in a full ProtoSpeak packet.
pub fn encode_payload<T: Serialize>(
    header: Header,
    message: &T,
    out: &mut [u8],
) -> Result<usize, Error> {
    if out.len() < HEADER_LEN + CRC_LEN {
        return Err(Error::BufferTooSmall);
    }

    // Serialize directly into the payload region of `out`, leaving room for the trailing CRC.
    let max_payload = out.len() - HEADER_LEN - CRC_LEN;
    let payload_slice = postcard::to_slice(message, &mut out[HEADER_LEN..HEADER_LEN + max_payload])
        .map_err(|err| match err {
            postcard::Error::SerializeBufferFull => Error::BufferTooSmall,
            _ => Error::Codec,
        })?;
    let payload_len = payload_slice.len();

    let mut final_header = header;
    final_header.payload_len = u16::try_from(payload_len).map_err(|_| Error::BufferTooSmall)?;
    out[..HEADER_LEN].copy_from_slice(&final_header.to_bytes());

    let crc_offset = HEADER_LEN + payload_len;
    let crc = crc16_ccitt(&out[..crc_offset]);
    out[crc_offset..crc_offset + CRC_LEN].copy_from_slice(&crc.to_le_bytes());

    Ok(crc_offset + CRC_LEN)
}

/// Parses and CRC-verifies a ProtoSpeak packet from `raw`.
///
/// Trailing padding bytes (for example zero-padding from the last 64-byte HID chunk)
/// are ignored once `HEADER_LEN + payload_len + CRC_LEN` bytes are validated.
pub fn decode_packet(raw: &[u8]) -> Result<DecodedPacket<'_>, Error> {
    if raw.len() < HEADER_LEN + CRC_LEN {
        return Err(Error::TruncatedPacket);
    }

    let header = Header::from_bytes(raw)?;
    let payload_len = header.payload_len as usize;
    let crc_offset = HEADER_LEN
        .checked_add(payload_len)
        .ok_or(Error::TruncatedPacket)?;
    let total_len = crc_offset
        .checked_add(CRC_LEN)
        .ok_or(Error::TruncatedPacket)?;

    if raw.len() < total_len {
        return Err(Error::TruncatedPacket);
    }

    let expected = crc16_ccitt(&raw[..crc_offset]);
    let actual = u16::from_le_bytes([raw[crc_offset], raw[crc_offset + 1]]);
    if expected != actual {
        return Err(Error::BadCrc { expected, actual });
    }

    Ok(DecodedPacket {
        header,
        payload: &raw[HEADER_LEN..crc_offset],
    })
}

/// Deserializes a postcard payload slice into `T`.
pub fn decode_payload<T: DeserializeOwned>(payload: &[u8]) -> Result<T, Error> {
    postcard::from_bytes(payload).map_err(|_| Error::Codec)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{HelloInfo, McuKind, PreviewState};

    #[test]
    fn header_round_trip_and_nibble_packing() {
        let header = Header {
            version: PROTOCOL_VERSION,
            domain: Domain::Control,
            flags: Flags::ACK_REQ.union(Flags::PRIORITY_SAFETY),
            opcode: Opcode::PREVIEW_STATE,
            src: NodeId::HOST,
            dst: NodeId::HEAD,
            seq: 0x1234,
            payload_len: 7,
        };

        let bytes = header.to_bytes();
        assert_eq!(bytes[0], 0x10); // version=1 (high nibble), domain=0 (low nibble)
        assert_eq!(bytes[1], 0x11); // ACK_REQ (0x01) | PRIORITY_SAFETY (0x10)
        assert_eq!(Header::from_bytes(&bytes), Ok(header));
    }

    #[test]
    fn encode_and_decode_preview_state() {
        let msg = PreviewState {
            expression_id: 2,
            tint_rgb: [56, 189, 248],
            brightness: 75,
            virtual_button_mask: 0b0000_0001,
        };
        let header = Header::control(
            Opcode::PREVIEW_STATE,
            NodeId::HOST,
            NodeId::HEAD,
            42,
            Flags::ACK_REQ,
        );

        let mut buf = [0u8; MAX_PACKET_SIZE];
        let written = encode_payload(header, &msg, &mut buf).ok();
        assert!(written.is_some());
        let written = written.unwrap_or(0);

        let decoded = decode_packet(&buf[..written]);
        assert!(decoded.is_ok());
        if let Ok(pkt) = decoded {
            assert_eq!(pkt.header.opcode, Opcode::PREVIEW_STATE);
            assert_eq!(pkt.header.seq, 42);
            assert_eq!(pkt.header.src, NodeId::HOST);
            assert_eq!(pkt.header.dst, NodeId::HEAD);
            assert!(pkt.header.flags.contains(Flags::ACK_REQ));
            assert_eq!(decode_payload::<PreviewState>(pkt.payload), Ok(msg));
        }
    }

    #[test]
    fn encode_and_decode_hello_info() {
        let info = HelloInfo {
            fw_major: 0,
            fw_minor: 1,
            fw_patch: 0,
            protocol_version: PROTOCOL_VERSION,
            mcu: McuKind::WasmSimulator,
            visor_width: 128,
            visor_height: 32,
            button_count: 1,
        };
        let header = Header::control(
            Opcode::HELLO_INFO,
            NodeId::HEAD,
            NodeId::HOST,
            1,
            Flags::IS_ACK,
        );

        let mut buf = [0u8; MAX_PACKET_SIZE];
        let len = encode_payload(header, &info, &mut buf).unwrap_or(0);
        assert!(len > HEADER_LEN + CRC_LEN);

        // Trailing zero bytes (like from a 64-byte HID chunk) must be tolerated.
        let decoded = decode_packet(&buf);
        assert!(decoded.is_ok());
        if let Ok(pkt) = decoded {
            assert_eq!(decode_payload::<HelloInfo>(pkt.payload), Ok(info));
        }
    }

    #[test]
    fn rejects_corrupted_payload_or_crc() {
        let header = Header::control(
            Opcode::PREVIEW_STATE,
            NodeId::HOST,
            NodeId::HEAD,
            7,
            Flags::NONE,
        );
        let mut buf = [0u8; 64];
        let len = encode_packet(header, &[1, 2, 3, 4], &mut buf).unwrap_or(0);

        // Flip one bit inside the payload.
        buf[HEADER_LEN + 1] ^= 0x04;
        assert!(matches!(
            decode_packet(&buf[..len]),
            Err(Error::BadCrc { .. })
        ));
    }

    #[test]
    fn rejects_wrong_version_and_truncated_input() {
        let mut buf = [0u8; 64];
        let header = Header::control(Opcode::HELLO_INFO, 0, 1, 1, Flags::NONE);
        let len = encode_packet(header, &[], &mut buf).unwrap_or(0);

        assert_eq!(decode_packet(&buf[..5]), Err(Error::TruncatedPacket));

        // Change version nibble from 1 to 2.
        buf[0] = 0x20;
        assert_eq!(
            decode_packet(&buf[..len]),
            Err(Error::UnsupportedVersion(2))
        );
    }
}
