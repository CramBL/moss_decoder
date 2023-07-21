use moss_protocol::MossPacket;
use pyo3::prelude::*;

pub mod moss_protocol;
use moss_protocol::MossHit;
use moss_protocol::MossWord;

/// A Python module for decoding raw MOSS data in Rust.
#[pymodule]
fn moss_decoder(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(decode_event, m)?)?;

    m.add_class::<MossHit>()?;

    Ok(())
}

/// Formats the sum of two numbers as string.
#[pyfunction]
fn decode_event(bytes: Vec<u8>) -> PyResult<MossPacket> {
    let mut hits = Vec::new();

    let mut packet = MossPacket {
        unit_id: 255, // placeholder
        hits: Vec::new(),
    };
    let mut is_moss_packet = false;
    let mut current_region: u8 = 0xff; // placeholder
    bytes.iter().for_each(|b| {
        if let Ok(word) = MossWord::from_byte(*b) {
            match word {
                MossWord::Idle => (),
                MossWord::UnitFrameHeader => {
                    debug_assert!(!is_moss_packet);
                    is_moss_packet = true;
                    packet.unit_id = *b & 0x0F
                }
                MossWord::UnitFrameTrailer => {
                    debug_assert!(is_moss_packet);
                    is_moss_packet = false
                }
                MossWord::RegionHeader => {
                    debug_assert!(is_moss_packet);
                    current_region = *b & 0x03;
                }
                MossWord::Data0 => {
                    debug_assert!(is_moss_packet);
                    hits.push(MossHit {
                        region: current_region,         // region id
                        row: ((*b & 0x3F) as u16) << 3, // row position [8:3]
                        column: 123,                    // placeholder
                    });
                }
                MossWord::Data1 => {
                    debug_assert!(is_moss_packet);
                    hits.last_mut().unwrap().row |= ((*b & 0x38) >> 3) as u16; // row position [2:0]
                    hits.last_mut().unwrap().column = ((*b & 0x07) as u16) << 6;
                    // position [8:6]
                }
                MossWord::Data2 => {
                    debug_assert!(is_moss_packet);
                    hits.last_mut().unwrap().column |= (*b & 0x3F) as u16; // col position [5:0]
                }
                MossWord::Delimiter => debug_assert!(!is_moss_packet),
            }
        }
    });
    debug_assert_ne!(packet.unit_id, 0xff);
    packet.hits.append(&mut hits);
    Ok(packet)
}

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

    fn fake_events_simple() -> Vec<u8> {
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

    #[test]
    fn test_decoding() {
        //
        let event = fake_events_simple();

        let packet = decode_event(event);

        assert_eq!(
            packet.unwrap(),
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
            }
        );
    }
}
