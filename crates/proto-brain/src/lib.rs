//! Core `#![no_std]` visor engine and `ProtoSpeak` Domain 0 command handler.
//!
//! `ProtoBrain` owns the active visor state (`Expression`, tint, brightness,
//! virtual button mask) and the `128x32` RGB framebuffer. Because it has no OS
//! or heap dependencies, the browser WASM simulator and the Teensy 4.1 firmware
//! run the exact same engine.

#![no_std]

pub mod faces;
pub mod framebuffer;

pub use faces::{blink_open_pct, render_expression, Expression};
pub use framebuffer::{
    Bitmap1Bit, Framebuffer, Rgb, DEFAULT_FRAMEBUFFER_RGBA_LEN, DEFAULT_FRAMEBUFFER_RGB_LEN,
    DEFAULT_VISOR_HEIGHT, DEFAULT_VISOR_WIDTH,
};

use proto_speak::{
    decode_packet, decode_payload, encode_payload, Flags, Header, HelloInfo, McuKind, NodeId,
    Opcode, PreviewAck, PreviewState, PROTOCOL_VERSION,
};

/// Errors returned while processing an incoming `ProtoSpeak` packet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrainError {
    /// Packet framing, CRC, or postcard codec error.
    Protocol(proto_speak::Error),
    /// Packet opcode is not handled by this firmware version.
    UnsupportedOpcode(u16),
}

impl From<proto_speak::Error> for BrainError {
    fn from(err: proto_speak::Error) -> Self {
        Self::Protocol(err)
    }
}

/// Core ProtoStack visor runtime state.
#[derive(Debug)]
pub struct ProtoBrain {
    mcu: McuKind,
    button_count: u8,
    base_expression: Expression,
    tint: Rgb,
    brightness: u8,
    virtual_button_mask: u32,
    frame: u32,
    framebuffer: Framebuffer,
}

impl ProtoBrain {
    /// Creates a new `ProtoBrain` initialized to `Neutral`, default cyan tint,
    /// and 80% brightness, with the initial frame already rendered.
    pub fn new(mcu: McuKind, button_count: u8) -> Self {
        let mut brain = Self {
            mcu,
            button_count,
            base_expression: Expression::Neutral,
            tint: Rgb::DEFAULT_TINT,
            brightness: 80,
            virtual_button_mask: 0,
            frame: 0,
            framebuffer: Framebuffer::new(),
        };
        brain.render();
        brain
    }

    /// Returns the current animation frame counter.
    pub const fn frame(&self) -> u32 {
        self.frame
    }

    /// Advances the animation by one frame and re-renders the framebuffer.
    pub fn tick(&mut self) {
        self.frame = self.frame.wrapping_add(1);
        self.render();
    }

    /// Sets the animation frame counter explicitly and re-renders.
    pub fn set_frame(&mut self, frame: u32) {
        self.frame = frame;
        self.render();
    }

    /// Returns the base expression chosen by the user (before temporary button overrides).
    pub const fn base_expression(&self) -> Expression {
        self.base_expression
    }

    /// Returns the currently displayed expression after applying active button overrides.
    pub const fn active_expression(&self) -> Expression {
        if (self.virtual_button_mask & 1) != 0 {
            match self.base_expression {
                Expression::Surprised => Expression::Happy,
                _ => Expression::Surprised,
            }
        } else {
            self.base_expression
        }
    }

    /// Returns the current RGB tint color.
    pub const fn tint(&self) -> Rgb {
        self.tint
    }

    /// Returns the effective brightness percentage (`0..=100`).
    pub const fn brightness(&self) -> u8 {
        self.brightness
    }

    /// Returns the active virtual button bitmask.
    pub const fn virtual_button_mask(&self) -> u32 {
        self.virtual_button_mask
    }

    /// Returns a read-only reference to the rendered RGB framebuffer.
    pub const fn framebuffer(&self) -> &Framebuffer {
        &self.framebuffer
    }

    /// Returns the active logical visor width in pixels.
    pub const fn visor_width(&self) -> u16 {
        self.framebuffer.width() as u16
    }

    /// Returns the active logical visor height in pixels.
    pub const fn visor_height(&self) -> u16 {
        self.framebuffer.height() as u16
    }

    /// Updates state from a `PreviewState` payload and immediately re-renders the framebuffer.
    pub fn apply_preview(&mut self, preview: PreviewState) -> PreviewAck {
        self.base_expression = Expression::from_u8(preview.expression_id);
        self.tint = Rgb::from_array(preview.tint_rgb);
        self.brightness = preview.brightness.min(100);
        self.virtual_button_mask = preview.virtual_button_mask;
        self.render();

        PreviewAck {
            active_expression: self.active_expression().as_u8(),
            effective_brightness: self.brightness,
        }
    }

    /// Renders the current state and animation frame into `self.framebuffer`.
    pub fn render(&mut self) {
        let expr = self.active_expression();
        let effective_color = self.tint.scale(self.brightness);
        render_expression(&mut self.framebuffer, expr, self.frame, effective_color);
    }

    /// Decodes one complete `ProtoSpeak` packet, applies it, and writes the response packet
    /// into `response_out`. Returns the number of response bytes written.
    pub fn handle_packet(
        &mut self,
        raw_packet: &[u8],
        response_out: &mut [u8],
    ) -> Result<usize, BrainError> {
        let pkt = decode_packet(raw_packet)?;
        let ack_header = Header::control(
            pkt.header.opcode,
            NodeId::HEAD,
            pkt.header.src,
            pkt.header.seq,
            Flags::IS_ACK,
        );

        match pkt.header.opcode {
            Opcode::HELLO_INFO => {
                let info = HelloInfo {
                    fw_major: 0,
                    fw_minor: 1,
                    fw_patch: 0,
                    protocol_version: PROTOCOL_VERSION,
                    mcu: self.mcu,
                    visor_width: self.visor_width(),
                    visor_height: self.visor_height(),
                    button_count: self.button_count,
                };
                Ok(encode_payload(ack_header, &info, response_out)?)
            }
            Opcode::PREVIEW_STATE => {
                let preview: PreviewState = decode_payload(pkt.payload)?;
                let ack = self.apply_preview(preview);
                Ok(encode_payload(ack_header, &ack, response_out)?)
            }
            other => Err(BrainError::UnsupportedOpcode(other)),
        }
    }
}
