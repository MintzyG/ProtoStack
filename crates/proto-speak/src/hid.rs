//! 64-byte USB WebHID framing and reassembly for ProtoSpeak packets.
//!
//! Each HID report is fixed at 64 bytes:
//! ```text
//! Byte 0:     Report ID (0x01)
//! Byte 1:     chunk_index (0 .. total_chunks - 1)
//! Byte 2:     total_chunks (1 ..= 255)
//! Bytes 3..64: Up to 61 bytes of ProtoSpeak packet data (zero-padded on the last chunk)
//! ```

use crate::Error;

/// Fixed USB HID report length in bytes.
pub const HID_REPORT_SIZE: usize = 64;

/// HID Report ID used for ProtoSpeak control packets.
pub const HID_REPORT_ID: u8 = 0x01;

/// Number of framing header bytes inside every 64-byte HID report.
pub const HID_HEADER_LEN: usize = 3;

/// Maximum ProtoSpeak bytes carried per 64-byte HID report.
pub const HID_CHUNK_DATA_LEN: usize = HID_REPORT_SIZE - HID_HEADER_LEN; // 61 bytes

/// Iterator that splits a ProtoSpeak packet slice into 64-byte HID reports.
#[derive(Debug, Clone)]
pub struct HidChunker<'a> {
    packet: &'a [u8],
    chunk_index: u8,
    total_chunks: u8,
}

impl<'a> HidChunker<'a> {
    /// Creates a chunker for `packet`. Fails if the packet is empty or needs more
    /// than 255 chunks (> 15,555 bytes).
    pub fn new(packet: &'a [u8]) -> Result<Self, Error> {
        if packet.is_empty() {
            return Err(Error::TruncatedPacket);
        }
        let chunks = packet.len().div_ceil(HID_CHUNK_DATA_LEN);
        let total_chunks = u8::try_from(chunks).map_err(|_| Error::PacketTooLarge)?;
        Ok(Self {
            packet,
            chunk_index: 0,
            total_chunks,
        })
    }

    /// Total number of 64-byte HID reports this packet will emit.
    pub fn total_chunks(&self) -> u8 {
        self.total_chunks
    }
}

impl Iterator for HidChunker<'_> {
    type Item = [u8; HID_REPORT_SIZE];

    fn next(&mut self) -> Option<Self::Item> {
        if self.chunk_index >= self.total_chunks {
            return None;
        }

        let start = (self.chunk_index as usize) * HID_CHUNK_DATA_LEN;
        let end = (start + HID_CHUNK_DATA_LEN).min(self.packet.len());
        let slice = &self.packet[start..end];

        let mut report = [0u8; HID_REPORT_SIZE];
        report[0] = HID_REPORT_ID;
        report[1] = self.chunk_index;
        report[2] = self.total_chunks;
        report[HID_HEADER_LEN..HID_HEADER_LEN + slice.len()].copy_from_slice(slice);

        self.chunk_index += 1;
        Some(report)
    }
}

/// Fixed-buffer state machine that reassembles 64-byte HID reports into a full packet.
#[derive(Debug)]
pub struct HidReassembler<const MAX_BUF: usize> {
    buf: [u8; MAX_BUF],
    written: usize,
    expected_chunk: u8,
    expected_total: u8,
}

impl<const MAX_BUF: usize> Default for HidReassembler<MAX_BUF> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const MAX_BUF: usize> HidReassembler<MAX_BUF> {
    /// Creates an empty reassembler ready for chunk 0.
    pub const fn new() -> Self {
        Self {
            buf: [0u8; MAX_BUF],
            written: 0,
            expected_chunk: 0,
            expected_total: 0,
        }
    }

    /// Discards any in-progress packet.
    pub fn reset(&mut self) {
        self.written = 0;
        self.expected_chunk = 0;
        self.expected_total = 0;
    }

    /// Feeds one 64-byte HID report.
    ///
    /// Returns:
    /// - `Ok(None)` when more chunks are still needed.
    /// - `Ok(Some(&[u8]))` when the final chunk completes the packet. The reassembler
    ///   resets automatically for the next packet.
    /// - `Err(_)` on invalid or out-of-order reports (state resets cleanly).
    pub fn push(&mut self, report: &[u8; HID_REPORT_SIZE]) -> Result<Option<&[u8]>, Error> {
        if report[0] != HID_REPORT_ID {
            self.reset();
            return Err(Error::InvalidReportId(report[0]));
        }

        let chunk_index = report[1];
        let total_chunks = report[2];

        if total_chunks == 0 || chunk_index >= total_chunks {
            self.reset();
            return Err(Error::InvalidChunkHeader);
        }

        // Starting a fresh message on chunk 0.
        if chunk_index == 0 {
            self.written = 0;
            self.expected_chunk = 0;
            self.expected_total = total_chunks;
        } else if chunk_index != self.expected_chunk || total_chunks != self.expected_total {
            let expected = self.expected_chunk;
            self.reset();
            return Err(Error::OutOfOrderChunk {
                expected,
                actual: chunk_index,
            });
        }

        let remaining = MAX_BUF.saturating_sub(self.written);
        if remaining == 0 {
            self.reset();
            return Err(Error::PacketTooLarge);
        }

        // Copy up to 61 bytes; on the final chunk we may trim trailing padding if
        // `MAX_BUF` is not a multiple of 61, while keeping the full packet intact.
        let copy_len = if chunk_index + 1 < total_chunks {
            if remaining < HID_CHUNK_DATA_LEN {
                self.reset();
                return Err(Error::PacketTooLarge);
            }
            HID_CHUNK_DATA_LEN
        } else {
            HID_CHUNK_DATA_LEN.min(remaining)
        };

        self.buf[self.written..self.written + copy_len]
            .copy_from_slice(&report[HID_HEADER_LEN..HID_HEADER_LEN + copy_len]);
        self.written += copy_len;
        self.expected_chunk += 1;

        if self.expected_chunk == self.expected_total {
            let len = self.written;
            self.written = 0;
            self.expected_chunk = 0;
            self.expected_total = 0;
            Ok(Some(&self.buf[..len]))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{decode_packet, encode_packet, Flags, Header, NodeId, Opcode, MAX_PACKET_SIZE};

    #[test]
    fn single_chunk_round_trip() {
        let header = Header::control(
            Opcode::HELLO_INFO,
            NodeId::HOST,
            NodeId::HEAD,
            3,
            Flags::NONE,
        );
        let mut raw = [0u8; 64];
        let len = encode_packet(header, &[0xAA, 0xBB], &mut raw).unwrap_or(0);

        let mut chunker = match HidChunker::new(&raw[..len]) {
            Ok(c) => c,
            Err(e) => panic!("unexpected error: {e:?}"),
        };
        assert_eq!(chunker.total_chunks(), 1);

        let report = match chunker.next() {
            Some(r) => r,
            None => panic!("expected one HID report"),
        };
        assert!(chunker.next().is_none());
        assert_eq!(report[0], HID_REPORT_ID);
        assert_eq!(report[1], 0);
        assert_eq!(report[2], 1);

        let mut reassembler = HidReassembler::<MAX_PACKET_SIZE>::new();
        let assembled = match reassembler.push(&report) {
            Ok(Some(bytes)) => bytes,
            other => panic!("expected complete packet, got {other:?}"),
        };

        let pkt = match decode_packet(assembled) {
            Ok(p) => p,
            Err(e) => panic!("decode failed: {e:?}"),
        };
        assert_eq!(pkt.header.seq, 3);
        assert_eq!(pkt.payload, &[0xAA, 0xBB]);
    }

    #[test]
    fn multi_chunk_round_trip() {
        // 150-byte payload + 12 header + 2 CRC = 164 bytes -> 3 HID chunks (61 + 61 + 42).
        let payload = [0x5Au8; 150];
        let header = Header::control(
            Opcode::PREVIEW_STATE,
            NodeId::HOST,
            NodeId::HEAD,
            99,
            Flags::ACK_REQ,
        );
        let mut raw = [0u8; MAX_PACKET_SIZE];
        let len = encode_packet(header, &payload, &mut raw).unwrap_or(0);
        assert_eq!(len, 164);

        let chunker = match HidChunker::new(&raw[..len]) {
            Ok(c) => c,
            Err(e) => panic!("unexpected error: {e:?}"),
        };
        assert_eq!(chunker.total_chunks(), 3);

        let mut reassembler = HidReassembler::<MAX_PACKET_SIZE>::new();
        let mut finished_len = 0usize;
        let mut out_copy = [0u8; MAX_PACKET_SIZE];

        for (i, report) in chunker.enumerate() {
            let res = reassembler.push(&report);
            if i < 2 {
                assert_eq!(res, Ok(None));
            } else if let Ok(Some(bytes)) = res {
                finished_len = bytes.len();
                out_copy[..finished_len].copy_from_slice(bytes);
            } else {
                panic!("expected final chunk to complete packet, got {res:?}");
            }
        }

        let pkt = match decode_packet(&out_copy[..finished_len]) {
            Ok(p) => p,
            Err(e) => panic!("decode failed: {e:?}"),
        };
        assert_eq!(pkt.header.seq, 99);
        assert_eq!(pkt.payload, &payload);
    }

    #[test]
    fn rejects_bad_report_id_and_out_of_order_chunks() {
        let mut reassembler = HidReassembler::<MAX_PACKET_SIZE>::new();

        let mut bad_id = [0u8; HID_REPORT_SIZE];
        bad_id[0] = 0x02;
        bad_id[1] = 0;
        bad_id[2] = 1;
        assert_eq!(reassembler.push(&bad_id), Err(Error::InvalidReportId(0x02)));

        let mut zero_total = [0u8; HID_REPORT_SIZE];
        zero_total[0] = HID_REPORT_ID;
        zero_total[1] = 0;
        zero_total[2] = 0;
        assert_eq!(
            reassembler.push(&zero_total),
            Err(Error::InvalidChunkHeader)
        );

        // Sending chunk 1 before chunk 0 fails as out-of-order.
        let mut ooo = [0u8; HID_REPORT_SIZE];
        ooo[0] = HID_REPORT_ID;
        ooo[1] = 1;
        ooo[2] = 2;
        assert_eq!(
            reassembler.push(&ooo),
            Err(Error::OutOfOrderChunk {
                expected: 0,
                actual: 1
            })
        );
    }
}
