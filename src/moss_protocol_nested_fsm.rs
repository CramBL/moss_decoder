//! Contains an FSM implementation of the MOSS data readout protocol
#![allow(non_camel_case_types)]

use crate::moss_protocol::MossWord;
use crate::MossHit;
use crate::MossPacket;

/// Advances the iterator until a Unit Frame Header is encountered, saves the unit ID,
/// and extracts the hits with the [extract_hits] function, before returning a MossPacket if one is found.
#[inline]
pub(crate) fn extract_packet<'a>(
    mut bytes: &mut impl Iterator<Item = &'a u8>,
) -> Option<MossPacket> {
    let mut unit_id: Option<u8> = None;
    for b in &mut bytes {
        if (0xD0..=0xD9).contains(b) {
            unit_id = Some(b & 0xF);
            break;
        }
    }

    unit_id.map(|unit_id| MossPacket {
        unit_id,
        hits: extract_hits(bytes).unwrap_or_else(|| Vec::with_capacity(0)),
    })
}

sm::sm! {

    MossDataFSM {
        InitialStates { _REGION_HEADER0_ }

        _Data {
            _REGION_HEADER0_ => DATA0_
            REGION_HEADER1_ => DATA0_
            REGION_HEADER2_ => DATA0_
            REGION_HEADER3_ => DATA0_
            DATA2_ => DATA0_
            IDLE_ => DATA0_
            DATA0_ => DATA1_
            DATA1_ => DATA2_
        }

        _Idle {
            DATA2_ => IDLE_
        }

        _RegionHeader0 {
            _REGION_HEADER0_ => _REGION_HEADER0_
        }

        _RegionHeader1 {
            _REGION_HEADER0_ => REGION_HEADER1_
            DATA2_ => REGION_HEADER1_
            IDLE_ => REGION_HEADER1_
        }

        _RegionHeader2 {
            REGION_HEADER1_ => REGION_HEADER2_
            DATA2_ => REGION_HEADER2_
            IDLE_ => REGION_HEADER2_
        }

        _RegionHeader3 {
            REGION_HEADER2_ => REGION_HEADER3_
            DATA2_ => REGION_HEADER3_
            IDLE_ => REGION_HEADER3_
        }

        _FrameTrailer {
            REGION_HEADER3_ => FRAME_TRAILER_
            DATA2_ => FRAME_TRAILER_
            IDLE_ => FRAME_TRAILER_
        }
    }
}

use MossDataFSM::Variant::*;
use MossDataFSM::*;

const REGION_HEADER0: u8 = 0xC0;
const REGION_HEADER1: u8 = 0xC1;
const REGION_HEADER2: u8 = 0xC2;
const REGION_HEADER3: u8 = 0xC3;

/// Take an iterator that should be advanced to the position after a unit frame header.
/// Advances the iterator and decodes any observed hits until a Unit Frame Trailer is encountered at which point the iteration stops.
/// Returns all the decoded [MossHit]s if any.
#[inline]
pub(crate) fn extract_hits<'a>(bytes: &mut impl Iterator<Item = &'a u8>) -> Option<Vec<MossHit>> {
    let mut sm = MossDataFSM::Machine::new(_REGION_HEADER0_).as_enum();
    let mut hits = Vec::<MossHit>::new();

    let mut current_region = 0xff;

    for b in bytes {
        sm = match sm {
            Initial_REGION_HEADER0_(st) => match *b {
                REGION_HEADER0 => st.transition(_RegionHeader0).as_enum(),
                _ => unreachable!("Expected Region Header 0, got: {b:#X}"),
            },
            _REGION_HEADER0_By_RegionHeader0(st) => match *b {
                REGION_HEADER1 => {
                    current_region = 1;
                    st.transition(_RegionHeader1).as_enum()
                }
                0..=0b0011_1111 => {
                    current_region = 0;
                    add_data0(&mut hits, *b, current_region);
                    st.transition(_Data).as_enum()
                }
                _ => unreachable!("Expected Region Header 1 or DATA 0, got: {b:#X}"),
            },
            DATA0_By_Data(st) => {
                add_data1(&mut hits, *b);
                st.transition(_Data).as_enum()
            }
            DATA1_By_Data(st) => {
                add_data2(&mut hits, *b);
                st.transition(_Data).as_enum()
            }
            DATA2_By_Data(st) => match *b {
                0..=0b0011_1111 => {
                    add_data0(&mut hits, *b, current_region);
                    st.transition(_Data).as_enum()
                }
                MossWord::IDLE => st.transition(_Idle).as_enum(),
                REGION_HEADER1 => {
                    current_region = 1;
                    st.transition(_RegionHeader1).as_enum()
                }
                REGION_HEADER2 => {
                    current_region = 2;
                    st.transition(_RegionHeader2).as_enum()
                }
                REGION_HEADER3 => {
                    current_region = 3;
                    st.transition(_RegionHeader3).as_enum()
                }
                MossWord::UNIT_FRAME_TRAILER => break,

                _ => {
                    unreachable!("Expected Region Header 1-3, DATA 0, or IDLE, got: {b:#X}")
                }
            },
            IDLE_By_Idle(st) => match *b {
                0..=0b0011_1111 => {
                    add_data0(&mut hits, *b, 0);
                    st.transition(_Data).as_enum()
                }
                REGION_HEADER1 => {
                    current_region = 1;
                    st.transition(_RegionHeader1).as_enum()
                }
                REGION_HEADER2 => {
                    current_region = 2;
                    st.transition(_RegionHeader2).as_enum()
                }
                REGION_HEADER3 => {
                    current_region = 3;
                    st.transition(_RegionHeader3).as_enum()
                }
                MossWord::UNIT_FRAME_TRAILER => break,

                _ => {
                    unreachable!("Expected Region Header 1-3, DATA 0, or IDLE, got: {b:#X}")
                }
            },
            REGION_HEADER1_By_RegionHeader1(st) => match *b {
                REGION_HEADER2 => {
                    current_region = 2;
                    st.transition(_RegionHeader2).as_enum()
                }
                0..=0b0011_1111 => {
                    current_region = 1;
                    add_data0(&mut hits, *b, current_region);
                    st.transition(_Data).as_enum()
                }
                _ => unreachable!("Expected Region Header 2 or DATA 0, got: {b:#X}"),
            },
            REGION_HEADER2_By_RegionHeader2(st) => match *b {
                REGION_HEADER3 => {
                    current_region = 3;
                    st.transition(_RegionHeader3).as_enum()
                }
                0..=0b0011_1111 => {
                    current_region = 2;
                    add_data0(&mut hits, *b, current_region);
                    st.transition(_Data).as_enum()
                }
                _ => unreachable!("Expected Region Header 3 or DATA 0, got: {b:#X}"),
            },
            REGION_HEADER3_By_RegionHeader3(st) => match *b {
                MossWord::UNIT_FRAME_TRAILER => break,
                0..=0b0011_1111 => {
                    current_region = 3;
                    add_data0(&mut hits, *b, current_region);
                    st.transition(_Data).as_enum()
                }
                _ => unreachable!("Expected Unit Frame Trailer or DATA 0, got: {b:#X}"),
            },
            FRAME_TRAILER_By_FrameTrailer(_) => {
                unreachable!("State machine should have already been used at this point")
            }
        };
    }

    if hits.is_empty() {
        None
    } else {
        Some(hits)
    }
}

#[inline]
fn add_data0(moss_hits: &mut Vec<MossHit>, data0: u8, region: u8) {
    moss_hits.push(MossHit {
        region,                            // region id
        row: ((data0 & 0x3F) as u16) << 3, // row position [8:3]
        column: 0,                         // placeholder
    })
}

#[inline]
fn add_data1(moss_hits: &mut [MossHit], data1: u8) {
    moss_hits
        .last_mut()
        .unwrap() // row position [2:0]
        .row |= ((data1 & 0x38) >> 3) as u16;

    moss_hits
        .last_mut()
        .unwrap() // col position [8:6]
        .column = ((data1 & 0x07) as u16) << 6;
}

#[inline]
fn add_data2(moss_hits: &mut [MossHit], data2: u8) {
    moss_hits.last_mut().unwrap().column |= (data2 & 0x3F) as u16;
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

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

    #[test]
    fn test_fsm() {
        //
        let event_data_packet = fake_event_simple();
        let slice = &event_data_packet;

        let mut byte_iter = slice.iter();
        let byte_count = byte_iter.len();

        let unit_id = loop {
            if let Some(val) = byte_iter.next() {
                if (0xD0..=0xD9).contains(val) {
                    break val & 0xF;
                }
            }
        };

        if let Some(hits) = extract_hits(&mut byte_iter) {
            assert_eq!(unit_id, 0);
            assert_eq!(hits.len(), 4);
            assert_eq!(byte_count - byte_iter.len() - 1, 18);
        } else {
            panic!("Decoding failed")
        }
    }

    #[test]
    fn test_fsm_multiple_events() {
        let mut event_data_packet = fake_event_simple();
        event_data_packet.append(&mut fake_event_simple());

        let slice = &event_data_packet;

        let mut byte_iter = slice.iter();
        let byte_count = byte_iter.len();

        let unit_id = loop {
            if let Some(val) = byte_iter.next() {
                if (0xD0..=0xD9).contains(val) {
                    break val & 0xF;
                }
            }
        };

        if let Some(hits) = extract_hits(&mut byte_iter) {
            assert_eq!(unit_id, 0);
            assert_eq!(hits.len(), 4);
            assert_eq!(byte_count - byte_iter.len() - 1, 18);
        } else {
            panic!("Decoding failed")
        }

        let unit_id = loop {
            if let Some(val) = byte_iter.next() {
                if (0xD0..=0xD9).contains(val) {
                    break val & 0xF;
                }
            }
        };

        if let Some(hits) = extract_hits(&mut byte_iter) {
            assert_eq!(unit_id, 0);
            assert_eq!(hits.len(), 4);
            assert_eq!(byte_count - byte_iter.len() - 1, 37);
        } else {
            panic!("Decoding failed")
        }
    }

    #[test]
    fn test_extract_packet() {
        let packet = fake_event_simple();
        let slice = packet.as_slice();
        let mut packet_iter = slice.iter();
        let p = extract_packet(&mut packet_iter);
        println!("{p:?}");
        assert!(p.is_some());
        assert_eq!(p.unwrap().hits.len(), 4);
    }
}
