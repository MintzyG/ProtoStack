//! Core `#![no_std]` visor engine and `ProtoSpeak` Domain 0 command handler.
//!
//! `ProtoBrain` ships with zero hardcoded faces. It renders active slots
//! through a pluggable [`FaceSource`] into a 24-bit RGB [`Framebuffer`] and
//! handles `ProtoSpeak` control packets. Because it has no OS or heap
//! dependencies, the browser WASM simulator and the Teensy 4.1 firmware run
//! the exact same engine.

#![no_std]

pub mod framebuffer;
pub mod slots;

pub use framebuffer::{
    Bitmap1Bit, Framebuffer, Rgb, DEFAULT_FRAMEBUFFER_RGBA_LEN, DEFAULT_FRAMEBUFFER_RGB_LEN,
    DEFAULT_VISOR_HEIGHT, DEFAULT_VISOR_WIDTH,
};
pub use slots::{
    FaceSlot, FaceSource, SlotBank, SlotError, DEFAULT_FRAMES_PER_SLOT, DEFAULT_MASK_BYTES,
    DEFAULT_SLOT_CAPACITY,
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

/// Core ProtoStack visor runtime state, generic over any [`FaceSource`] backend.
#[derive(Debug)]
pub struct ProtoBrain<S: FaceSource = SlotBank> {
    mcu: McuKind,
    button_count: u8,
    base_slot: u8,
    button_override_slot: u8,
    tint: Rgb,
    brightness: u8,
    virtual_button_mask: u32,
    frame: u32,
    source: S,
    framebuffer: Framebuffer,
}

impl ProtoBrain<SlotBank> {
    /// Creates a new `ProtoBrain` backed by an empty in-RAM [`SlotBank`], default
    /// cyan tint, and 80% brightness. The framebuffer stays blank until faces are loaded.
    pub fn new(mcu: McuKind, button_count: u8) -> Self {
        Self::with_source(mcu, button_count, SlotBank::new())
    }
}

impl<S: FaceSource> ProtoBrain<S> {
    /// Creates a `ProtoBrain` with a custom [`FaceSource`] backend (for example
    /// a multi-color GIF flash store or a 3D blendshape renderer).
    pub fn with_source(mcu: McuKind, button_count: u8, source: S) -> Self {
        let mut brain = Self {
            mcu,
            button_count,
            base_slot: 0,
            button_override_slot: 1,
            tint: Rgb::DEFAULT_TINT,
            brightness: 80,
            virtual_button_mask: 0,
            frame: 0,
            source,
            framebuffer: Framebuffer::new(),
        };
        brain.render();
        brain
    }

    /// Returns a read-only reference to the active [`FaceSource`] backend.
    pub const fn source(&self) -> &S {
        &self.source
    }

    /// Returns a mutable reference to the active [`FaceSource`] backend.
    /// Call [`Self::render`] after modifying loaded face data to refresh the framebuffer.
    pub fn source_mut(&mut self) -> &mut S {
        &mut self.source
    }

    /// Returns the base slot index selected by the user (before temporary button overrides).
    pub const fn base_slot(&self) -> u8 {
        self.base_slot
    }

    /// Returns the slot index temporarily activated while virtual button 1 is pressed.
    pub const fn button_override_slot(&self) -> u8 {
        self.button_override_slot
    }

    /// Sets which slot index virtual button 1 activates while pressed.
    pub fn set_button_override_slot(&mut self, slot_id: u8) {
        self.button_override_slot = slot_id;
        self.render();
    }

    /// Returns the currently displayed slot index after applying active button overrides.
    pub const fn active_slot(&self) -> u8 {
        if (self.virtual_button_mask & 1) != 0 {
            if self.base_slot == self.button_override_slot {
                0
            } else {
                self.button_override_slot
            }
        } else {
            self.base_slot
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

    /// Returns the current animation tick counter.
    pub const fn frame(&self) -> u32 {
        self.frame
    }

    /// Advances the animation by one tick and re-renders the framebuffer.
    pub fn tick(&mut self) {
        self.frame = self.frame.wrapping_add(1);
        self.render();
    }

    /// Sets the animation tick counter explicitly and re-renders.
    pub fn set_frame(&mut self, frame: u32) {
        self.frame = frame;
        self.render();
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
        self.base_slot = preview.expression_id;
        self.tint = Rgb::from_array(preview.tint_rgb);
        self.brightness = preview.brightness.min(100);
        self.virtual_button_mask = preview.virtual_button_mask;
        self.render();

        PreviewAck {
            active_expression: self.active_slot(),
            effective_brightness: self.brightness,
        }
    }

    /// Renders the active slot and current animation tick into `self.framebuffer`.
    pub fn render(&mut self) {
        self.framebuffer.clear();
        let slot_id = self.active_slot();
        self.source.render_slot(
            slot_id,
            self.frame,
            self.tint,
            self.brightness,
            &mut self.framebuffer,
        );
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
