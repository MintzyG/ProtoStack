//! Integration tests for `ProtoBrain` state transitions and `ProtoSpeak` packet handling.

use proto_brain::{BrainError, Expression, ProtoBrain, Rgb, VISOR_HEIGHT, VISOR_WIDTH};
use proto_speak::{
    decode_packet, decode_payload, encode_payload, Flags, Header, HelloInfo, HelloRequest, McuKind,
    NodeId, Opcode, PreviewAck, PreviewState, MAX_PACKET_SIZE, PROTOCOL_VERSION,
};

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
    assert_eq!(info.visor_width, VISOR_WIDTH as u16);
    assert_eq!(info.visor_height, VISOR_HEIGHT as u16);
    assert_eq!(info.button_count, 1);
}

#[test]
fn handles_preview_state_clamping_and_virtual_button_override() {
    let mut brain = ProtoBrain::new(McuKind::WasmSimulator, 1);

    // Send Happy with brightness 150 (must clamp to 100) and virtual button 1 pressed.
    let preview = PreviewState {
        expression_id: Expression::Happy.as_u8(),
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

    // Base expression is Happy, but pressing virtual button 1 overrides to Surprised.
    assert_eq!(brain.base_expression(), Expression::Happy);
    assert_eq!(brain.active_expression(), Expression::Surprised);
    assert_eq!(ack.active_expression, Expression::Surprised.as_u8());
    assert_eq!(ack.effective_brightness, 100);
    assert_eq!(brain.tint(), Rgb::from_array([100, 255, 180]));

    // Releasing virtual button 1 returns cleanly to Happy.
    let released = PreviewState {
        virtual_button_mask: 0,
        ..preview
    };
    let ack2 = brain.apply_preview(released);
    assert_eq!(ack2.active_expression, Expression::Happy.as_u8());
    assert_eq!(brain.active_expression(), Expression::Happy);
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
