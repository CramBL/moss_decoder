use moss_protocol::MossPacket;
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;

pub mod moss_protocol;
use moss_protocol::MossHit;
use moss_protocol::MossWord;

/// A Python module for decoding raw MOSS data in Rust.
#[pymodule]
fn moss_decoder(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(decode_event, m)?)?;

    m.add_class::<MossHit>()?;
    m.add_class::<MossPacket>()?;

    Ok(())
}

const INVALID_NO_HEADER_SEEN: u8 = 0xFF;
/// Decodes a single MOSS event into a [MossPacket]
#[pyfunction]
fn decode_event(bytes: Vec<u8>) -> PyResult<(MossPacket, Vec<u8>)> {
    let mut hits = Vec::new();

    let mut packet = MossPacket {
        unit_id: INVALID_NO_HEADER_SEEN, // placeholder
        hits: Vec::new(),
    };
    let mut processed_bytes_idx = 0;

    let mut is_moss_packet = false;
    let mut current_region: u8 = 0xff; // placeholder

    for (i, byte) in bytes.iter().enumerate() {
        if let Ok(word) = MossWord::from_byte(*byte) {
            match word {
                MossWord::Idle => (),
                MossWord::UnitFrameHeader => {
                    debug_assert!(!is_moss_packet);
                    is_moss_packet = true;
                    packet.unit_id = *byte & 0x0F
                }
                MossWord::UnitFrameTrailer => {
                    debug_assert!(is_moss_packet);
                    processed_bytes_idx = i + 1;
                    break;
                }
                MossWord::RegionHeader => {
                    debug_assert!(is_moss_packet);
                    current_region = *byte & 0x03;
                }
                MossWord::Data0 => {
                    debug_assert!(is_moss_packet);
                    hits.push(MossHit {
                        region: current_region,            // region id
                        row: ((*byte & 0x3F) as u16) << 3, // row position [8:3]
                        column: 0,                         // placeholder
                    });
                }
                MossWord::Data1 => {
                    debug_assert!(is_moss_packet);
                    // row position [2:0]
                    hits.last_mut().unwrap().row |= ((*byte & 0x38) >> 3) as u16;
                    // col position [8:6]
                    hits.last_mut().unwrap().column = ((*byte & 0x07) as u16) << 6;
                }
                MossWord::Data2 => {
                    debug_assert!(is_moss_packet);
                    hits.last_mut().unwrap().column |= (*byte & 0x3F) as u16; // col position [5:0]
                }
                MossWord::Delimiter => {
                    debug_assert!(!is_moss_packet);
                    processed_bytes_idx = i + 1;
                    break;
                }
            }
        }
    }

    if packet.unit_id == INVALID_NO_HEADER_SEEN {
        return Err(PyTypeError::new_err("No MOSS Packets in event"));
    }
    packet.hits.append(&mut hits);
    let (_processed, unprocessed) = bytes.split_at(processed_bytes_idx);
    Ok((packet, unprocessed.to_vec()))
}

// #[pyfunction]
// fn decode_multiple_events(bytes: Vec<u8>) -> PyResult<Vec<MossPacket>> {}

#[cfg(test)]
mod tests {
    use super::*;

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

        let (packet, unprocessed_bytes) = decode_event(event).unwrap();

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

        if let Ok((packet, _unprocessed_data)) = decode_event(events) {
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

        while let Ok((packet, unprocessed_data)) = decode_event(events) {
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
    fn test_decoding_multiple_events_delimiter() {
        let mut events = fake_multiple_events();
        events.append(&mut vec![0xFA, 0xFA, 0xFA]);

        let mut moss_packets: Vec<MossPacket> = Vec::new();

        while let Ok((packet, unprocessed_data)) = decode_event(events) {
            moss_packets.push(packet);
            events = unprocessed_data;
        }

        let packet_count = moss_packets.len();

        println!("{packet_count}");

        for p in moss_packets {
            println!("{p:?}");
        }
    }
}
