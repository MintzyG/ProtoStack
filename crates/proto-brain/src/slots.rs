//! Pluggable [`FaceSource`] trait and default in-RAM 1-bit [`SlotBank`].
//!
//! `proto-brain` ships with zero hardcoded faces and does not assume a fixed
//! slot count or pixel format. Any backend that implements [`FaceSource`]
//! (in-RAM 1-bit masks, flash/SD multi-color GIF stores, or real-time 3D
//! blendshape renderers) can drive the 24-bit RGB [`Framebuffer`].

use crate::framebuffer::{Bitmap1Bit, Framebuffer, Rgb, DEFAULT_VISOR_HEIGHT, DEFAULT_VISOR_WIDTH};

/// Default in-RAM slot count for [`SlotBank`] in v0.1 (protocol allows up to 256).
pub const DEFAULT_SLOT_CAPACITY: usize = 8;

/// Default maximum animation frames per slot for [`SlotBank`] in v0.1.
pub const DEFAULT_FRAMES_PER_SLOT: usize = 4;

/// Maximum 1-bit packed mask size per frame for [`SlotBank`] (`128 * 32 / 8 = 512` bytes).
pub const DEFAULT_MASK_BYTES: usize = DEFAULT_VISOR_WIDTH.div_ceil(8) * DEFAULT_VISOR_HEIGHT;

/// Pluggable visor renderer / asset store consumed by [`crate::ProtoBrain`].
///
/// Backends can render 1-bit tinted sprites ([`SlotBank`]), full-color animated
/// GIF/RGB565 frames from flash or MicroSD, or live 3D blendshape meshes into
/// the 24-bit RGB [`Framebuffer`].
pub trait FaceSource {
    /// Renders `slot_id` at animation `tick` into `fb` (already cleared to black).
    ///
    /// `tint` is the user color tint (`[255, 255, 255]` leaves full-color assets
    /// untinted) and `brightness` is clamped to `0..=100`. Empty or unknown
    /// slots should leave `fb` blank.
    fn render_slot(&self, slot_id: u8, tick: u32, tint: Rgb, brightness: u8, fb: &mut Framebuffer);
}

/// Errors returned while loading frames into a [`SlotBank`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotError {
    /// Slot index is outside `0..self.capacity()`.
    InvalidSlot(u8),
    /// Frame index is outside `0..slot.max_frames()`.
    InvalidFrameIndex(u8),
    /// Frame dimensions are zero or exceed the fixed mask storage capacity.
    InvalidDimensions {
        /// Requested frame width in pixels.
        width: u16,
        /// Requested frame height in pixels.
        height: u16,
    },
    /// Supplied mask byte slice does not match `width.div_ceil(8) * height`.
    InvalidMaskLength {
        /// Required byte count for `(width, height)`.
        expected: usize,
        /// Byte count provided by caller.
        actual: usize,
    },
}

/// One animated 1-bit face slot holding up to [`DEFAULT_FRAMES_PER_SLOT`] frames in RAM.
#[derive(Debug, Clone)]
pub struct FaceSlot {
    width: u16,
    height: u16,
    frame_count: u8,
    frame_ticks: u8,
    frames: [[u8; DEFAULT_MASK_BYTES]; DEFAULT_FRAMES_PER_SLOT],
}

impl Default for FaceSlot {
    fn default() -> Self {
        Self::empty()
    }
}

impl FaceSlot {
    /// Creates an empty slot (renders blank until frames are loaded).
    pub const fn empty() -> Self {
        Self {
            width: 0,
            height: 0,
            frame_count: 0,
            frame_ticks: 1,
            frames: [[0u8; DEFAULT_MASK_BYTES]; DEFAULT_FRAMES_PER_SLOT],
        }
    }

    /// Maximum number of frames this slot can store.
    pub const fn max_frames(&self) -> usize {
        DEFAULT_FRAMES_PER_SLOT
    }

    /// Returns true if no frames have been loaded into this slot.
    pub const fn is_empty(&self) -> bool {
        self.frame_count == 0
    }

    /// Returns the number of loaded animation frames.
    pub const fn frame_count(&self) -> u8 {
        self.frame_count
    }

    /// Returns the number of ticks each frame is held before advancing.
    pub const fn frame_ticks(&self) -> u8 {
        self.frame_ticks
    }

    /// Returns the frame dimensions `(width, height)` in pixels.
    pub const fn dimensions(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    /// Clears all frames in this slot back to empty.
    pub fn clear(&mut self) {
        self.width = 0;
        self.height = 0;
        self.frame_count = 0;
        self.frame_ticks = 1;
        for frame in &mut self.frames {
            frame.fill(0);
        }
    }

    /// Sets how many ticks each animation frame is held (clamped to `>= 1`).
    pub fn set_frame_ticks(&mut self, ticks: u8) {
        self.frame_ticks = ticks.max(1);
    }

    /// Loads one 1-bit MSB-first packed mask frame into `frame_index`.
    ///
    /// If `frame_index >= self.frame_count`, `frame_count` grows to `frame_index + 1`.
    pub fn load_frame(
        &mut self,
        frame_index: u8,
        width: u16,
        height: u16,
        mask: &[u8],
    ) -> Result<(), SlotError> {
        let idx = frame_index as usize;
        if idx >= self.max_frames() {
            return Err(SlotError::InvalidFrameIndex(frame_index));
        }

        let expected = required_mask_bytes(width, height)?;
        if mask.len() != expected {
            return Err(SlotError::InvalidMaskLength {
                expected,
                actual: mask.len(),
            });
        }

        self.width = width;
        self.height = height;
        self.frames[idx][..expected].copy_from_slice(mask);
        if frame_index + 1 > self.frame_count {
            self.frame_count = frame_index + 1;
        }
        Ok(())
    }

    /// Returns the active [`Bitmap1Bit`] for animation tick `tick`, or `None` if empty.
    pub fn frame_at(&self, tick: u32) -> Option<Bitmap1Bit<'_>> {
        if self.frame_count == 0 || self.width == 0 || self.height == 0 {
            return None;
        }
        let ticks_per_frame = self.frame_ticks.max(1) as u32;
        let frame_idx = ((tick / ticks_per_frame) % (self.frame_count as u32)) as usize;
        let len = (self.width as usize).div_ceil(8) * (self.height as usize);
        Some(Bitmap1Bit {
            width: self.width as usize,
            height: self.height as usize,
            data: &self.frames[frame_idx][..len],
        })
    }

    /// Renders the frame for `tick` centered onto `fb` using `color`.
    pub fn render_into(&self, fb: &mut Framebuffer, tick: u32, color: Rgb) {
        if color == Rgb::BLACK {
            return;
        }
        let Some(bitmap) = self.frame_at(tick) else {
            return;
        };
        let x = ((fb.width() as i32) - (bitmap.width as i32)) / 2;
        let y = ((fb.height() as i32) - (bitmap.height as i32)) / 2;
        fb.draw_bitmap_1bit(x, y, bitmap, color);
    }
}

/// Default in-RAM bank of 1-bit animated face slots (implements [`FaceSource`]).
#[derive(Debug, Clone)]
pub struct SlotBank {
    slots: [FaceSlot; DEFAULT_SLOT_CAPACITY],
}

impl Default for SlotBank {
    fn default() -> Self {
        Self::new()
    }
}

impl SlotBank {
    /// Creates an empty slot bank (all slots blank until loaded).
    pub const fn new() -> Self {
        Self {
            slots: [const { FaceSlot::empty() }; DEFAULT_SLOT_CAPACITY],
        }
    }

    /// Number of slots available in this bank.
    pub const fn capacity(&self) -> usize {
        self.slots.len()
    }

    /// Returns a read-only reference to slot `slot_id`, if in range.
    pub fn get(&self, slot_id: u8) -> Option<&FaceSlot> {
        self.slots.get(slot_id as usize)
    }

    /// Returns a mutable reference to slot `slot_id`, if in range.
    pub fn get_mut(&mut self, slot_id: u8) -> Option<&mut FaceSlot> {
        self.slots.get_mut(slot_id as usize)
    }

    /// Loads one 1-bit frame into `slot_id` at `frame_index`.
    pub fn load_frame(
        &mut self,
        slot_id: u8,
        frame_index: u8,
        width: u16,
        height: u16,
        mask: &[u8],
    ) -> Result<(), SlotError> {
        let slot = self
            .slots
            .get_mut(slot_id as usize)
            .ok_or(SlotError::InvalidSlot(slot_id))?;
        slot.load_frame(frame_index, width, height, mask)
    }

    /// Configures animation speed (`ticks` per frame) for `slot_id`.
    pub fn set_frame_ticks(&mut self, slot_id: u8, ticks: u8) -> Result<(), SlotError> {
        let slot = self
            .slots
            .get_mut(slot_id as usize)
            .ok_or(SlotError::InvalidSlot(slot_id))?;
        slot.set_frame_ticks(ticks);
        Ok(())
    }

    /// Clears `slot_id` back to empty.
    pub fn clear_slot(&mut self, slot_id: u8) -> Result<(), SlotError> {
        let slot = self
            .slots
            .get_mut(slot_id as usize)
            .ok_or(SlotError::InvalidSlot(slot_id))?;
        slot.clear();
        Ok(())
    }
}

impl FaceSource for SlotBank {
    fn render_slot(&self, slot_id: u8, tick: u32, tint: Rgb, brightness: u8, fb: &mut Framebuffer) {
        if let Some(slot) = self.get(slot_id) {
            let color = tint.scale(brightness);
            slot.render_into(fb, tick, color);
        }
    }
}

fn required_mask_bytes(width: u16, height: u16) -> Result<usize, SlotError> {
    if width == 0 || height == 0 {
        return Err(SlotError::InvalidDimensions { width, height });
    }
    let bytes = (width as usize)
        .div_ceil(8)
        .checked_mul(height as usize)
        .ok_or(SlotError::InvalidDimensions { width, height })?;
    if bytes > DEFAULT_MASK_BYTES {
        return Err(SlotError::InvalidDimensions { width, height });
    }
    Ok(bytes)
}
