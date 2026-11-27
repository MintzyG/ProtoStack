//! Thin `wasm-bindgen` bridge connecting `web/` to `proto-brain` and `proto-speak`.
//!
//! All simulator commands flow through real 64-byte `ProtoSpeak` HID reports
//! with CRC-16 validation so the browser simulator runs the exact same wire
//! format and Rust engine as the Teensy 4.1 firmware.

use proto_brain::{BrainError, ProtoBrain, SlotError};
use proto_speak::{
    decode_packet, decode_payload, encode_payload, Flags, Header, HelloInfo, HelloRequest,
    HidChunker, HidReassembler, McuKind, NodeId, Opcode, PreviewAck, PreviewState, HID_REPORT_SIZE,
    MAX_PACKET_SIZE,
};
use wasm_bindgen::prelude::*;

/// Host-friendly snapshot of a decoded `HELLO_INFO` response.
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JsHelloInfo {
    /// Firmware major version.
    pub fw_major: u8,
    /// Firmware minor version.
    pub fw_minor: u8,
    /// Firmware patch version.
    pub fw_patch: u8,
    /// Wire protocol version.
    pub protocol_version: u8,
    /// MCU kind raw discriminant (`0 = WasmSimulator`, `1 = Teensy41`, `2 = Esp32S3`).
    pub mcu_kind: u8,
    /// Logical visor canvas width in pixels.
    pub visor_width: u16,
    /// Logical visor canvas height in pixels.
    pub visor_height: u16,
    /// Number of configured input buttons.
    pub button_count: u8,
}

impl From<HelloInfo> for JsHelloInfo {
    fn from(info: HelloInfo) -> Self {
        Self {
            fw_major: info.fw_major,
            fw_minor: info.fw_minor,
            fw_patch: info.fw_patch,
            protocol_version: info.protocol_version,
            mcu_kind: info.mcu as u8,
            visor_width: info.visor_width,
            visor_height: info.visor_height,
            button_count: info.button_count,
        }
    }
}

/// Host-friendly snapshot of a decoded `PREVIEW_STATE` acknowledgement.
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JsPreviewAck {
    /// Active face slot after applying button overrides.
    pub active_expression: u8,
    /// Effective brightness percentage after clamping (`0..=100`).
    pub effective_brightness: u8,
}

impl From<PreviewAck> for JsPreviewAck {
    fn from(ack: PreviewAck) -> Self {
        Self {
            active_expression: ack.active_expression,
            effective_brightness: ack.effective_brightness,
        }
    }
}

/// Simulated helmet backed by `ProtoBrain` and 64-byte `HidReassembler`.
#[wasm_bindgen]
pub struct WasmHelmet {
    brain: ProtoBrain,
    reassembler: HidReassembler<MAX_PACKET_SIZE>,
}

#[wasm_bindgen]
impl WasmHelmet {
    /// Creates a blank simulated helmet with `button_count` virtual buttons.
    #[wasm_bindgen(constructor)]
    pub fn new(button_count: u8) -> Self {
        Self {
            brain: ProtoBrain::new(McuKind::WasmSimulator, button_count),
            reassembler: HidReassembler::new(),
        }
    }

    /// Loads one 1-bit MSB-first packed mask frame into `slot_id` at `frame_index`.
    pub fn load_slot_frame(
        &mut self,
        slot_id: u8,
        frame_index: u8,
        width: u16,
        height: u16,
        mask: &[u8],
    ) -> Result<(), JsValue> {
        self.load_slot_frame_raw(slot_id, frame_index, width, height, mask)
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))
    }

    /// Sets how many ticks each animation frame is held for `slot_id`.
    pub fn set_slot_frame_ticks(&mut self, slot_id: u8, ticks: u8) -> Result<(), JsValue> {
        self.brain
            .source_mut()
            .set_frame_ticks(slot_id, ticks)
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))
    }

    /// Sets which slot index virtual button 1 temporarily activates while held.
    pub fn set_button_override_slot(&mut self, slot_id: u8) {
        self.brain.set_button_override_slot(slot_id);
    }

    /// Clears `slot_id` back to blank and re-renders.
    pub fn clear_slot(&mut self, slot_id: u8) -> Result<(), JsValue> {
        self.brain
            .source_mut()
            .clear_slot(slot_id)
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))?;
        self.brain.render();
        Ok(())
    }

    /// Feeds one 64-byte `ProtoSpeak` HID report into the simulated helmet.
    ///
    /// Returns flat concatenated 64-byte response HID reports once a complete
    /// packet has been processed, or an empty vector if more chunks are pending.
    pub fn push_hid_report(&mut self, report: &[u8]) -> Result<Vec<u8>, JsValue> {
        self.push_hid_report_raw(report)
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))
    }

    /// Advances the visor animation by one tick and re-renders.
    pub fn tick(&mut self) {
        self.brain.tick();
    }

    /// Active logical visor width in pixels.
    pub fn visor_width(&self) -> u16 {
        self.brain.visor_width()
    }

    /// Active logical visor height in pixels.
    pub fn visor_height(&self) -> u16 {
        self.brain.visor_height()
    }

    /// Active face slot index after applying button overrides.
    pub fn active_slot(&self) -> u8 {
        self.brain.active_slot()
    }

    /// Effective brightness percentage (`0..=100`).
    pub fn brightness(&self) -> u8 {
        self.brain.brightness()
    }

    /// Copies the current framebuffer into a freshly sized RGBA byte vector for HTML5 Canvas.
    pub fn render_rgba(&self) -> Vec<u8> {
        let fb = self.brain.framebuffer();
        let mut out = vec![0u8; fb.rgba_len()];
        let _ = fb.copy_rgba_into(&mut out);
        out
    }
}

impl WasmHelmet {
    /// Pure-Rust slot frame loader (usable in native tests without `JsValue`).
    pub fn load_slot_frame_raw(
        &mut self,
        slot_id: u8,
        frame_index: u8,
        width: u16,
        height: u16,
        mask: &[u8],
    ) -> Result<(), SlotError> {
        self.brain
            .source_mut()
            .load_frame(slot_id, frame_index, width, height, mask)?;
        self.brain.render();
        Ok(())
    }

    /// Pure-Rust 64-byte HID report handler (usable in native tests without `JsValue`).
    pub fn push_hid_report_raw(&mut self, report: &[u8]) -> Result<Vec<u8>, BrainError> {
        let fixed: [u8; HID_REPORT_SIZE] = report
            .try_into()
            .map_err(|_| BrainError::Protocol(proto_speak::Error::TruncatedPacket))?;

        let Some(packet_bytes) = self.reassembler.push(&fixed)? else {
            return Ok(Vec::new());
        };

        let mut packet_copy = [0u8; MAX_PACKET_SIZE];
        let len = packet_bytes.len();
        if len > MAX_PACKET_SIZE {
            return Err(BrainError::Protocol(proto_speak::Error::PacketTooLarge));
        }
        packet_copy[..len].copy_from_slice(packet_bytes);

        let mut resp_buf = [0u8; MAX_PACKET_SIZE];
        let resp_len = self
            .brain
            .handle_packet(&packet_copy[..len], &mut resp_buf)?;
        chunk_packet_to_flat_hid(&resp_buf[..resp_len]).map_err(BrainError::Protocol)
    }
}

/// Encodes a `HELLO_INFO` request into concatenated 64-byte HID reports.
#[wasm_bindgen]
pub fn encode_hello_hid_reports(seq: u16) -> Result<Vec<u8>, JsValue> {
    encode_hello_hid_reports_raw(seq).map_err(|e| JsValue::from_str(&format!("{e:?}")))
}

/// Encodes a `PREVIEW_STATE` command into concatenated 64-byte HID reports.
#[wasm_bindgen]
pub fn encode_preview_hid_reports(
    seq: u16,
    expression_id: u8,
    tint_r: u8,
    tint_g: u8,
    tint_b: u8,
    brightness: u8,
    virtual_button_mask: u32,
) -> Result<Vec<u8>, JsValue> {
    let preview = PreviewState {
        expression_id,
        tint_rgb: [tint_r, tint_g, tint_b],
        brightness,
        virtual_button_mask,
    };
    encode_preview_hid_reports_raw(seq, preview).map_err(|e| JsValue::from_str(&format!("{e:?}")))
}

/// Decodes concatenated 64-byte HID response reports into [`JsHelloInfo`].
#[wasm_bindgen]
pub fn decode_hello_hid_reports(reports: &[u8]) -> Result<JsHelloInfo, JsValue> {
    decode_hello_hid_reports_raw(reports)
        .map(JsHelloInfo::from)
        .map_err(|e| JsValue::from_str(&format!("{e:?}")))
}

/// Decodes concatenated 64-byte HID response reports into [`JsPreviewAck`].
#[wasm_bindgen]
pub fn decode_preview_ack_hid_reports(reports: &[u8]) -> Result<JsPreviewAck, JsValue> {
    decode_preview_ack_hid_reports_raw(reports)
        .map(JsPreviewAck::from)
        .map_err(|e| JsValue::from_str(&format!("{e:?}")))
}

/// Pure-Rust helper: encodes `HELLO_INFO` into concatenated 64-byte HID reports.
pub fn encode_hello_hid_reports_raw(seq: u16) -> Result<Vec<u8>, proto_speak::Error> {
    let header = Header::control(
        Opcode::HELLO_INFO,
        NodeId::HOST,
        NodeId::HEAD,
        seq,
        Flags::ACK_REQ,
    );
    let mut buf = [0u8; MAX_PACKET_SIZE];
    let len = encode_payload(header, &HelloRequest, &mut buf)?;
    chunk_packet_to_flat_hid(&buf[..len])
}

/// Pure-Rust helper: encodes `PREVIEW_STATE` into concatenated 64-byte HID reports.
pub fn encode_preview_hid_reports_raw(
    seq: u16,
    preview: PreviewState,
) -> Result<Vec<u8>, proto_speak::Error> {
    let header = Header::control(
        Opcode::PREVIEW_STATE,
        NodeId::HOST,
        NodeId::HEAD,
        seq,
        Flags::ACK_REQ,
    );
    let mut buf = [0u8; MAX_PACKET_SIZE];
    let len = encode_payload(header, &preview, &mut buf)?;
    chunk_packet_to_flat_hid(&buf[..len])
}

/// Pure-Rust helper: reassembles and decodes a `HELLO_INFO` response from 64-byte HID reports.
pub fn decode_hello_hid_reports_raw(reports: &[u8]) -> Result<HelloInfo, proto_speak::Error> {
    let raw_pkt = reassemble_flat_hid(reports)?;
    let decoded = decode_packet(&raw_pkt)?;
    if decoded.header.opcode != Opcode::HELLO_INFO {
        return Err(proto_speak::Error::Codec);
    }
    decode_payload(decoded.payload)
}

/// Pure-Rust helper: reassembles and decodes a `PREVIEW_STATE` ack from 64-byte HID reports.
pub fn decode_preview_ack_hid_reports_raw(
    reports: &[u8],
) -> Result<PreviewAck, proto_speak::Error> {
    let raw_pkt = reassemble_flat_hid(reports)?;
    let decoded = decode_packet(&raw_pkt)?;
    if decoded.header.opcode != Opcode::PREVIEW_STATE {
        return Err(proto_speak::Error::Codec);
    }
    decode_payload(decoded.payload)
}

fn chunk_packet_to_flat_hid(packet: &[u8]) -> Result<Vec<u8>, proto_speak::Error> {
    let chunker = HidChunker::new(packet)?;
    let mut out = Vec::with_capacity((chunker.total_chunks() as usize) * HID_REPORT_SIZE);
    for report in chunker {
        out.extend_from_slice(&report);
    }
    Ok(out)
}

fn reassemble_flat_hid(reports: &[u8]) -> Result<Vec<u8>, proto_speak::Error> {
    let (chunks, remainder) = reports.as_chunks::<HID_REPORT_SIZE>();
    if chunks.is_empty() || !remainder.is_empty() {
        return Err(proto_speak::Error::TruncatedPacket);
    }
    let mut reassembler = HidReassembler::<MAX_PACKET_SIZE>::new();
    for fixed in chunks {
        if let Some(pkt) = reassembler.push(fixed)? {
            return Ok(pkt.to_vec());
        }
    }
    Err(proto_speak::Error::TruncatedPacket)
}
