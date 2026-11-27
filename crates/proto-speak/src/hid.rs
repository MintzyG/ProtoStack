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
