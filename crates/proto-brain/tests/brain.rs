//! Integration tests for `ProtoBrain` state transitions, `FaceSource`, and `ProtoSpeak` handling.

use proto_brain::{BrainError, FaceSource, Framebuffer, ProtoBrain, Rgb, SlotError};
use proto_speak::{
    decode_packet, decode_payload, encode_payload, Flags, Header, HelloInfo, HelloRequest, McuKind,
    NodeId, Opcode, PreviewAck, PreviewState, MAX_PACKET_SIZE, PROTOCOL_VERSION,
};

/// Example multi-color / high-slot-index backend proving `ProtoBrain` does not
/// assume 8 slots or 1-bit monochrome sprites.
struct MultiColorTestSource;

impl FaceSource for MultiColorTestSource {
    fn render_slot(
        &self,
        slot_id: u8,
        tick: u32,
        _tint: Rgb,
        brightness: u8,
        fb: &mut Framebuffer,
    ) {
        if slot_id == 42 {
            let x = (tick as i32) % (fb.width() as i32);
            fb.set_pixel(x, 0, Rgb::from_array([255, 0, 0]).scale(brightness));
            fb.set_pixel(x, 1, Rgb::from_array([0, 255, 0]).scale(brightness));
            fb.set_pixel(x, 2, Rgb::from_array([0, 0, 255]).scale(brightness));
        }
    }
}

#[test]
fn starts_empty_and_renders_black_without_prebaked_faces() {
    let brain = ProtoBrain::new(McuKind::WasmSimulator, 1);
    assert!(brain.source().get(0).is_some_and(|s| s.is_empty()));
    assert!(brain.framebuffer().as_rgb_bytes().iter().all(|&b| b == 0));
}

#[test]
fn supports_custom_multicolor_facesource_beyond_default_slot_capacity() {
    let mut brain = ProtoBrain::with_source(McuKind::WasmSimulator, 1, MultiColorTestSource);
    brain.apply_preview(PreviewState {
        expression_id: 42,
        tint_rgb: [255, 255, 255],
        brightness: 50,
        virtual_button_mask: 0,
    });
    brain.set_frame(7);

    assert_eq!(
        brain.framebuffer().get_pixel(7, 0),
        Rgb::from_array([128, 0, 0])
    );
    assert_eq!(
        brain.framebuffer().get_pixel(7, 1),
        Rgb::from_array([0, 128, 0])
    );
    assert_eq!(
        brain.framebuffer().get_pixel(7, 2),
        Rgb::from_array([0, 0, 128])
    );
}

#[test]
fn handles_hello_info_handshake() {
    let mut brain = ProtoBrain::new(McuKind::WasmSimulator, 1);

    let req_header = Header::control(
        Opcode::HELLO_INFO,
        NodeId::HOST,
        NodeId::HEAD,
        10,
        Flags::ACK_REQ,
    );
    let mut req_buf = [0u8; MAX_PACKET_SIZE];
    let req_len = encode_payload(req_header, &HelloRequest, &mut req_buf).unwrap_or(0);

    let mut resp_buf = [0u8; MAX_PACKET_SIZE];
    let resp_len = match brain.handle_packet(&req_buf[..req_len], &mut resp_buf) {
        Ok(n) => n,
        Err(e) => panic!("hello failed: {e:?}"),
    };

    let decoded = match decode_packet(&resp_buf[..resp_len]) {
        Ok(p) => p,
        Err(e) => panic!("decode failed: {e:?}"),
    };
    assert_eq!(decoded.header.opcode, Opcode::HELLO_INFO);
    assert_eq!(decoded.header.seq, 10);
    assert_eq!(decoded.header.src, NodeId::HEAD);
    assert_eq!(decoded.header.dst, NodeId::HOST);
    assert!(decoded.header.flags.contains(Flags::IS_ACK));

    let info: HelloInfo = match decode_payload(decoded.payload) {
        Ok(i) => i,
        Err(e) => panic!("payload decode failed: {e:?}"),
    };
    assert_eq!(info.protocol_version, PROTOCOL_VERSION);
    assert_eq!(info.mcu, McuKind::WasmSimulator);
    assert_eq!(info.visor_width, brain.visor_width());
    assert_eq!(info.visor_height, brain.visor_height());
    assert_eq!(info.button_count, 1);
}

#[test]
fn handles_preview_state_clamping_and_virtual_button_override() {
    let mut brain = ProtoBrain::new(McuKind::WasmSimulator, 1);
    brain.set_button_override_slot(2);

    // Select slot 1 with brightness 150 (clamps to 100) and virtual button 1 pressed.
    let preview = PreviewState {
        expression_id: 1,
        tint_rgb: [100, 255, 180],
        brightness: 150,
        virtual_button_mask: 1,
    };
    let req_header = Header::control(
        Opcode::PREVIEW_STATE,
        NodeId::HOST,
        NodeId::HEAD,
        21,
        Flags::ACK_REQ,
    );
    let mut req_buf = [0u8; MAX_PACKET_SIZE];
    let req_len = encode_payload(req_header, &preview, &mut req_buf).unwrap_or(0);

    let mut resp_buf = [0u8; MAX_PACKET_SIZE];
    let resp_len = match brain.handle_packet(&req_buf[..req_len], &mut resp_buf) {
        Ok(n) => n,
        Err(e) => panic!("preview failed: {e:?}"),
    };

    let decoded = match decode_packet(&resp_buf[..resp_len]) {
        Ok(p) => p,
        Err(e) => panic!("decode failed: {e:?}"),
    };
    let ack: PreviewAck = match decode_payload(decoded.payload) {
        Ok(a) => a,
        Err(e) => panic!("ack decode failed: {e:?}"),
    };

    // Base slot is 1, but holding virtual button 1 temporarily activates slot 2.
    assert_eq!(brain.base_slot(), 1);
    assert_eq!(brain.active_slot(), 2);
    assert_eq!(ack.active_expression, 2);
    assert_eq!(ack.effective_brightness, 100);
    assert_eq!(brain.tint(), Rgb::from_array([100, 255, 180]));

    // Releasing virtual button 1 returns cleanly to slot 1.
    let released = PreviewState {
        virtual_button_mask: 0,
        ..preview
    };
    let ack2 = brain.apply_preview(released);
    assert_eq!(ack2.active_expression, 1);
    assert_eq!(brain.active_slot(), 1);
}

#[test]
fn validates_slot_bounds_and_mask_dimensions() {
    let mut brain = ProtoBrain::new(McuKind::WasmSimulator, 1);
    let cap = brain.source().capacity() as u8;
    let max_frames = brain.source().get(0).map_or(0, |s| s.max_frames()) as u8;

    assert_eq!(
        brain.source_mut().load_frame(cap, 0, 8, 8, &[0u8; 8]),
        Err(SlotError::InvalidSlot(cap))
    );
    assert_eq!(
        brain
            .source_mut()
            .load_frame(0, max_frames, 8, 8, &[0u8; 8]),
        Err(SlotError::InvalidFrameIndex(max_frames))
    );
    assert_eq!(
        brain.source_mut().load_frame(0, 0, 0, 8, &[]),
        Err(SlotError::InvalidDimensions {
            width: 0,
            height: 8
        })
    );
    assert_eq!(
        brain.source_mut().load_frame(0, 0, 8, 8, &[0u8; 4]),
        Err(SlotError::InvalidMaskLength {
            expected: 8,
            actual: 4
        })
    );
}

#[test]
fn rejects_unsupported_opcode_and_bad_crc() {
    let mut brain = ProtoBrain::new(McuKind::WasmSimulator, 1);
    let mut req_buf = [0u8; 64];
    let mut resp_buf = [0u8; 64];

    let unknown_header = Header::control(0x9999, NodeId::HOST, NodeId::HEAD, 5, Flags::NONE);
    let len = encode_payload(unknown_header, &HelloRequest, &mut req_buf).unwrap_or(0);
    assert_eq!(
        brain.handle_packet(&req_buf[..len], &mut resp_buf),
        Err(BrainError::UnsupportedOpcode(0x9999))
    );

    // Flip a byte to break CRC.
    req_buf[2] ^= 0x01;
    assert!(matches!(
        brain.handle_packet(&req_buf[..len], &mut resp_buf),
        Err(BrainError::Protocol(proto_speak::Error::BadCrc { .. }))
    ));
}
