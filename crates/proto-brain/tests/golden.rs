//! Deterministic golden framebuffer tests for animated `ProtoBrain` visor expressions.
//!
//! Each keyframe is compared pixel-for-pixel against a human-readable 128x32
//! ASCII snapshot in `tests/snapshots/` (using community LumiFur_Controller
//! bitmaps; see `THIRD_PARTY_NOTICES.md`).
//!
//! To regenerate snapshots intentionally after editing `faces.rs`, run:
//! `UPDATE_GOLDENS=1 cargo test -p proto-brain --test golden`

use proto_brain::{blink_open_pct, render_expression, Expression, Framebuffer, Rgb};

fn render_full_ascii(fb: &Framebuffer, expected_color: Rgb) -> String {
    let mut out = String::with_capacity((fb.width() + 1) * fb.height());
    for y in 0..fb.height() {
        for x in 0..fb.width() {
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

fn verify_snapshot(expression: Expression, frame: u32, name: &str, expected: &str) {
    let color = Rgb::DEFAULT_TINT.scale(100);
    let mut fb = Framebuffer::new();
    render_expression(&mut fb, expression, frame, color);

    let actual = render_full_ascii(&fb, color);

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
fn golden_neutral_open_and_blink_frames() {
    assert_eq!(blink_open_pct(0), 100);
    assert_eq!(blink_open_pct(56), 0);

    verify_snapshot(
        Expression::Neutral,
        0,
        "neutral",
        include_str!("snapshots/neutral.txt"),
    );
    verify_snapshot(
        Expression::Neutral,
        56,
        "neutral_blink",
        include_str!("snapshots/neutral_blink.txt"),
    );
}

#[test]
fn golden_happy_closed_and_open_maw_frames() {
    verify_snapshot(
        Expression::Happy,
        0,
        "happy",
        include_str!("snapshots/happy.txt"),
    );
    verify_snapshot(
        Expression::Happy,
        10,
        "happy_open",
        include_str!("snapshots/happy_open.txt"),
    );
}

#[test]
fn golden_surprised_face() {
    verify_snapshot(
        Expression::Surprised,
        0,
        "surprised",
        include_str!("snapshots/surprised.txt"),
    );
}

#[test]
fn zero_brightness_renders_completely_black() {
    let mut fb = Framebuffer::new();
    render_expression(&mut fb, Expression::Happy, 0, Rgb::DEFAULT_TINT.scale(0));
    assert!(fb.as_rgb_bytes().iter().all(|&b| b == 0));
}

#[test]
fn brightness_scales_lit_pixels_and_rgba_copy_preserves_channels() {
    let tint = Rgb::from_array([200, 100, 50]);
    let expected = tint.scale(50);
    assert_eq!(expected, Rgb::from_array([100, 50, 25]));

    let mut fb = Framebuffer::new();
    render_expression(&mut fb, Expression::Neutral, 0, expected);

    let mut lit_count = 0usize;
    for y in 0..fb.height() {
        for x in 0..fb.width() {
            let p = fb.get_pixel(x, y);
            if p != Rgb::BLACK {
                assert_eq!(p, expected);
                lit_count += 1;
            }
        }
    }
    assert!(lit_count > 100);

    let mut rgba = vec![0u8; fb.rgba_len()];
    assert!(fb.copy_rgba_into(&mut rgba));
    for pixel in rgba.as_chunks::<4>().0 {
        assert_eq!(pixel[3], 255);
    }
}
