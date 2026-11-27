//! Tests for 64-byte WebHID chunking and reassembly.

use proto_speak::{
    decode_packet, encode_packet, Error, Flags, Header, HidChunker, HidReassembler, NodeId, Opcode,
    HID_REPORT_ID, HID_REPORT_SIZE, MAX_PACKET_SIZE,
};

#[test]
fn single_chunk_round_trip() {
    let header = Header::control(
        Opcode::HELLO_INFO,
        NodeId::HOST,
        NodeId::HEAD,
        3,
        Flags::NONE,
    );
    let mut raw = [0u8; 64];
    let len = encode_packet(header, &[0xAA, 0xBB], &mut raw).unwrap_or(0);

    let mut chunker = match HidChunker::new(&raw[..len]) {
        Ok(c) => c,
        Err(e) => panic!("unexpected error: {e:?}"),
    };
    assert_eq!(chunker.total_chunks(), 1);

    let report = match chunker.next() {
        Some(r) => r,
        None => panic!("expected one HID report"),
    };
    assert!(chunker.next().is_none());
    assert_eq!(report[0], HID_REPORT_ID);
    assert_eq!(report[1], 0);
    assert_eq!(report[2], 1);

    let mut reassembler = HidReassembler::<MAX_PACKET_SIZE>::new();
    let assembled = match reassembler.push(&report) {
        Ok(Some(bytes)) => bytes,
        other => panic!("expected complete packet, got {other:?}"),
    };

    let pkt = match decode_packet(assembled) {
        Ok(p) => p,
        Err(e) => panic!("decode failed: {e:?}"),
    };
    assert_eq!(pkt.header.seq, 3);
    assert_eq!(pkt.payload, &[0xAA, 0xBB]);
}

#[test]
fn multi_chunk_round_trip() {
    // 150-byte payload + 12 header + 2 CRC = 164 bytes -> 3 HID chunks (61 + 61 + 42).
    let payload = [0x5Au8; 150];
    let header = Header::control(
        Opcode::PREVIEW_STATE,
        NodeId::HOST,
        NodeId::HEAD,
        99,
        Flags::ACK_REQ,
    );
    let mut raw = [0u8; MAX_PACKET_SIZE];
    let len = encode_packet(header, &payload, &mut raw).unwrap_or(0);
    assert_eq!(len, 164);

    let chunker = match HidChunker::new(&raw[..len]) {
        Ok(c) => c,
        Err(e) => panic!("unexpected error: {e:?}"),
    };
    assert_eq!(chunker.total_chunks(), 3);

    let mut reassembler = HidReassembler::<MAX_PACKET_SIZE>::new();
    let mut finished_len = 0usize;
    let mut out_copy = [0u8; MAX_PACKET_SIZE];

    for (i, report) in chunker.enumerate() {
        let res = reassembler.push(&report);
        if i < 2 {
            assert_eq!(res, Ok(None));
        } else if let Ok(Some(bytes)) = res {
            finished_len = bytes.len();
            out_copy[..finished_len].copy_from_slice(bytes);
        } else {
            panic!("expected final chunk to complete packet, got {res:?}");
        }
    }

    let pkt = match decode_packet(&out_copy[..finished_len]) {
        Ok(p) => p,
        Err(e) => panic!("decode failed: {e:?}"),
    };
    assert_eq!(pkt.header.seq, 99);
    assert_eq!(pkt.payload, &payload);
}

#[test]
fn rejects_bad_report_id_and_out_of_order_chunks() {
    let mut reassembler = HidReassembler::<MAX_PACKET_SIZE>::new();

    let mut bad_id = [0u8; HID_REPORT_SIZE];
    bad_id[0] = 0x02;
    bad_id[1] = 0;
    bad_id[2] = 1;
    assert_eq!(reassembler.push(&bad_id), Err(Error::InvalidReportId(0x02)));

    let mut zero_total = [0u8; HID_REPORT_SIZE];
    zero_total[0] = HID_REPORT_ID;
    zero_total[1] = 0;
    zero_total[2] = 0;
    assert_eq!(
        reassembler.push(&zero_total),
        Err(Error::InvalidChunkHeader)
    );

    // Sending chunk 1 before chunk 0 fails as out-of-order.
    let mut ooo = [0u8; HID_REPORT_SIZE];
    ooo[0] = HID_REPORT_ID;
    ooo[1] = 1;
    ooo[2] = 2;
    assert_eq!(
        reassembler.push(&ooo),
        Err(Error::OutOfOrderChunk {
            expected: 0,
            actual: 1
        })
    );
}
