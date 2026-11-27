//! Integration tests for the `proto-wasm` 64-byte HID loopback bridge.

use proto_speak::{McuKind, PreviewState, HID_REPORT_SIZE, PROTOCOL_VERSION};
use proto_wasm::{
    decode_hello_hid_reports_raw, decode_preview_ack_hid_reports_raw, encode_hello_hid_reports_raw,
    encode_preview_hid_reports_raw, WasmHelmet,
};

#[test]
fn hello_and_preview_round_trip_over_64_byte_hid_reports() {
    let mut helmet = WasmHelmet::new(1);

    // 1. Handshake over real 64-byte HID reports.
    let hello_req = match encode_hello_hid_reports_raw(1) {
        Ok(bytes) => bytes,
        Err(e) => panic!("encode hello failed: {e:?}"),
    };
    assert_eq!(hello_req.len(), HID_REPORT_SIZE);

    let hello_resp = match helmet.push_hid_report_raw(&hello_req) {
        Ok(bytes) => bytes,
        Err(e) => panic!("push hello failed: {e:?}"),
    };
    let info = match decode_hello_hid_reports_raw(&hello_resp) {
        Ok(i) => i,
        Err(e) => panic!("decode hello failed: {e:?}"),
    };
    assert_eq!(info.protocol_version, PROTOCOL_VERSION);
    assert_eq!(info.mcu, McuKind::WasmSimulator);
    assert_eq!(info.visor_width, helmet.visor_width());
    assert_eq!(info.visor_height, helmet.visor_height());
    assert_eq!(info.button_count, 1);

    // 2. Load a 1-bit pixel into slot 0 and preview it with cyan tint at 100% brightness.
    let mask = [0x80u8; 1]; // 1x1 lit pixel
    assert!(helmet.load_slot_frame_raw(0, 0, 1, 1, &mask).is_ok());

    let preview_req = match encode_preview_hid_reports_raw(
        2,
        PreviewState {
            expression_id: 0,
            tint_rgb: [56, 189, 248],
            brightness: 100,
            virtual_button_mask: 0,
        },
    ) {
        Ok(bytes) => bytes,
        Err(e) => panic!("encode preview failed: {e:?}"),
    };
    assert_eq!(preview_req.len(), HID_REPORT_SIZE);

    let preview_resp = match helmet.push_hid_report_raw(&preview_req) {
        Ok(bytes) => bytes,
        Err(e) => panic!("push preview failed: {e:?}"),
    };
    let ack = match decode_preview_ack_hid_reports_raw(&preview_resp) {
        Ok(a) => a,
        Err(e) => panic!("decode preview ack failed: {e:?}"),
    };
    assert_eq!(ack.active_expression, 0);
    assert_eq!(ack.effective_brightness, 100);

    // Confirm rendered RGBA contains lit cyan pixels and full alpha.
    let rgba = helmet.render_rgba();
    let expected_len = (helmet.visor_width() as usize) * (helmet.visor_height() as usize) * 4;
    assert_eq!(rgba.len(), expected_len);
    let has_cyan = rgba.as_chunks::<4>().0.contains(&[56, 189, 248, 255]);
    assert!(has_cyan);
}
