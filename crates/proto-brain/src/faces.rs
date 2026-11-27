//! Built-in starter Protogen visor expressions rendered symmetrically across both 64x32 panels.
//!
//! Coordinate convention on the left 64x32 half (`x = 0..64`, `y = 0..32`):
//! - `x = 0` is the outer/ear side of the left cheek.
//! - `x = 63` is the center nose seam where the two panels meet.
//!
//! Every drawing call uses mirrored primitives so the right panel (`x = 64..128`)
//! is an exact horizontal mirror of the left panel.

use crate::framebuffer::{Framebuffer, Rgb};

/// Built-in starter visor expressions shipped in v0.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum Expression {
    /// Calm default visor face: sleek eyes, nose dot, gentle smile.
    #[default]
    Neutral = 0,
    /// Curved `^ ^` happy eyes and bright upbeat smile.
    Happy = 1,
    /// Wide open eyes and small `o` surprised mouth.
    Surprised = 2,
}

impl Expression {
    /// Total number of built-in starter expressions.
    pub const COUNT: u8 = 3;

    /// Converts a wire expression ID into an `Expression`, falling back to `Neutral`.
    pub const fn from_u8(value: u8) -> Self {
        match value {
            1 => Self::Happy,
            2 => Self::Surprised,
            _ => Self::Neutral,
        }
    }

    /// Converts the expression into its wire ID.
    pub const fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Renders `expression` into `fb` using `color` (already scaled for brightness).
pub fn render_expression(fb: &mut Framebuffer, expression: Expression, color: Rgb) {
    fb.clear();
    if color == Rgb::BLACK {
        return;
    }

    match expression {
        Expression::Neutral => draw_neutral(fb, color),
        Expression::Happy => draw_happy(fb, color),
        Expression::Surprised => draw_surprised(fb, color),
    }
}

/// Small symmetric nose accent near the center seam (`x = 58..61`).
fn draw_nose(fb: &mut Framebuffer, color: Rgb) {
    fb.draw_mirrored_line(57, 15, 60, 17, 2, color);
}

fn draw_neutral(fb: &mut Framebuffer, color: Rgb) {
    // Sleek Protogen eye (outer upper-left cheek, x = 14..38, y = 6..15).
    fb.fill_mirrored_ellipse(26, 11, 10, 4, color);
    // Subtle flat bottom cut on the eye for a crisp visor silhouette.
    fb.fill_mirrored_rect(15, 14, 23, 3, Rgb::BLACK);

    draw_nose(fb, color);

    // Calm classic Protogen zigzag/wave mouth along the lower cheek.
    fb.draw_mirrored_line(10, 24, 22, 26, 2, color);
    fb.draw_mirrored_line(22, 26, 36, 23, 2, color);
    fb.draw_mirrored_line(36, 23, 50, 26, 2, color);
    fb.draw_mirrored_line(50, 26, 58, 24, 2, color);
}

fn draw_happy(fb: &mut Framebuffer, color: Rgb) {
    // Thick curved `^` happy eye on each cheek.
    fb.draw_mirrored_line(14, 14, 23, 7, 3, color);
    fb.draw_mirrored_line(23, 7, 30, 7, 3, color);
    fb.draw_mirrored_line(30, 7, 38, 14, 3, color);

    draw_nose(fb, color);

    // Wide cheerful smile rising toward the cheek.
    fb.draw_mirrored_line(8, 20, 18, 26, 2, color);
    fb.draw_mirrored_line(18, 26, 38, 27, 2, color);
    fb.draw_mirrored_line(38, 27, 56, 23, 2, color);
}

fn draw_surprised(fb: &mut Framebuffer, color: Rgb) {
    // Large hollow oval eye (outer bright ring, dark inner center).
    fb.fill_mirrored_ellipse(26, 11, 9, 7, color);
    fb.fill_mirrored_ellipse(26, 11, 5, 4, Rgb::BLACK);

    draw_nose(fb, color);

    // Small surprised mouth near the front-lower cheek.
    fb.fill_mirrored_ellipse(48, 25, 6, 4, color);
    fb.fill_mirrored_ellipse(48, 25, 3, 2, Rgb::BLACK);
}
