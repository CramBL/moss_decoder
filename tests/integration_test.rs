use crate::moss_protocol::test_util::*;
use moss_decoder::*;

use pretty_assertions::assert_eq;

const FILE_MOSS_NOISE: &str = "tests/test-data/moss_noise.raw";
const MOSS_NOISE_PACKETS: usize = 100000;
const MOSS_NOISE_HITS: usize = 2716940;
const MOSS_NOISE_LAST_TRAILER_IDX: usize = 9582574;

const FILE_MOSS_NOISE_ALL_REGION: &str = "tests/test-data/noise_all_regions.raw";
const NOISE_ALL_REGION_PACKETS: usize = 1000;
const NOISE_ALL_REGION_HITS: usize = 6085;
const NOISE_ALL_REGION_LAST_TRAILER_IDX: usize = 26542;

const FILE_NOISE_RANDOM_REGION: &str = "tests/test-data/noise_random_region.raw";
const NOISE_RANDOM_REGION_PACKETS: usize = 1044;
const NOISE_RANDOM_REGION_HITS: usize = 5380;
const NOISE_RANDOM_REGION_LAST_TRAILER_IDX: usize = 22696;

const FILE_PATTERN_ALL_REGIONS: &str = "tests/test-data/pattern_all_regions.raw";
const PATTERN_ALL_REGIONS_PACKETS: usize = 1000;
const PATTERN_ALL_REGIONS_HITS: usize = 4000;
const PATTERN_ALL_REGIONS_LAST_TRAILER_IDX: usize = 19997;

const FILE_4_EVENTS_PARTIAL_END: &str = "tests/test-data/moss_noise_0-499b.raw"; // 4 events, last event is partial ~4.5 events
const FOUR_EVENTS_PARTIAL_END_PACKETS: usize = 4;
const FOUR_EVENTS_PARTIAL_END_HITS: usize = 128;
const FOUR_EVENTS_PARTIAL_END_LAST_TRAILER_IDX: usize = 456;

const FILE_3_EVENTS_PARTIAL_START: &str = "tests/test-data/moss_noise_500-999b.raw"; // 3 events, first event is partial ~3.5 events, also ends with a partial event
const THREE_EVENTS_PARTIAL_START_PACKETS: usize = 3;
const THREE_EVENTS_PARTIAL_START_HITS: usize = 77;
const THREE_EVENTS_PARTIAL_START_LAST_TRAILER_IDX: usize = 379;

// Utility to compare all packets in two vectors (for comparing result of different decoding methods)
fn compare_all_packets(a_packets: &[MossPacket], b_packets: &[MossPacket]) {
    assert_eq!(a_packets.len(), b_packets.len());

    for (i, a_packet) in a_packets.iter().enumerate() {
        let b_packet = &b_packets[i];
        assert_eq!(a_packet.unit_id, b_packet.unit_id);
        assert_eq!(a_packet.hits.len(), b_packet.hits.len());
        for (j, a_hit) in a_packet.hits.iter().enumerate() {
            let b_hit = &b_packet.hits[j];

            assert_eq!(a_hit.region, b_hit.region);
            assert_eq!(a_hit.row, b_hit.row);
            assert_eq!(a_hit.column, b_hit.column);
        }
    }
}

// Compare the result of all decoding methods that can decode all packets from a file/byte array
fn compare_all_decoding_methods(
    test_file: &str,
    expect_packets: usize,
    expect_hits: usize,
    expect_trailer_idx: usize,
) {
    let bytes = std::fs::read(std::path::PathBuf::from(test_file)).unwrap();

    // Do an initial comparison with the simple naive decoder and the expected values
    let (debug_packets, debug_last_trailer_idx, invalid_words) =
        moss_decoder::debug_decode_all_events(&bytes).unwrap();
    assert_eq!(debug_last_trailer_idx, expect_trailer_idx, "Unexpected last trailer index, got trailer index: {debug_last_trailer_idx}, expected: {expect_trailer_idx}. From trailer index to end of bytes: {remainder:#X?}", remainder = bytes.get(debug_last_trailer_idx..).unwrap());
    assert_eq!(
        debug_packets.len(),
        expect_packets,
        "Unexpected number of packets, got {packets}, expected {expect_packets}",
        packets = debug_packets.len()
    );
    assert_eq!(
        debug_packets.iter().fold(0, |acc, p| acc + p.hits.len()),
        expect_hits
    );
    assert_eq!(invalid_words.len(), 0);

    // Then use that result to compare with the other decoding methods

    // Check moss_decoder::debug_decode_all_events_from_file
    let (debug_packets_from_file, debug_last_trailer_idx_from_file, invalid_words_from_file) =
        moss_decoder::debug_decode_all_events_from_file(test_file.into()).unwrap();
    assert_eq!(
        debug_last_trailer_idx_from_file, debug_last_trailer_idx,
        "Unexpected last trailer index, got trailer index: {debug_last_trailer_idx_from_file}, expected: {debug_last_trailer_idx}. From trailer index to end of bytes: {remainder:#X?}",
        remainder = bytes.get(debug_last_trailer_idx_from_file..).unwrap()
    );
    compare_all_packets(&debug_packets, &debug_packets_from_file);
    assert_eq!(invalid_words_from_file.len(), 0);

    // Check moss_decoder::decode_all_events
    let (decode_all_events_packets, decode_all_events_last_trailer_idx) =
        moss_decoder::decode_all_events(&bytes).unwrap();
    assert_eq!(debug_last_trailer_idx, decode_all_events_last_trailer_idx);
    compare_all_packets(&debug_packets, &decode_all_events_packets);

    // Check moss_decoder::decode_from_file
    let packets = moss_decoder::decode_from_file(test_file.into()).unwrap();
    compare_all_packets(&packets, &decode_all_events_packets);

    // Check moss_decoder::skip_n_take_all
    let (packets, remainder) = moss_decoder::skip_n_take_all(&bytes, 0).unwrap();
    let packets = packets.unwrap();
    assert!(remainder.is_none());
    compare_all_packets(&packets, &decode_all_events_packets);

    // Check moss_decoder::decode_n_events
    let (packets, last_trailer_idx) =
        moss_decoder::decode_n_events(&bytes, expect_packets, None, None).unwrap();
    assert_eq!(last_trailer_idx, debug_last_trailer_idx);
    compare_all_packets(&packets, &decode_all_events_packets);
}

#[test]
fn test_decoding_single_event() {
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
    let f = std::fs::read(std::path::PathBuf::from(FILE_MOSS_NOISE)).unwrap();
    println!(
        "Read file in: {t:?}. Bytes: {cnt}",
        t = time.elapsed(),
        cnt = f.len()
    );

    println!("Decoding content...");
    let (p, last_trailer_idx) = decode_all_events(&f).unwrap();
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

    let packets = moss_decoder::decode_from_file(FILE_MOSS_NOISE.to_string().into()).unwrap();
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
fn test_decode_from_file_noise_all_region() {
    let packets =
        moss_decoder::decode_from_file(FILE_MOSS_NOISE_ALL_REGION.to_string().into()).unwrap();
    assert_eq!(
        packets.len(),
        NOISE_ALL_REGION_PACKETS,
        "Expected {NOISE_ALL_REGION_PACKETS} packets, got {}",
        packets.len()
    );
    // Count total hits
    let total_hits = packets.iter().fold(0, |acc, p| acc + p.hits.len());
    assert_eq!(
        total_hits, NOISE_ALL_REGION_HITS,
        "Expected {NOISE_ALL_REGION_HITS} hits, got {total_hits}",
    );
}

#[test]
fn test_decode_from_file_noise_random_region() {
    let expect_packets = 1044;
    let expect_hits = 5380;

    let packets =
        moss_decoder::decode_from_file(FILE_NOISE_RANDOM_REGION.to_string().into()).unwrap();
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
fn test_decode_from_file_pattern_all_region() {
    let expect_packets = 1000;
    let expect_hits = 4000;

    let packets =
        moss_decoder::decode_from_file(FILE_PATTERN_ALL_REGIONS.to_string().into()).unwrap();
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

    let f = std::fs::read(std::path::PathBuf::from(FILE_MOSS_NOISE)).unwrap();
    println!(
        "Read file in: {t:?}. Bytes: {cnt}",
        t = time.elapsed(),
        cnt = f.len()
    );

    println!("Decoding content...");
    let (p, last_trailer_idx) = decode_all_events(&f).unwrap();
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

    let packets = moss_decoder::decode_from_file(FILE_MOSS_NOISE.to_string().into()).unwrap();
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
    let f = std::fs::read(std::path::PathBuf::from(FILE_MOSS_NOISE)).unwrap();
    let (p, last_trailer_idx) = decode_n_events(&f, take, None, None).unwrap();

    println!("Got: {packets} packets", packets = p.len());
    println!("Last trailer at index: {last_trailer_idx}");
    assert_eq!(p.len(), take, "Expected {take} packets, got {}", p.len());
}

#[test]
fn test_decode_events_skip_10_take_1() {
    let skip = 10;
    let take = 1;
    let f = std::fs::read(std::path::PathBuf::from(FILE_MOSS_NOISE)).unwrap();

    let (p, last_trailer_idx) = decode_n_events(&f, take, Some(skip), None).unwrap();

    println!("Got: {packets} packets", packets = p.len());
    println!("Last trailer at index: {last_trailer_idx}");
    assert_eq!(p.len(), take, "Expected {take} packets, got {}", p.len());
}

#[test]
fn test_decode_events_skip_500_take_100() {
    let skip = 500;
    let take = 100;
    let f = std::fs::read(std::path::PathBuf::from(FILE_MOSS_NOISE)).unwrap();

    let (p, last_trailer_idx) = decode_n_events(&f, take, Some(skip), None).unwrap();

    println!("Got: {packets} packets", packets = p.len());
    println!("Last trailer at index: {last_trailer_idx}");
    assert_eq!(p.len(), take, "Expected {take} packets, got {}", p.len());
}

#[test]
fn test_decode_events_skip_99000_take_1000() {
    let skip = 99000;
    let take = 1000;
    let f = std::fs::read(std::path::PathBuf::from(FILE_MOSS_NOISE)).unwrap();

    let (p, last_trailer_idx) = decode_n_events(&f, take, Some(skip), None).unwrap();
    println!("Got: {packets} packets", packets = p.len());
    println!("Last trailer at index: {last_trailer_idx}");
    assert_eq!(p.len(), take, "Expected {take} packets, got {}", p.len());
}

#[test]
#[should_panic = "Failed decoding packet #5"]
fn test_decode_split_events_skip_0_take_5() {
    pyo3::prepare_freethreaded_python();
    let take = 5;
    let f = std::fs::read(std::path::PathBuf::from(FILE_4_EVENTS_PARTIAL_END)).unwrap();

    let (p, last_trailer_idx) = decode_n_events(&f, take, None, None).unwrap();

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

    let (p, last_trailer_idx) = decode_n_events(&f, take, Some(skip), None).unwrap();

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

    let (p, last_trailer_idx) = decode_n_events(&f, take, Some(skip), None).unwrap();

    println!("Got: {packets} packets", packets = p.len());
    println!("Last trailer at index: {last_trailer_idx}");
    assert_eq!(p.len(), take, "Expected {take} packets, got {}", p.len());
}

#[test]
fn test_decode_split_events_with_remainder() {
    pyo3::prepare_freethreaded_python();
    let take = 100;
    let f = std::fs::read(std::path::PathBuf::from(FILE_4_EVENTS_PARTIAL_END)).unwrap();

    assert!(decode_n_events(&f, take, None, None).is_err());

    let (packets, remainder) = skip_n_take_all(&f, 0).unwrap();

    let remainder = remainder.unwrap();
    let packets = packets.unwrap();

    println!("Got: {packets} packets", packets = packets.len());
    println!("Remainder: {remainder} bytes", remainder = remainder.len());
    assert_eq!(packets.len(), 4);
    assert_eq!(remainder.len(), 43);
}

#[test]
fn test_decode_split_events_from_both_files() {
    pyo3::prepare_freethreaded_python();
    let take = 6;
    let f = std::fs::read(std::path::PathBuf::from(FILE_4_EVENTS_PARTIAL_END)).unwrap();
    let f2 = std::fs::read(std::path::PathBuf::from(FILE_3_EVENTS_PARTIAL_START)).unwrap();

    // First attempt to decode 6 events from the first file, that should fail
    assert!(decode_n_events(&f, take, None, None).is_err());

    // Then fall back to decoding as many as possible and returning the remainder
    let (packets, remainder) = skip_n_take_all(&f, 0).unwrap();
    let packets = packets.unwrap();
    let decoded_packets = packets.len();

    // Now take the rest from the remainder and the next file
    let (packets2, last_trailer_idx) =
        decode_n_events(&f2, take - decoded_packets, None, remainder).unwrap();

    println!("Got: {packets} packets", packets = packets.len());
    println!("Got: {packets2} packets", packets2 = packets2.len());
    println!("Last trailer at index: {last_trailer_idx}");
    assert_eq!(packets.len() + packets2.len(), take);
}

#[test]
fn test_decode_2_events_from_path() {
    pyo3::prepare_freethreaded_python();
    let take = 2;
    let p = std::path::PathBuf::from(FILE_4_EVENTS_PARTIAL_END);
    let res = decode_n_events_from_file(p, take, None, None);
    let packets = res.unwrap();
    println!("Got: {packets} packets", packets = packets.len());
    assert_eq!(packets.len(), take);
}

#[test]
fn test_decode_split_events_from_path_repeated_until_err() {
    pyo3::prepare_freethreaded_python();
    let take_first = 2;
    let p = std::path::PathBuf::from(FILE_4_EVENTS_PARTIAL_END);
    let res = decode_n_events_from_file(p.clone(), take_first, None, None);
    let mut running_packets = res.unwrap();
    println!("Got: {packets} packets", packets = running_packets.len());
    assert_eq!(running_packets.len(), take_first);

    let take_second = 2;
    let res = decode_n_events_from_file(p.clone(), take_second, Some(running_packets.len()), None);
    running_packets.extend(res.unwrap());
    println!("Got: {packets} packets", packets = running_packets.len());
    assert_eq!(running_packets.len(), take_first + take_second);

    let take_third = 2;
    let res = decode_n_events_from_file(p, take_third, Some(running_packets.len()), None);
    println!("Got : {:?}", res);
    assert!(res.is_err());
    assert!(res
        .unwrap_err()
        .to_string()
        .contains("No MOSS Packets in events"));
}

#[test]
fn test_decode_split_events_from_path_take_too_many() {
    pyo3::prepare_freethreaded_python();
    let take_first = 10;
    let p = std::path::PathBuf::from(FILE_4_EVENTS_PARTIAL_END);
    let res = decode_n_events_from_file(p.clone(), take_first, None, None);
    println!("Got : {:?}", res);
    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("BytesWarning"));
}

#[test]
fn test_skip_n_take_all_from_file() {
    pyo3::prepare_freethreaded_python();
    let p = std::path::PathBuf::from(FILE_4_EVENTS_PARTIAL_END);
    let res = skip_n_take_all_from_file(p.clone(), 0);
    assert!(res.is_ok());
    let (packets, remainder) = res.unwrap();
    assert!(packets.is_some());
    assert!(remainder.is_some());
    assert_eq!(packets.unwrap().len(), 4);
    let remainder = remainder.unwrap();
    println!("Got {} remainder bytes", remainder.len());
    println!("Got remainder: {:02X?}", remainder);

    let (packets, _) = skip_n_take_all_from_file(p.clone(), 1).unwrap();
    assert_eq!(packets.unwrap().len(), 3);
    let (packets, _) = skip_n_take_all_from_file(p.clone(), 2).unwrap();
    assert_eq!(packets.unwrap().len(), 2);
    let (packets, _) = skip_n_take_all_from_file(p.clone(), 3).unwrap();
    assert_eq!(packets.unwrap().len(), 1);
    let (packets, _) = skip_n_take_all_from_file(p.clone(), 4).unwrap();
    assert!(packets.is_none());
}

#[test]
fn test_decode_split_events_from_file_spillover() {
    pyo3::prepare_freethreaded_python();
    let mut running_packets = Vec::new();
    let take = 2;
    let p = std::path::PathBuf::from(FILE_4_EVENTS_PARTIAL_END);
    loop {
        let skip = if running_packets.is_empty() {
            None
        } else {
            Some(running_packets.len())
        };
        let res = decode_n_events_from_file(p.clone(), take, skip, None);
        if res.is_err() {
            println!("Got error: {:?}", res);
            break;
        }
        running_packets.extend(res.unwrap());
    }
    let skip = running_packets.len();
    let (packets, remainder) = skip_n_take_all_from_file(p.clone(), skip).unwrap();
    assert!(
        packets.is_none(),
        "take is two ({take}) but there's still packets in the file"
    );
    let p2 = std::path::PathBuf::from(FILE_3_EVENTS_PARTIAL_START);
    let res = decode_n_events_from_file(p2.clone(), take, None, remainder);
    assert_eq!(res.unwrap().len(), 2);
}

#[test]
fn test_debug_decode_noise_all_region() {
    pyo3::prepare_freethreaded_python();

    let time = std::time::Instant::now();

    let bytes = std::fs::read(std::path::PathBuf::from(FILE_MOSS_NOISE_ALL_REGION)).unwrap();

    let res = moss_decoder::debug_decode_all_events(&bytes);

    println!("Decoded in: {t:?}\n", t = time.elapsed());

    let (packets, last_trailer_idx, invalid_words) = res.unwrap();

    println!("Got: {packets} packets", packets = packets.len());
    println!(
        "Last trailer at index: {last_trailer_idx}/{}",
        bytes.len() - 1
    );
    println!(
        "Got: {invalid_words} invalid words",
        invalid_words = invalid_words.len()
    );

    assert_eq!(packets.len(), NOISE_ALL_REGION_PACKETS);
    assert_eq!(
        last_trailer_idx,
        bytes.len() - 2,
        "Last 10 bytes of file: {:X?}",
        bytes.get(bytes.len() - 10..)
    );
    assert_eq!(invalid_words.len(), 0);

    // Count total hits
    let total_hits = packets.iter().fold(0, |acc, p| acc + p.hits.len());
    assert_eq!(
        total_hits, NOISE_ALL_REGION_HITS,
        "Expected {NOISE_ALL_REGION_HITS} hits, got {total_hits}",
    );
}

#[test]
fn test_compare_result_noise_all_region() {
    pyo3::prepare_freethreaded_python();
    compare_all_decoding_methods(
        FILE_MOSS_NOISE_ALL_REGION,
        NOISE_ALL_REGION_PACKETS,
        NOISE_ALL_REGION_HITS,
        NOISE_ALL_REGION_LAST_TRAILER_IDX,
    );
}

#[test]
fn test_compare_result_noise_random_region() {
    pyo3::prepare_freethreaded_python();
    compare_all_decoding_methods(
        FILE_NOISE_RANDOM_REGION,
        NOISE_RANDOM_REGION_PACKETS,
        NOISE_RANDOM_REGION_HITS,
        NOISE_RANDOM_REGION_LAST_TRAILER_IDX,
    );
}

#[test]
fn test_compare_result_pattern_all_regions() {
    pyo3::prepare_freethreaded_python();
    compare_all_decoding_methods(
        FILE_PATTERN_ALL_REGIONS,
        PATTERN_ALL_REGIONS_PACKETS,
        PATTERN_ALL_REGIONS_HITS,
        PATTERN_ALL_REGIONS_LAST_TRAILER_IDX,
    );
}

#[test]
fn test_compare_result_moss_noise() {
    pyo3::prepare_freethreaded_python();
    compare_all_decoding_methods(
        FILE_MOSS_NOISE,
        MOSS_NOISE_PACKETS,
        MOSS_NOISE_HITS,
        MOSS_NOISE_LAST_TRAILER_IDX,
    );
}

#[test]
fn test_compare_result_4_events_partial_end() {
    pyo3::prepare_freethreaded_python();
    let bytes = std::fs::read(std::path::PathBuf::from(FILE_4_EVENTS_PARTIAL_END)).unwrap();

    // Do an initial comparison with the simple naive decoder and the expected values
    let (debug_packets, debug_last_trailer_idx, invalid_words) =
        moss_decoder::debug_decode_all_events(&bytes).unwrap();
    assert_eq!(debug_last_trailer_idx, FOUR_EVENTS_PARTIAL_END_LAST_TRAILER_IDX, "Unexpected last trailer index, got trailer index: {debug_last_trailer_idx}, expected: {FOUR_EVENTS_PARTIAL_END_LAST_TRAILER_IDX}. From trailer index to end of bytes: {remainder:#X?}", remainder = bytes.get(debug_last_trailer_idx..).unwrap());
    assert_eq!(
        debug_packets.len(),
        FOUR_EVENTS_PARTIAL_END_PACKETS,
        "Unexpected number of packets, got {packets}, expected {FOUR_EVENTS_PARTIAL_END_PACKETS}",
        packets = debug_packets.len()
    );
    assert_eq!(
        debug_packets.iter().fold(0, |acc, p| acc + p.hits.len()),
        FOUR_EVENTS_PARTIAL_END_HITS
    );
    assert_eq!(invalid_words.len(), 0);

    // Then use that result to compare with the other decoding methods

    // Check moss_decoder::decode_all_events
    match moss_decoder::decode_all_events(&bytes) {
        Ok((decode_all_events_packets, decode_all_events_last_trailer_idx)) => panic!("This should have failed, got {decode_all_events_packets:?} packets, last trailer index: {decode_all_events_last_trailer_idx}"),
        Err(e) => {println!("Got error: {e}"); assert!(e.to_string().contains("Failed decoding packet #5"))},
    }

    // Check moss_decoder::decode_from_file
    let packets = moss_decoder::decode_from_file(FILE_4_EVENTS_PARTIAL_END.into()).unwrap();
    compare_all_packets(&packets, &debug_packets);

    // Check moss_decoder::skip_n_take_all
    let (packets, remainder) = moss_decoder::skip_n_take_all(&bytes, 0).unwrap();
    let packets = packets.unwrap();
    assert!(remainder.is_some());
    assert!(
        remainder.unwrap().len() == bytes.len() - (FOUR_EVENTS_PARTIAL_END_LAST_TRAILER_IDX + 1)
    );
    compare_all_packets(&packets, &debug_packets);

    // Check moss_decoder::decode_n_events
    let (packets, last_trailer_idx) =
        moss_decoder::decode_n_events(&bytes, FOUR_EVENTS_PARTIAL_END_PACKETS, None, None).unwrap();
    assert_eq!(last_trailer_idx, debug_last_trailer_idx);
    compare_all_packets(&packets, &debug_packets);
}

#[test]
fn test_compare_result_3_events_partial_start() {
    pyo3::prepare_freethreaded_python();
    let bytes = std::fs::read(std::path::PathBuf::from(FILE_3_EVENTS_PARTIAL_START)).unwrap();

    // Do an initial comparison with the simple naive decoder and the expected values
    let (debug_packets, debug_last_trailer_idx, invalid_words) =
        moss_decoder::debug_decode_all_events(&bytes).unwrap();
    assert_eq!(debug_last_trailer_idx, THREE_EVENTS_PARTIAL_START_LAST_TRAILER_IDX, "Unexpected last trailer index, got trailer index: {debug_last_trailer_idx}, expected: {THREE_EVENTS_PARTIAL_START_LAST_TRAILER_IDX}. From trailer index to end of bytes: {remainder:#X?}", remainder = bytes.get(debug_last_trailer_idx..).unwrap());
    assert_eq!(
        debug_packets.len(),
        THREE_EVENTS_PARTIAL_START_PACKETS,
        "Unexpected number of packets, got {packets}, expected {THREE_EVENTS_PARTIAL_START_PACKETS}",
        packets = debug_packets.len()
    );
    assert_eq!(
        debug_packets.iter().fold(0, |acc, p| acc + p.hits.len()),
        THREE_EVENTS_PARTIAL_START_HITS
    );
    assert_eq!(invalid_words.len(), 108);

    // Then use that result to compare with the other decoding methods

    // Check moss_decoder::decode_all_events
    match moss_decoder::decode_all_events(&bytes) {
        Ok((decode_all_events_packets, decode_all_events_last_trailer_idx)) => panic!("This should have failed, got {decode_all_events_packets:?} packets, last trailer index: {decode_all_events_last_trailer_idx}"),
        Err(e) => {println!("Got error: {e}"); assert!(e.to_string().contains("Failed decoding packet #1"))},
    }

    // Check moss_decoder::decode_from_file
    match moss_decoder::decode_from_file(FILE_3_EVENTS_PARTIAL_START.into()) {
        Ok(packets) => panic!("This should have failed, got {packets:?} packets"),
        Err(e) => {
            println!("Got error: {e}");
            assert!(e.to_string().contains("Failed decoding packet #1"))
        }
    }

    // Check moss_decoder::skip_n_take_all
    match moss_decoder::skip_n_take_all(&bytes, 0) {
        Ok(packets) => panic!("This should have failed, got {packets:?} packets"),
        Err(e) => {
            println!("Got error: {e}");
            assert!(e.to_string().contains("Failed decoding packet #1"))
        }
    }

    // Check moss_decoder::decode_n_events
    match moss_decoder::decode_n_events(&bytes, THREE_EVENTS_PARTIAL_START_PACKETS, None, None) {
        Ok(packets) => panic!("This should have failed, got {packets:?} packets"),
        Err(e) => {
            println!("Got error: {e}");
            assert!(e.to_string().contains("Failed decoding packet #1"))
        }
    }
}
