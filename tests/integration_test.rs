use moss_decoder::*;

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

#[test]
fn test_decoding_single_event() {
    //
    let event = fake_event_simple();

    let (packet, unprocessed_bytes) = decode_event(&event).unwrap();

    assert!(
        unprocessed_bytes.is_empty(),
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

    if let Ok((packet, _unprocessed_data)) = decode_event(&events) {
        moss_packets.push(packet);
    }

    let packet_count = moss_packets.len();

    println!("{packet_count}");

    for p in moss_packets {
        println!("{p:?}");
    }
}

#[test]
fn test_decoding_multiple_events() {
    let mut events = fake_multiple_events();

    let mut moss_packets: Vec<MossPacket> = Vec::new();

    while let Ok((packet, unprocessed_data)) = decode_event(&events) {
        moss_packets.push(packet);
        events = unprocessed_data;
    }

    let packet_count = moss_packets.len();

    println!("{packet_count}");

    for p in moss_packets {
        println!("{p:?}");
    }
}

#[test]
fn test_decoding_multiple_events_alt() {
    let events = fake_multiple_events();

    let (packets, unprocessed_data) = decode_multiple_events_alt(&events).unwrap();

    let packet_count = packets.len();

    println!("last trailer at idx: {unprocessed_data}");
    println!("{packet_count}");

    for p in packets {
        println!("{p:?}");
    }
}

#[test]
fn test_decoding_multiple_events_delimiter() {
    let mut events = fake_multiple_events();
    events.append(&mut vec![0xFA, 0xFA, 0xFA]);

    let mut moss_packets: Vec<MossPacket> = Vec::new();

    while let Ok((packet, unprocessed_data)) = decode_event(&events) {
        moss_packets.push(packet);
        events = unprocessed_data;
    }

    let packet_count = moss_packets.len();

    println!("{packet_count}");

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
    let (p, last_trailer_idx) = decode_multiple_events_alt(&f).unwrap();
    println!("Decoded in: {t:?}\n", t = time.elapsed());

    println!("Got: {packets}", packets = p.len());
    println!("Last trailer at index: {last_trailer_idx}");
}
