//! Contains an FSM implementation of the MOSS data readout protocol
#![allow(non_camel_case_types)]

use crate::moss_protocol::MossWord;
use crate::MossHit;

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
pub(crate) fn extract_hits<'a>(
    bytes: &mut (impl Iterator<Item = &'a u8>
              + std::iter::DoubleEndedIterator
              + std::iter::ExactSizeIterator),
) -> Result<Vec<MossHit>, (&str, usize)> {
    let mut sm = MossDataFSM::Machine::new(_REGION_HEADER0_).as_enum();
    let mut hits = Vec::<MossHit>::new();

    let mut current_region = 0xff;

    for (i, b) in bytes.enumerate() {
        sm = match sm {
            Initial_REGION_HEADER0_(st) => match *b {
                REGION_HEADER0 => st.transition(_RegionHeader0).as_enum(),
                _ => return Err(("Expected REGION_HEADER_1", i)),
            },
            _REGION_HEADER0_By_RegionHeader0(st) => match *b {
                REGION_HEADER1 => {
                    current_region = 1;
                    st.transition(_RegionHeader1).as_enum()
                }
                b if MossWord::DATA_0_RANGE.contains(&b) => {
                    current_region = 0;
                    add_data0(&mut hits, b, current_region);
                    st.transition(_Data).as_enum()
                }
                _ => return Err(("Expected REGION_HEADER_1/DATA_0", i)),
            },
            DATA0_By_Data(st) => {
                if MossWord::DATA_1_RANGE.contains(b) {
                    add_data1(&mut hits, *b);
                    st.transition(_Data).as_enum()
                } else {
                    return Err(("Expected DATA_1", i));
                }
            }
            DATA1_By_Data(st) => {
                if MossWord::DATA_2_RANGE.contains(b) {
                    add_data2(&mut hits, *b);
                    st.transition(_Data).as_enum()
                } else {
                    return Err(("Expected DATA_2", i));
                }
            }
            DATA2_By_Data(st) => match *b {
                b if MossWord::DATA_0_RANGE.contains(&b) => {
                    add_data0(&mut hits, b, current_region);
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
                _ => return Err(("Expected REGION_HEADER_{1-3}/DATA_0/IDLE", i)),
            },
            IDLE_By_Idle(st) => match *b {
                b if MossWord::DATA_0_RANGE.contains(&b) => {
                    add_data0(&mut hits, b, 0);
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
                _ => return Err(("Expected REGION_HEADER_{1-3}/DATA_0/IDLE", i)),
            },
            REGION_HEADER1_By_RegionHeader1(st) => match *b {
                REGION_HEADER2 => {
                    current_region = 2;
                    st.transition(_RegionHeader2).as_enum()
                }
                b if MossWord::DATA_0_RANGE.contains(&b) => {
                    current_region = 1;
                    add_data0(&mut hits, b, current_region);
                    st.transition(_Data).as_enum()
                }
                _ => return Err(("Expected REGION_HEADER_2/DATA_0", i)),
            },
            REGION_HEADER2_By_RegionHeader2(st) => match *b {
                REGION_HEADER3 => {
                    current_region = 3;
                    st.transition(_RegionHeader3).as_enum()
                }
                b if MossWord::DATA_0_RANGE.contains(&b) => {
                    current_region = 2;
                    add_data0(&mut hits, b, current_region);
                    st.transition(_Data).as_enum()
                }
                _ => return Err(("Expected REGION_HEADER_3/DATA_0", i)),
            },
            REGION_HEADER3_By_RegionHeader3(st) => match *b {
                MossWord::UNIT_FRAME_TRAILER => break,
                b if MossWord::DATA_0_RANGE.contains(&b) => {
                    current_region = 3;
                    add_data0(&mut hits, b, current_region);
                    st.transition(_Data).as_enum()
                }
                _ => return Err(("Expected UNIT_FRAME_TRAILER/DATA_0", i)),
            },
            FRAME_TRAILER_By_FrameTrailer(_) => {
                unreachable!("State machine should have already been used at this point")
            }
        };
    }

    if hits.is_empty() {
        Ok(Vec::with_capacity(0))
    } else {
        Ok(hits)
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
    use super::*;
    use crate::moss_protocol::test_util::*;
    use crate::rust_only::extract_packet;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_fsm() {
        //
        let event_data_packet = fake_event_simple();
        let slice = &event_data_packet;

        let mut byte_iter = slice.iter();
        let byte_count = byte_iter.len();

        let unit_id = loop {
            if let Some(val) = byte_iter.next() {
                if MossWord::UNIT_FRAME_HEADER_RANGE.contains(val) {
                    break val & 0xF;
                }
            }
        };

        if let Ok(hits) = extract_hits(&mut byte_iter) {
            assert_eq!(unit_id, 1);
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
                if MossWord::UNIT_FRAME_HEADER_RANGE.contains(val) {
                    break val & 0xF;
                }
            }
        };

        if let Ok(hits) = extract_hits(&mut byte_iter) {
            assert_eq!(unit_id, 1);
            assert_eq!(hits.len(), 4);
            assert_eq!(byte_count - byte_iter.len() - 1, 18);
        } else {
            panic!("Decoding failed")
        }

        let unit_id = loop {
            if let Some(val) = byte_iter.next() {
                if MossWord::UNIT_FRAME_HEADER_RANGE.contains(val) {
                    break val & 0xF;
                }
            }
        };

        if let Ok(hits) = extract_hits(&mut byte_iter) {
            assert_eq!(unit_id, 1);
            assert_eq!(hits.len(), 4);
            assert_eq!(byte_count - byte_iter.len() - 1, 37);
        } else {
            panic!("Decoding failed")
        }
    }

    #[test]
    fn test_extract_packet() {
        let packet = fake_event_simple();
        let p = extract_packet(&packet);
        println!("{p:?}");
        assert!(p.is_ok());
        let (p, trailer_idx) = p.unwrap();
        assert_eq!(p.hits.len(), 4);
        assert_eq!(trailer_idx, 18);
    }

    #[test]
    fn test_protocol_error() {
        let packet = fake_event_protocol_error();

        if let Err(e) = extract_packet(&packet) {
            println!("{e:?}");
        } else {
            panic!("Expected error, got OK")
        }
    }
}
