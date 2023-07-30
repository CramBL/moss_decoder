use moss_decoder::*;

use pretty_assertions::assert_eq;

const IDLE: u8 = 0xFF;
const UNIT_FRAME_TRAILER: u8 = 0xE0;
const UNIT_FRAME_HEADER_0: u8 = 0xD0;
const REGION_HEADER_0: u8 = 0xC0;
const REGION_HEADER_1: u8 = 0xC1;
const REGION_HEADER_2: u8 = 0xC2;
const REGION_HEADER_3: u8 = 0xC3;

fn fake_event_simple() -> Vec<u8> {
    vec![
        UNIT_FRAME_HEADER_0,
        REGION_HEADER_0,
        // Hit row 2, col 8
        0x00,
        0x50,
        0x88,
        IDLE,
        0x01,
        0x50,
        0x88,
        REGION_HEADER_1,
        // Hit row 301, col 433
        0x25,
        0x6E,
        0xB1,
        REGION_HEADER_2,
        REGION_HEADER_3,
        // Hit row 2, col 8
        0x00,
        0x50,
        0x88,
        UNIT_FRAME_TRAILER,
    ]
}

fn fake_multiple_events() -> Vec<u8> {
    vec![
        UNIT_FRAME_HEADER_0,
        IDLE,
        IDLE,
        REGION_HEADER_0,
        // Hit row 2, col 8
        0x00,
        0x50,
        0x88,
        REGION_HEADER_1,
        // Hit row 301, col 433
        0x25,
        0x6E,
        0xB1,
        REGION_HEADER_2,
        REGION_HEADER_3,
        // Hit row 2, col 8
        0x00,
        0x50,
        0x88,
        UNIT_FRAME_TRAILER,
        0xD1, // Unit 1, otherwise identical event
        IDLE,
        IDLE,
        REGION_HEADER_0,
        // Hit row 2, col 8
        0x00,
        0x50,
        0x88,
        REGION_HEADER_1,
        // Hit row 301, col 433
        0x25,
        0x6E,
        0xB1,
        REGION_HEADER_2,
        REGION_HEADER_3,
        // Hit row 2, col 8
        0x00,
        0x50,
        0x88,
        UNIT_FRAME_TRAILER,
        0xD2, // Unit 2, empty
        REGION_HEADER_0,
        REGION_HEADER_1,
        REGION_HEADER_2,
        IDLE,
        REGION_HEADER_3,
        UNIT_FRAME_TRAILER,
        0xD3, // Unit 3, simple hits
        REGION_HEADER_0,
        0x00,
        0b0100_0000, // row 0
        0b1000_0000, // col 0
        REGION_HEADER_1,
        0x00,
        0b0100_1000, // row 1
        0b1000_0001, // col 1
        REGION_HEADER_2,
        0x00,
        0b0101_0000, // row 2
        0b1000_0010, // col 2
        REGION_HEADER_3,
        0x00,
        0b0101_1000, // row 3
        0b1000_0011, // col 3
        IDLE,
        UNIT_FRAME_TRAILER,
    ]
}

fn fake_event_protocol_error() -> Vec<u8> {
    vec![
        UNIT_FRAME_HEADER_0,
        IDLE,
        IDLE,
        REGION_HEADER_0,
        // Hit row 2, col 8
        0x00,
        0xF0, // Protocol error
        0x88,
        REGION_HEADER_1,
        // Hit row 301, col 433
        0x25,
        0x6E,
        0xB1,
        REGION_HEADER_2,
        REGION_HEADER_3,
        // Hit row 2, col 8
        0x00,
        0x50,
        0x88,
        UNIT_FRAME_TRAILER,
    ]
}

#[test]
fn test_decoding_single_event() {
    //
    let event = fake_event_simple();

    let (packet, last_trailer_idx) = decode_event(&event).unwrap();

    assert!(
        last_trailer_idx == event.len() - 1,
        "All bytes were not processed!"
    );

    assert_eq!(
        packet,
        MossPacket {
            unit_id: 0,
            hits: vec![
                MossHit {
                    region: 0,
                    row: 2,
                    column: 8
                },
                MossHit {
                    region: 0,
                    row: 10,
                    column: 8
                },
                MossHit {
                    region: 1,
                    row: 301,
                    column: 433
                },
                MossHit {
                    region: 3,
                    row: 2,
                    column: 8
                },
            ]
        },
        "unexpected decoding result"
    );
}

#[test]
fn test_decoding_single_event_fsm() {
    //
    let event = fake_event_simple();

    let (packet, last_trailer_idx) = decode_event_fsm(&event).unwrap();

    assert!(
        last_trailer_idx == event.len() - 1,
        "All bytes were not processed!"
    );

    assert_eq!(
        packet,
        MossPacket {
            unit_id: 0,
            hits: vec![
                MossHit {
                    region: 0,
                    row: 2,
                    column: 8
                },
                MossHit {
                    region: 0,
                    row: 10,
                    column: 8
                },
                MossHit {
                    region: 1,
                    row: 301,
                    column: 433
                },
                MossHit {
                    region: 3,
                    row: 2,
                    column: 8
                },
            ]
        },
        "unexpected decoding result"
    );
}

#[test]
fn test_decoding_multiple_events_one_call() {
    let events = fake_multiple_events();

    let mut moss_packets: Vec<MossPacket> = Vec::new();

    // There's multiple events in the data but we only call decode_event once so we should only get one packet
    if let Ok((packet, _unprocessed_data)) = decode_event(&events) {
        moss_packets.push(packet);
    }

    let packet_count = moss_packets.len();

    assert_eq!(packet_count, 1, "Expected 1 packet, got {}", packet_count);

    for p in moss_packets {
        println!("{p:?}");
    }
}

#[test]
fn test_read_file_decode() {
    let time = std::time::Instant::now();

    println!("Reading file...");
    let f = std::fs::read(std::path::PathBuf::from("tests/moss_noise.raw")).unwrap();
    println!(
        "Read file in: {t:?}. Bytes: {cnt}",
        t = time.elapsed(),
        cnt = f.len()
    );

    println!("Decoding content...");
    let (p, last_trailer_idx) = decode_multiple_events(&f).unwrap();
    println!("Decoded in: {t:?}\n", t = time.elapsed());

    println!("Got: {packets} packets", packets = p.len());
    println!("Last trailer at index: {last_trailer_idx}");

    assert_eq!(
        last_trailer_idx,
        f.len() - 2,
        "All bytes were not processed!"
    );
    assert_eq!(p.len(), 100000, "Expected 100k packets, got {}", p.len());

    println!("{:#X?}", f.get(..=50));
}

#[test]
fn test_decode_from_file() {
    let time = std::time::Instant::now();
    let expect_packets = 100000;
    let expect_hits = 2716940;

    let packets =
        moss_decoder::decode_from_file("tests/moss_noise.raw".to_string().into()).unwrap();
    println!("Decoded in: {t:?}\n", t = time.elapsed());

    println!("Got: {packets}", packets = packets.len());

    assert_eq!(
        packets.len(),
        expect_packets,
        "Expected {expect_packets} packets, got {}",
        packets.len()
    );

    // Count total hits
    let total_hits = packets.iter().fold(0, |acc, p| acc + p.hits.len());
    assert_eq!(
        total_hits, expect_hits,
        "Expected {expect_hits} hits, got {total_hits}",
    );
}

#[test]
fn test_decode_protocol_error() {
    pyo3::prepare_freethreaded_python();

    let event = fake_event_protocol_error();

    match decode_event(&event) {
        Ok(_) => {
            panic!("This packet has a protocol error, but it was not detected!")
        }
        Err(e) if e.to_string().contains("Protocol error") => {
            println!("Got expected error: {e}");
        }
        Err(e) => {
            panic!("Got unexpected error: {e}");
        }
    }
}

#[test]
fn test_decode_from_file_func_fsm() {
    let expect_packets = 100000;
    let expect_hits = 2716940;

    println!("Reading file...");
    let time = std::time::Instant::now();

    let f = std::fs::read(std::path::PathBuf::from("tests/moss_noise.raw")).unwrap();
    println!(
        "Read file in: {t:?}. Bytes: {cnt}",
        t = time.elapsed(),
        cnt = f.len()
    );

    println!("Decoding content...");
    let (p, last_trailer_idx) = decode_multiple_events_fsm(&f).unwrap();
    println!("Decoded in: {t:?}\n", t = time.elapsed());

    println!("Got: {packets} packets", packets = p.len());
    println!("Last trailer at index: {last_trailer_idx}");

    assert_eq!(
        last_trailer_idx,
        f.len() - 2,
        "All bytes were not processed!"
    );
    assert_eq!(
        p.len(),
        expect_packets,
        "Expected 100k packets, got {}",
        p.len()
    );

    // Count total hits
    let total_hits = p.iter().fold(0, |acc, p| acc + p.hits.len());
    assert_eq!(
        total_hits, expect_hits,
        "Expected {expect_hits} hits, got {total_hits}",
    );
}

#[test]
fn test_decode_from_file_fsm() {
    let time = std::time::Instant::now();
    let expect_packets = 100000;
    let expect_hits = 2716940;

    let packets =
        moss_decoder::decode_from_file_fsm("tests/moss_noise.raw".to_string().into()).unwrap();
    println!("Decoded in: {t:?}\n", t = time.elapsed());

    println!("Got: {packets}", packets = packets.len());

    assert_eq!(
        packets.len(),
        expect_packets,
        "Expected {expect_packets} packets, got {}",
        packets.len()
    );

    // Count total hits
    let total_hits = packets.iter().fold(0, |acc, p| acc + p.hits.len());
    assert_eq!(
        total_hits, expect_hits,
        "Expected {expect_hits} hits, got {total_hits}",
    );
}
