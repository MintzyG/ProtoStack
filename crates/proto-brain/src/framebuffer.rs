//! Logical `128x32` RGB framebuffer and simple 2D pixel drawing primitives.

/// Logical visor bounding-box width in pixels (two chained 64x32 HUB75 panels).
pub const VISOR_WIDTH: usize = 128;

/// Logical visor bounding-box height in pixels.
pub const VISOR_HEIGHT: usize = 32;

/// Number of bytes in the packed RGB framebuffer (`128 * 32 * 3`).
pub const FRAMEBUFFER_RGB_LEN: usize = VISOR_WIDTH * VISOR_HEIGHT * 3;

/// Number of bytes in an RGBA copy for HTML5 Canvas (`128 * 32 * 4`).
pub const FRAMEBUFFER_RGBA_LEN: usize = VISOR_WIDTH * VISOR_HEIGHT * 4;

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

/// Fixed-size `128x32` RGB framebuffer (no heap allocation).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Framebuffer {
    pixels: [u8; FRAMEBUFFER_RGB_LEN],
}

impl Default for Framebuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl Framebuffer {
    /// Creates a black (all-zero) `128x32` framebuffer.
    pub const fn new() -> Self {
        Self {
            pixels: [0u8; FRAMEBUFFER_RGB_LEN],
        }
    }

    /// Clears all pixels to black.
    pub fn clear(&mut self) {
        self.pixels.fill(0);
    }

    /// Returns the raw packed RGB slice (`[r, g, b, r, g, b, ...]`).
    pub fn as_rgb_bytes(&self) -> &[u8; FRAMEBUFFER_RGB_LEN] {
        &self.pixels
    }

    /// Copies the framebuffer into an RGBA buffer (`alpha = 255`) for browser canvas rendering.
    /// Returns `false` if `out.len() < FRAMEBUFFER_RGBA_LEN`.
    pub fn copy_rgba_into(&self, out: &mut [u8]) -> bool {
        if out.len() < FRAMEBUFFER_RGBA_LEN {
            return false;
        }
        for i in 0..(VISOR_WIDTH * VISOR_HEIGHT) {
            let src = i * 3;
            let dst = i * 4;
            out[dst] = self.pixels[src];
            out[dst + 1] = self.pixels[src + 1];
            out[dst + 2] = self.pixels[src + 2];
            out[dst + 3] = 255;
        }
        true
    }

    /// Sets one pixel if `(x, y)` is inside `0..128` x `0..32`. Out-of-bounds writes are ignored.
    pub fn set_pixel(&mut self, x: i32, y: i32, color: Rgb) {
        if x < 0 || y < 0 {
            return;
        }
        let (ux, uy) = (x as usize, y as usize);
        if ux >= VISOR_WIDTH || uy >= VISOR_HEIGHT {
            return;
        }
        let idx = (uy * VISOR_WIDTH + ux) * 3;
        self.pixels[idx] = color.r;
        self.pixels[idx + 1] = color.g;
        self.pixels[idx + 2] = color.b;
    }

    /// Reads one pixel. Returns `Rgb::BLACK` for out-of-bounds coordinates.
    pub fn get_pixel(&self, x: usize, y: usize) -> Rgb {
        if x >= VISOR_WIDTH || y >= VISOR_HEIGHT {
            return Rgb::BLACK;
        }
        let idx = (y * VISOR_WIDTH + x) * 3;
        Rgb {
            r: self.pixels[idx],
            g: self.pixels[idx + 1],
            b: self.pixels[idx + 2],
        }
    }

    /// Sets a pixel on the left panel `(x, y)` and mirrors it onto the right panel `(127 - x, y)`.
    pub fn set_mirrored_pixel(&mut self, x: i32, y: i32, color: Rgb) {
        self.set_pixel(x, y, color);
        self.set_pixel((VISOR_WIDTH as i32 - 1) - x, y, color);
    }

    /// Draws a filled rectangle mirrored across both left and right visor panels.
    pub fn fill_mirrored_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: Rgb) {
        for dy in 0..h {
            for dx in 0..w {
                self.set_mirrored_pixel(x + dx, y + dy, color);
            }
        }
    }

    /// Draws a thick line segment (square brush of size `thickness`) mirrored across both panels.
    pub fn draw_mirrored_line(
        &mut self,
        mut x0: i32,
        mut y0: i32,
        x1: i32,
        y1: i32,
        thickness: i32,
        color: Rgb,
    ) {
        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        loop {
            self.fill_mirrored_rect(x0, y0, thickness, thickness, color);
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    /// Draws a mirrored filled ellipse centered at `(cx, cy)` with radii `(rx, ry)`.
    pub fn fill_mirrored_ellipse(&mut self, cx: i32, cy: i32, rx: i32, ry: i32, color: Rgb) {
        if rx <= 0 || ry <= 0 {
            return;
        }
        let rx2 = rx * rx;
        let ry2 = ry * ry;
        let bound = rx2 * ry2;
        for dy in -ry..=ry {
            for dx in -rx..=rx {
                if dx * dx * ry2 + dy * dy * rx2 < bound {
                    self.set_mirrored_pixel(cx + dx, cy + dy, color);
                }
            }
        }
    }
}
