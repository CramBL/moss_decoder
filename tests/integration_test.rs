use crate::moss_protocol::test_util::*;
use moss_decoder::*;

use pretty_assertions::assert_eq;

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
            unit_id: 1,
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

    let (packet, last_trailer_idx) = decode_event(&event).unwrap();

    assert!(
        last_trailer_idx == event.len() - 1,
        "All bytes were not processed!"
    );

    assert_eq!(
        packet,
        MossPacket {
            unit_id: 1,
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

    for p in moss_packets {
        println!("{p:?}");
    }

    assert_eq!(packet_count, 1, "Expected 1 packet, got {}", packet_count);
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
        Err(e) if e.to_string().contains("Decoding failed") => {
            println!("Got expected error: {e}");
        }
        Err(e) => {
            panic!("Got unexpected error: {e}");
        }
    }
}

#[test]
fn test_decode_multiple_events_fsm() {
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
    let (p, last_trailer_idx) = decode_multiple_events(&f).unwrap();
    println!("Decoded in: {t:?}\n", t = time.elapsed());

    println!("Got: {packets} packets", packets = p.len());
    println!("Last trailer at index: {last_trailer_idx}");
    println!("Last 10 bytes of file: {:X?}", f.get(f.len() - 10..));

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
fn test_decode_protocol_error_fsm() {
    pyo3::prepare_freethreaded_python();

    let event = fake_event_protocol_error();

    match decode_event(&event) {
        Ok(_) => {
            panic!("This packet has a protocol error, but it was not detected!")
        }
        Err(e) if e.to_string().contains("Decoding failed") => {
            println!("Got expected error: {e}");
        }
        Err(e) => {
            panic!("Got unexpected error: {e}");
        }
    }
}

#[test]
fn test_decode_events_skip_0_take_10() {
    let take = 10;
    let f = std::fs::read(std::path::PathBuf::from("tests/moss_noise.raw")).unwrap();
    let (p, last_trailer_idx) = decode_events_take_n(&f, take, None).unwrap();

    println!("Got: {packets} packets", packets = p.len());
    println!("Last trailer at index: {last_trailer_idx}");
    assert_eq!(p.len(), take, "Expected {take} packets, got {}", p.len());
}

#[test]
fn test_decode_events_skip_10_take_1() {
    let skip = 10;
    let take = 1;
    let f = std::fs::read(std::path::PathBuf::from("tests/moss_noise.raw")).unwrap();

    let (p, last_trailer_idx) = decode_events_take_n(&f, take, Some(skip)).unwrap();

    println!("Got: {packets} packets", packets = p.len());
    println!("Last trailer at index: {last_trailer_idx}");
    assert_eq!(p.len(), take, "Expected {take} packets, got {}", p.len());
}

#[test]
fn test_decode_events_skip_500_take_100() {
    let skip = 500;
    let take = 100;
    let f = std::fs::read(std::path::PathBuf::from("tests/moss_noise.raw")).unwrap();

    let (p, last_trailer_idx) = decode_events_take_n(&f, take, Some(skip)).unwrap();

    println!("Got: {packets} packets", packets = p.len());
    println!("Last trailer at index: {last_trailer_idx}");
    assert_eq!(p.len(), take, "Expected {take} packets, got {}", p.len());
}

#[test]
fn test_decode_events_skip_99000_take_1000() {
    let skip = 99000;
    let take = 1000;
    let f = std::fs::read(std::path::PathBuf::from("tests/moss_noise.raw")).unwrap();

    let (p, last_trailer_idx) = decode_events_take_n(&f, take, Some(skip)).unwrap();
    println!("Got: {packets} packets", packets = p.len());
    println!("Last trailer at index: {last_trailer_idx}");
    assert_eq!(p.len(), take, "Expected {take} packets, got {}", p.len());
}

const FILE_4_EVENTS_PARTIAL_END: &str = "tests/moss_noise_0-499b.raw";
const FILE_3_EVENTS_PARTIAL_START: &str = "tests/moss_noise_500-999b.raw";

#[test]
#[should_panic = "Failed decoding packet #5"]
fn test_decode_split_events_skip_0_take_5() {
    pyo3::prepare_freethreaded_python();
    let take = 5;
    let f = std::fs::read(std::path::PathBuf::from(FILE_4_EVENTS_PARTIAL_END)).unwrap();

    let (p, last_trailer_idx) = decode_events_take_n(&f, take, None).unwrap();

    println!("Got: {packets} packets", packets = p.len());
    println!("Last trailer at index: {last_trailer_idx}");
    assert_eq!(p.len(), take, "Expected {take} packets, got {}", p.len());
}

#[test]
fn test_decode_split_events_skip_1_take_2() {
    pyo3::prepare_freethreaded_python();
    let skip = 1;
    let take = 2;
    let f = std::fs::read(std::path::PathBuf::from(FILE_4_EVENTS_PARTIAL_END)).unwrap();

    let (p, last_trailer_idx) = decode_events_take_n(&f, take, Some(skip)).unwrap();

    println!("Got: {packets} packets", packets = p.len());
    println!("Last trailer at index: {last_trailer_idx}");
    assert_eq!(p.len(), take, "Expected {take} packets, got {}", p.len());
}

#[test]
fn test_decode_split_events_from_partial_event_skip_1_take_2() {
    pyo3::prepare_freethreaded_python();
    let skip = 1;
    let take = 2;
    let f = std::fs::read(std::path::PathBuf::from(FILE_3_EVENTS_PARTIAL_START)).unwrap();

    let (p, last_trailer_idx) = decode_events_take_n(&f, take, Some(skip)).unwrap();

    println!("Got: {packets} packets", packets = p.len());
    println!("Last trailer at index: {last_trailer_idx}");
    assert_eq!(p.len(), take, "Expected {take} packets, got {}", p.len());
}

#[test]
fn test_decode_split_events_with_remainder() {
    pyo3::prepare_freethreaded_python();
    let take = 100;
    let f = std::fs::read(std::path::PathBuf::from(FILE_4_EVENTS_PARTIAL_END)).unwrap();

    assert!(decode_events_take_n(&f, take, None).is_err());

    let (packets, remainder) = decode_events_skip_n_take_all_with_remainder(&f, 0).unwrap();

    let remainder = remainder.unwrap();

    println!("Got: {packets} packets", packets = packets.len());
    println!("Remainder: {remainder} bytes", remainder = remainder.len());
    assert_eq!(packets.len(), 4);
    assert_eq!(remainder.len(), 43);
}
