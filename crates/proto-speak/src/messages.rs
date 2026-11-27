//! Typed Domain 0 control message payloads serialized with `postcard`.

use serde::{Deserialize, Serialize};

/// Hardware / runtime target reported in `HELLO_INFO`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum McuKind {
    /// Browser WASM simulator (`crates/proto-wasm`).
    WasmSimulator = 0,
    /// Teensy 4.1 (NXP i.MX RT1062).
    Teensy41 = 1,
    /// ESP32-S3 (planned v1.1+).
    Esp32S3 = 2,
}

/// Empty request payload for `Opcode::HELLO_INFO` (host -> device).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct HelloRequest;

/// Device identity and capabilities returned by `Opcode::HELLO_INFO` (device -> host).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HelloInfo {
    /// Semantic version major.
    pub fw_major: u8,
    /// Semantic version minor.
    pub fw_minor: u8,
    /// Semantic version patch.
    pub fw_patch: u8,
    /// Wire protocol version (`PROTOCOL_VERSION`).
    pub protocol_version: u8,
    /// Board or simulator runtime kind.
    pub mcu: McuKind,
    /// Logical visor width in pixels (e.g. 128).
    pub visor_width: u16,
    /// Logical visor height in pixels (e.g. 32).
    pub visor_height: u16,
    /// Number of configured paw buttons (0..=8).
    pub button_count: u8,
}

/// Live RAM preview update sent with `Opcode::PREVIEW_STATE` (host -> device).
/// Does not write to flash.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreviewState {
    /// Active starter expression index (0 = Neutral, 1 = Happy, 2 = Surprised).
    pub expression_id: u8,
    /// RGB tint multiplier applied to the visor expression.
    pub tint_rgb: [u8; 3],
    /// Master brightness percentage (`0..=100`).
    pub brightness: u8,
    /// Bitmask of virtual paw buttons currently pressed (bit 0 = button 1).
    pub virtual_button_mask: u8,
}

/// Acknowledgement payload for `Opcode::PREVIEW_STATE` (device -> host).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreviewAck {
    /// Expression currently rendered after applying button overrides.
    pub active_expression: u8,
    /// Effective brightness percentage after safety clamping (`0..=100`).
    pub effective_brightness: u8,
}
