//! Tests for ProtoSpeak packet header, payload codec, and CRC validation.

use proto_speak::{
    decode_packet, decode_payload, encode_packet, encode_payload, Domain, Error, Flags, Header,
    HelloInfo, McuKind, NodeId, Opcode, PreviewState, CRC_LEN, HEADER_LEN, MAX_PACKET_SIZE,
    PROTOCOL_VERSION,
};

#[test]
fn header_round_trip_and_nibble_packing() {
    let header = Header {
        version: PROTOCOL_VERSION,
        domain: Domain::Control,
        flags: Flags::ACK_REQ.union(Flags::PRIORITY_SAFETY),
        opcode: Opcode::PREVIEW_STATE,
        src: NodeId::HOST,
        dst: NodeId::HEAD,
        seq: 0x1234,
        payload_len: 7,
    };

    let bytes = header.to_bytes();
    assert_eq!(bytes[0], 0x10); // version=1 (high nibble), domain=0 (low nibble)
    assert_eq!(bytes[1], 0x11); // ACK_REQ (0x01) | PRIORITY_SAFETY (0x10)
    assert_eq!(Header::from_bytes(&bytes), Ok(header));
}

#[test]
fn encode_and_decode_preview_state() {
    let msg = PreviewState {
        expression_id: 2,
        tint_rgb: [56, 189, 248],
        brightness: 75,
        virtual_button_mask: 0b0000_0001,
    };
    let header = Header::control(
        Opcode::PREVIEW_STATE,
        NodeId::HOST,
        NodeId::HEAD,
        42,
        Flags::ACK_REQ,
    );

    let mut buf = [0u8; MAX_PACKET_SIZE];
    let written = encode_payload(header, &msg, &mut buf).ok();
    assert!(written.is_some());
    let written = written.unwrap_or(0);

    let decoded = decode_packet(&buf[..written]);
    assert!(decoded.is_ok());
    if let Ok(pkt) = decoded {
        assert_eq!(pkt.header.opcode, Opcode::PREVIEW_STATE);
        assert_eq!(pkt.header.seq, 42);
        assert_eq!(pkt.header.src, NodeId::HOST);
        assert_eq!(pkt.header.dst, NodeId::HEAD);
        assert!(pkt.header.flags.contains(Flags::ACK_REQ));
        assert_eq!(decode_payload::<PreviewState>(pkt.payload), Ok(msg));
    }
}

#[test]
fn encode_and_decode_hello_info() {
    let info = HelloInfo {
        fw_major: 0,
        fw_minor: 1,
        fw_patch: 0,
        protocol_version: PROTOCOL_VERSION,
        mcu: McuKind::WasmSimulator,
        visor_width: 128,
        visor_height: 32,
        button_count: 1,
    };
    let header = Header::control(
        Opcode::HELLO_INFO,
        NodeId::HEAD,
        NodeId::HOST,
        1,
        Flags::IS_ACK,
    );

    let mut buf = [0u8; MAX_PACKET_SIZE];
    let len = encode_payload(header, &info, &mut buf).unwrap_or(0);
    assert!(len > HEADER_LEN + CRC_LEN);

    // Trailing zero bytes (like from a 64-byte HID chunk) must be tolerated.
    let decoded = decode_packet(&buf);
    assert!(decoded.is_ok());
    if let Ok(pkt) = decoded {
        assert_eq!(decode_payload::<HelloInfo>(pkt.payload), Ok(info));
    }
}

#[test]
fn rejects_corrupted_payload_or_crc() {
    let header = Header::control(
        Opcode::PREVIEW_STATE,
        NodeId::HOST,
        NodeId::HEAD,
        7,
        Flags::NONE,
    );
    let mut buf = [0u8; 64];
    let len = encode_packet(header, &[1, 2, 3, 4], &mut buf).unwrap_or(0);

    // Flip one bit inside the payload.
    buf[HEADER_LEN + 1] ^= 0x04;
    assert!(matches!(
        decode_packet(&buf[..len]),
        Err(Error::BadCrc { .. })
    ));
}

#[test]
fn rejects_wrong_version_and_truncated_input() {
    let mut buf = [0u8; 64];
    let header = Header::control(Opcode::HELLO_INFO, 0, 1, 1, Flags::NONE);
    let len = encode_packet(header, &[], &mut buf).unwrap_or(0);

    assert_eq!(decode_packet(&buf[..5]), Err(Error::TruncatedPacket));

    // Change version nibble from 1 to 2.
    buf[0] = 0x20;
    assert_eq!(
        decode_packet(&buf[..len]),
        Err(Error::UnsupportedVersion(2))
    );
}
