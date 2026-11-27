//! Logical RGB framebuffer and 1-bit bitmap blitting primitives.
//!
//! v0.1 allocates a default `128x32` backing buffer, while exposing dynamic
//! `width()`, `height()`, and slice APIs so callers do not assume `128x32` is
//! the only or maximum resolution supported later.

/// Default logical visor width in pixels (two chained 64x32 HUB75 panels).
pub const DEFAULT_VISOR_WIDTH: usize = 128;

/// Default logical visor height in pixels.
pub const DEFAULT_VISOR_HEIGHT: usize = 32;

/// Packed RGB byte count for the default `128x32` visor (`128 * 32 * 3`).
pub const DEFAULT_FRAMEBUFFER_RGB_LEN: usize = DEFAULT_VISOR_WIDTH * DEFAULT_VISOR_HEIGHT * 3;

/// Packed RGBA byte count for the default `128x32` visor (`128 * 32 * 4`).
pub const DEFAULT_FRAMEBUFFER_RGBA_LEN: usize = DEFAULT_VISOR_WIDTH * DEFAULT_VISOR_HEIGHT * 4;

/// 24-bit RGB color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rgb {
    /// Red channel (0..=255).
    pub r: u8,
    /// Green channel (0..=255).
    pub g: u8,
    /// Blue channel (0..=255).
    pub b: u8,
}

impl Rgb {
    /// Black (LED off).
    pub const BLACK: Self = Self { r: 0, g: 0, b: 0 };

    /// Default ProtoStack cyan tint (`#38bdf8`).
    pub const DEFAULT_TINT: Self = Self {
        r: 56,
        g: 189,
        b: 248,
    };

    /// Creates an RGB color from `[r, g, b]`.
    pub const fn from_array(rgb: [u8; 3]) -> Self {
        Self {
            r: rgb[0],
            g: rgb[1],
            b: rgb[2],
        }
    }

    /// Converts the color into `[r, g, b]`.
    pub const fn to_array(self) -> [u8; 3] {
        [self.r, self.g, self.b]
    }

    /// Scales each channel by `brightness_pct` (clamped to `0..=100`).
    pub const fn scale(self, brightness_pct: u8) -> Self {
        let pct = if brightness_pct > 100 {
            100
        } else {
            brightness_pct
        } as u16;
        Self {
            r: ((self.r as u16 * pct + 50) / 100) as u8,
            g: ((self.g as u16 * pct + 50) / 100) as u8,
            b: ((self.b as u16 * pct + 50) / 100) as u8,
        }
    }
}

/// Borrowed 1-bit MSB-first packed bitmap (`row_stride = width.div_ceil(8)`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bitmap1Bit<'a> {
    /// Width in pixels.
    pub width: usize,
    /// Height in pixels.
    pub height: usize,
    /// Packed MSB-first pixel bytes.
    pub data: &'a [u8],
}

/// Fixed-storage RGB framebuffer (no heap allocation).
///
/// Currently backed by `DEFAULT_VISOR_WIDTH x DEFAULT_VISOR_HEIGHT`; callers
/// should read `width()`, `height()`, and `as_rgb_bytes()` instead of assuming
/// fixed dimensions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Framebuffer {
    pixels: [u8; DEFAULT_FRAMEBUFFER_RGB_LEN],
}

impl Default for Framebuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl Framebuffer {
    /// Creates a black (all-zero) framebuffer at the default `128x32` resolution.
    pub const fn new() -> Self {
        Self {
            pixels: [0u8; DEFAULT_FRAMEBUFFER_RGB_LEN],
        }
    }

    /// Logical framebuffer width in pixels.
    pub const fn width(&self) -> usize {
        DEFAULT_VISOR_WIDTH
    }

    /// Logical framebuffer height in pixels.
    pub const fn height(&self) -> usize {
        DEFAULT_VISOR_HEIGHT
    }

    /// Byte length of the packed RGB buffer (`width * height * 3`).
    pub const fn rgb_len(&self) -> usize {
        self.width() * self.height() * 3
    }

    /// Byte length required for an RGBA copy (`width * height * 4`).
    pub const fn rgba_len(&self) -> usize {
        self.width() * self.height() * 4
    }

    /// Clears all pixels to black.
    pub fn clear(&mut self) {
        self.pixels.fill(0);
    }

    /// Returns the raw packed RGB slice (`[r, g, b, r, g, b, ...]`).
    pub fn as_rgb_bytes(&self) -> &[u8] {
        &self.pixels
    }

    /// Copies the framebuffer into an RGBA buffer (`alpha = 255`) for browser canvas rendering.
    /// Returns `false` if `out.len() < self.rgba_len()`.
    pub fn copy_rgba_into(&self, out: &mut [u8]) -> bool {
        if out.len() < self.rgba_len() {
            return false;
        }
        let pixel_count = self.width() * self.height();
        for i in 0..pixel_count {
            let src = i * 3;
            let dst = i * 4;
            out[dst] = self.pixels[src];
            out[dst + 1] = self.pixels[src + 1];
            out[dst + 2] = self.pixels[src + 2];
            out[dst + 3] = 255;
        }
        true
    }

    /// Sets one pixel if `(x, y)` is inside `0..width` x `0..height`. Out-of-bounds writes are ignored.
    pub fn set_pixel(&mut self, x: i32, y: i32, color: Rgb) {
        if x < 0 || y < 0 {
            return;
        }
        let (ux, uy) = (x as usize, y as usize);
        if ux >= self.width() || uy >= self.height() {
            return;
        }
        let idx = (uy * self.width() + ux) * 3;
        self.pixels[idx] = color.r;
        self.pixels[idx + 1] = color.g;
        self.pixels[idx + 2] = color.b;
    }

    /// Reads one pixel. Returns `Rgb::BLACK` for out-of-bounds coordinates.
    pub fn get_pixel(&self, x: usize, y: usize) -> Rgb {
        if x >= self.width() || y >= self.height() {
            return Rgb::BLACK;
        }
        let idx = (y * self.width() + x) * 3;
        Rgb {
            r: self.pixels[idx],
            g: self.pixels[idx + 1],
            b: self.pixels[idx + 2],
        }
    }

    /// Draws an MSB-first 1-bit packed bitmap at `(x, y)`.
    pub fn draw_bitmap_1bit(&mut self, x: i32, y: i32, bitmap: Bitmap1Bit<'_>, color: Rgb) {
        let row_bytes = bitmap.width.div_ceil(8);
        for row in 0..bitmap.height {
            for col in 0..bitmap.width {
                let byte_idx = row * row_bytes + (col / 8);
                let Some(&byte) = bitmap.data.get(byte_idx) else {
                    continue;
                };
                if (byte & (0x80 >> (col % 8))) != 0 {
                    self.set_pixel(x + col as i32, y + row as i32, color);
                }
            }
        }
    }

    /// Draws a 1-bit bitmap squashed vertically toward its center row for blinking.
    ///
    /// `open_pct` is `100` for fully open and `0` for closed to a 2-pixel horizontal slit.
    pub fn draw_bitmap_1bit_blink(
        &mut self,
        x: i32,
        y: i32,
        bitmap: Bitmap1Bit<'_>,
        color: Rgb,
        open_pct: u8,
    ) {
        if open_pct >= 100 || bitmap.height <= 2 {
            self.draw_bitmap_1bit(x, y, bitmap, color);
            return;
        }

        let target_h = ((bitmap.height * open_pct as usize) / 100).max(2);
        let y_offset = ((bitmap.height - target_h) / 2) as i32;
        let row_bytes = bitmap.width.div_ceil(8);

        for dst_row in 0..target_h {
            let src_row = (dst_row * bitmap.height) / target_h;
            for col in 0..bitmap.width {
                let byte_idx = src_row * row_bytes + (col / 8);
                let Some(&byte) = bitmap.data.get(byte_idx) else {
                    continue;
                };
                if (byte & (0x80 >> (col % 8))) != 0 {
                    self.set_pixel(x + col as i32, y + y_offset + dst_row as i32, color);
                }
            }
        }
    }
}
