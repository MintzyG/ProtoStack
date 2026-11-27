//! Deterministic golden framebuffer tests for `ProtoBrain` visor rendering.
//!
//! Each starter expression is compared pixel-for-pixel against a human-readable
//! 64x32 left-half ASCII snapshot in `tests/snapshots/`, and verified to be
//! horizontally mirrored onto the right 64x32 half.
//!
//! To regenerate snapshots intentionally after editing `faces.rs`, run:
//! `UPDATE_GOLDENS=1 cargo test -p proto-brain --test golden`

use proto_brain::{
    render_expression, Expression, Framebuffer, Rgb, FRAMEBUFFER_RGBA_LEN, VISOR_HEIGHT,
    VISOR_WIDTH,
};

fn render_left_half_ascii(fb: &Framebuffer, expected_color: Rgb) -> String {
    let mut out = String::with_capacity((VISOR_WIDTH / 2 + 1) * VISOR_HEIGHT);
    for y in 0..VISOR_HEIGHT {
        for x in 0..(VISOR_WIDTH / 2) {
            let p = fb.get_pixel(x, y);
            if p == Rgb::BLACK {
                out.push('.');
            } else {
                assert_eq!(p, expected_color, "unexpected color at ({x}, {y})");
                out.push('#');
            }
        }
        out.push('\n');
    }
    out
}

fn assert_horizontally_symmetric(fb: &Framebuffer) {
    for y in 0..VISOR_HEIGHT {
        for x in 0..(VISOR_WIDTH / 2) {
            let left = fb.get_pixel(x, y);
            let right = fb.get_pixel(VISOR_WIDTH - 1 - x, y);
            assert_eq!(left, right, "asymmetry at ({x}, {y})");
        }
    }
}

fn verify_snapshot(expression: Expression, name: &str, expected: &str) {
    let color = Rgb::DEFAULT_TINT.scale(100);
    let mut fb = Framebuffer::new();
    render_expression(&mut fb, expression, color);

    assert_horizontally_symmetric(&fb);
    let actual = render_left_half_ascii(&fb, color);

    if std::env::var("UPDATE_GOLDENS").is_ok() {
        let path = format!("{}/tests/snapshots/{name}.txt", env!("CARGO_MANIFEST_DIR"));
        if let Err(err) = std::fs::write(&path, &actual) {
            panic!("failed to write golden snapshot {path}: {err}");
        }
        return;
    }

    assert_eq!(
        actual, expected,
        "golden mismatch for `{name}` (run UPDATE_GOLDENS=1 cargo test -p proto-brain --test golden if intentional)"
    );
}

#[test]
fn golden_neutral_face() {
    verify_snapshot(
        Expression::Neutral,
        "neutral",
        include_str!("snapshots/neutral.txt"),
    );
}

#[test]
fn golden_happy_face() {
    verify_snapshot(
        Expression::Happy,
        "happy",
        include_str!("snapshots/happy.txt"),
    );
}

#[test]
fn golden_surprised_face() {
    verify_snapshot(
        Expression::Surprised,
        "surprised",
        include_str!("snapshots/surprised.txt"),
    );
}

#[test]
fn zero_brightness_renders_completely_black() {
    let mut fb = Framebuffer::new();
    render_expression(&mut fb, Expression::Happy, Rgb::DEFAULT_TINT.scale(0));
    assert!(fb.as_rgb_bytes().iter().all(|&b| b == 0));
}

#[test]
fn brightness_scales_lit_pixels_and_rgba_copy_preserves_channels() {
    let tint = Rgb::from_array([200, 100, 50]);
    let expected = tint.scale(50);
    assert_eq!(expected, Rgb::from_array([100, 50, 25]));

    let mut fb = Framebuffer::new();
    render_expression(&mut fb, Expression::Neutral, expected);

    // Every lit pixel must carry the exact scaled tint color.
    let mut lit_count = 0usize;
    for y in 0..VISOR_HEIGHT {
        for x in 0..VISOR_WIDTH {
            let p = fb.get_pixel(x, y);
            if p != Rgb::BLACK {
                assert_eq!(p, expected);
                lit_count += 1;
            }
        }
    }
    assert!(lit_count > 100);

    // Verify RGBA export for HTML5 Canvas (`alpha == 255` everywhere).
    let mut rgba = [0u8; FRAMEBUFFER_RGBA_LEN];
    assert!(fb.copy_rgba_into(&mut rgba));
    for pixel in rgba.as_chunks::<4>().0 {
        assert_eq!(pixel[3], 255);
    }
}
