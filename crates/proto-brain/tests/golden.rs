//! Golden framebuffer tests using community LumiFur_Controller Protogen visor fixtures.
//!
//! `proto-brain` itself contains zero hardcoded faces. These tests load real
//! community animated Protogen designs (`tests/snapshots/*.txt`, adapted from
//! Stefan Ritchie's LumiFur_Controller under BSD 3-Clause; see
//! `THIRD_PARTY_NOTICES.md`) into `SlotBank` and verify animation playback,
//! pixel-exact rendering, tinting, brightness scaling, and RGBA export.

use proto_brain::{Framebuffer, ProtoBrain, Rgb};
use proto_speak::{McuKind, PreviewState};

/// Packs a readable `#`/`.` ASCII snapshot into an MSB-first 1-bit mask.
fn pack_ascii_mask(ascii: &str) -> (u16, u16, Vec<u8>) {
    let lines: Vec<&str> = ascii.lines().filter(|l| !l.is_empty()).collect();
    let height = lines.len();
    let width = lines.first().map_or(0, |l| l.len());
    let row_bytes = width.div_ceil(8);
    let mut mask = vec![0u8; row_bytes * height];

    for (y, line) in lines.iter().enumerate() {
        assert_eq!(
            line.len(),
            width,
            "inconsistent snapshot row width at row {y}"
        );
        for (x, ch) in line.bytes().enumerate() {
            if ch == b'#' {
                mask[y * row_bytes + (x / 8)] |= 0x80 >> (x % 8);
            }
        }
    }

    (width as u16, height as u16, mask)
}

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

fn load_ascii_frame(brain: &mut ProtoBrain, slot_id: u8, frame_idx: u8, ascii: &str) {
    let (w, h, mask) = pack_ascii_mask(ascii);
    if let Err(err) = brain
        .slots_mut()
        .load_frame(slot_id, frame_idx, w, h, &mask)
    {
        panic!("failed to load test fixture into slot {slot_id} frame {frame_idx}: {err:?}");
    }
}

#[test]
fn animates_loaded_community_neutral_and_happy_slots() {
    let neutral_open = include_str!("snapshots/neutral.txt");
    let neutral_blink = include_str!("snapshots/neutral_blink.txt");
    let happy_closed = include_str!("snapshots/happy.txt");
    let happy_open = include_str!("snapshots/happy_open.txt");
    let surprised = include_str!("snapshots/surprised.txt");

    let mut brain = ProtoBrain::new(McuKind::WasmSimulator, 1);

    // Slot 0: Neutral (open + blink frames, 10 ticks per frame).
    load_ascii_frame(&mut brain, 0, 0, neutral_open);
    load_ascii_frame(&mut brain, 0, 1, neutral_blink);
    assert!(brain.slots_mut().set_frame_ticks(0, 10).is_ok());

    // Slot 1: Happy (closed + open maw frames, 5 ticks per frame).
    load_ascii_frame(&mut brain, 1, 0, happy_closed);
    load_ascii_frame(&mut brain, 1, 1, happy_open);
    assert!(brain.slots_mut().set_frame_ticks(1, 5).is_ok());

    // Slot 2: Surprised (single frame).
    load_ascii_frame(&mut brain, 2, 0, surprised);

    let tint = Rgb::DEFAULT_TINT;
    let color = tint.scale(100);

    // Preview Slot 0 at tick 0 -> neutral_open; tick 10 -> neutral_blink; tick 20 -> loops back.
    brain.apply_preview(PreviewState {
        expression_id: 0,
        tint_rgb: tint.to_array(),
        brightness: 100,
        virtual_button_mask: 0,
    });
    assert_eq!(render_full_ascii(brain.framebuffer(), color), neutral_open);

    brain.set_frame(10);
    assert_eq!(render_full_ascii(brain.framebuffer(), color), neutral_blink);

    brain.set_frame(20);
    assert_eq!(render_full_ascii(brain.framebuffer(), color), neutral_open);

    // Switch to Slot 1 (Happy) and verify closed -> open maw animation.
    brain.set_frame(0);
    brain.apply_preview(PreviewState {
        expression_id: 1,
        tint_rgb: tint.to_array(),
        brightness: 100,
        virtual_button_mask: 0,
    });
    assert_eq!(render_full_ascii(brain.framebuffer(), color), happy_closed);

    for _ in 0..5 {
        brain.tick();
    }
    assert_eq!(render_full_ascii(brain.framebuffer(), color), happy_open);

    // Hold virtual button 1 (configured override slot 2) -> renders Surprised.
    brain.set_button_override_slot(2);
    brain.apply_preview(PreviewState {
        expression_id: 1,
        tint_rgb: tint.to_array(),
        brightness: 100,
        virtual_button_mask: 1,
    });
    assert_eq!(render_full_ascii(brain.framebuffer(), color), surprised);
}

#[test]
fn zero_brightness_and_cleared_slot_render_black() {
    let mut brain = ProtoBrain::new(McuKind::WasmSimulator, 1);
    load_ascii_frame(&mut brain, 0, 0, include_str!("snapshots/happy.txt"));

    brain.apply_preview(PreviewState {
        expression_id: 0,
        tint_rgb: Rgb::DEFAULT_TINT.to_array(),
        brightness: 0,
        virtual_button_mask: 0,
    });
    assert!(brain.framebuffer().as_rgb_bytes().iter().all(|&b| b == 0));

    assert!(brain.slots_mut().clear_slot(0).is_ok());
    brain.apply_preview(PreviewState {
        expression_id: 0,
        tint_rgb: Rgb::DEFAULT_TINT.to_array(),
        brightness: 100,
        virtual_button_mask: 0,
    });
    assert!(brain.framebuffer().as_rgb_bytes().iter().all(|&b| b == 0));
}

#[test]
fn brightness_scales_lit_pixels_and_rgba_copy_preserves_channels() {
    let mut brain = ProtoBrain::new(McuKind::WasmSimulator, 1);
    load_ascii_frame(&mut brain, 0, 0, include_str!("snapshots/neutral.txt"));

    let tint = Rgb::from_array([200, 100, 50]);
    let expected = tint.scale(50);
    assert_eq!(expected, Rgb::from_array([100, 50, 25]));

    brain.apply_preview(PreviewState {
        expression_id: 0,
        tint_rgb: tint.to_array(),
        brightness: 50,
        virtual_button_mask: 0,
    });

    let fb = brain.framebuffer();
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
