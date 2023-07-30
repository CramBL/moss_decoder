#![allow(non_camel_case_types)]
#![allow(dead_code)]
//! Contains an FSM implementation of the MOSS data readout protocol
sm::sm! {

    MossReadoutFSM {

        InitialStates { _AWAITING_ }

        _WasFrameHeader {
            _AWAITING_ => FRAME_HEADER_
            FRAME_TRAILER_ => FRAME_HEADER_
        }

        _WasRegionHeader {
            FRAME_HEADER_ => REGION_HEADER_
            REGION_HEADER_ => REGION_HEADER_
            DATA2_ => REGION_HEADER_
        }

        _WasData0 {
            REGION_HEADER_ => DATA0_
            IDLE_ => DATA0_
            DATA2_ => DATA0_
        }

        _WasData1 {
            DATA0_ => DATA1_
        }

        _WasData2 {
            DATA1_ => DATA2_
        }

        _WasIdle {
            DATA2_ => IDLE_
        }

        _WasFrameTrailer {
            REGION_HEADER_ => FRAME_TRAILER_
            DATA2_ => FRAME_TRAILER_
            IDLE_ => FRAME_TRAILER_
        }

        _WasDelimiter {
            FRAME_TRAILER_ => _AWAITING_
            _AWAITING_ => _AWAITING_
        }

    }
}
use MossReadoutFSM::Variant::*;
use MossReadoutFSM::*;

use crate::moss_protocol::MossWord;
use crate::{MossHit, MossPacket};

/// Struct that follows the FSM and maintains the state
pub struct MossFsm {
    state_machine: MossReadoutFSM::Variant,
}

impl Default for MossFsm {
    fn default() -> Self {
        Self::new()
    }
}

impl MossFsm {
    /// Create a new state machine in the initial state.
    #[inline]
    pub(crate) fn new() -> Self {
        Self {
            state_machine: MossReadoutFSM::Machine::new(_AWAITING_).as_enum(),
        }
    }

    #[inline]
    pub(crate) fn reset(&mut self) {
        self.state_machine = MossReadoutFSM::Machine::new(_AWAITING_).as_enum()
    }

    #[inline]
    pub(crate) fn advance(&mut self, byte: u8) -> MossWord {
        let (current_word, next_state): (MossWord, Variant) = match self.state_machine.clone() {
            Initial_AWAITING_(st) => match byte {
                MossWord::DELIMITER => {
                    (MossWord::Delimiter, st.transition(_WasDelimiter).as_enum())
                }
                0xD0..=0xD9 => (
                    MossWord::UnitFrameHeader,
                    st.transition(_WasFrameHeader).as_enum(),
                ),
                _ => unreachable!(
                    "In initial state, expected Delimiter or Unit Frame Header, got: {byte:#X}"
                ),
            },
            FRAME_HEADER_By_WasFrameHeader(st) => (
                MossWord::RegionHeader,
                st.transition(_WasRegionHeader).as_enum(),
            ),
            REGION_HEADER_By_WasRegionHeader(st) => match byte {
                0xC1..=0xC3 => (
                    MossWord::RegionHeader,
                    st.transition(_WasRegionHeader).as_enum(),
                ),
                0..=0b0011_1111 => (MossWord::Data0, st.transition(_WasData0).as_enum()),
                MossWord::UNIT_FRAME_TRAILER => (
                    MossWord::UnitFrameTrailer,
                    st.transition(_WasFrameTrailer).as_enum(),
                ),
                _ => unreachable!(
                    "Expected Region Header, DATA 0, or Unit Frame Trailer, got: {byte:#X}"
                ),
            },
            DATA0_By_WasData0(st) => (MossWord::Data1, st.transition(_WasData1).as_enum()),
            DATA1_By_WasData1(st) => (MossWord::Data2, st.transition(_WasData2).as_enum()),
            DATA2_By_WasData2(st) => match byte {
                MossWord::IDLE => (MossWord::Idle, st.transition(_WasIdle).as_enum()),
                0..=0b0011_1111 => (MossWord::Data0, st.transition(_WasData0).as_enum()),
                0xC1..=0xC3 => (
                    MossWord::RegionHeader,
                    st.transition(_WasRegionHeader).as_enum(),
                ),
                MossWord::UNIT_FRAME_TRAILER => (
                    MossWord::UnitFrameTrailer,
                    st.transition(_WasFrameTrailer).as_enum(),
                ),
                _ => unreachable!(
                    "Expected Region Header, DATA 0, or Unit Frame Trailer got: {byte:#X}"
                ),
            },
            IDLE_By_WasIdle(st) => match byte {
                0..=0b0011_1111 => (MossWord::Data0, st.transition(_WasData0).as_enum()),
                MossWord::UNIT_FRAME_TRAILER => (
                    MossWord::UnitFrameTrailer,
                    st.transition(_WasFrameTrailer).as_enum(),
                ),
                _ => unreachable!(
                    "Expected Region Header, DATA 0, or Unit Frame Trailer got: {byte:#X}"
                ),
            },
            FRAME_TRAILER_By_WasFrameTrailer(st) => match byte {
                MossWord::DELIMITER => {
                    (MossWord::Delimiter, st.transition(_WasDelimiter).as_enum())
                }
                0xD0..=0xD9 => (
                    MossWord::UnitFrameHeader,
                    st.transition(_WasFrameHeader).as_enum(),
                ),
                _ => unreachable!("Expected Delimiter or Unit Frame Header, got: {byte:#X}"),
            },
            _AWAITING_By_WasDelimiter(st) => match byte {
                MossWord::DELIMITER => {
                    (MossWord::Delimiter, st.transition(_WasDelimiter).as_enum())
                }
                0xD0..=0xD9 => (
                    MossWord::UnitFrameHeader,
                    st.transition(_WasFrameHeader).as_enum(),
                ),
                _ => unreachable!("Expected Delimiter or Unit Frame Header got: {byte:#X}"),
            },
        };

        self.state_machine = next_state;
        current_word
    }
}

/// Convenience function that takes a slice of [MossPacket]s, a DATA_0 byte and the current region ID and adds a new [MossHit] to the slice of [MossPacket]s
#[inline]
pub fn add_data0(moss_packets: &mut [MossPacket], data0: u8, current_region: u8) {
    moss_packets.last_mut().unwrap().hits.push(MossHit {
        region: current_region,            // region id
        row: ((data0 & 0x3F) as u16) << 3, // row position [8:3]
        column: 0,                         // placeholder
    })
}

/// Convenience function that takes a slice of [MossPacket]s and a DATA_1 byte and adds the information it contains to the last hit
#[inline]
pub fn add_data1(moss_packets: &mut [MossPacket], data1: u8) {
    moss_packets
        .last_mut()
        .unwrap()
        .hits
        .last_mut()
        .unwrap() // row position [2:0]
        .row |= ((data1 & 0x38) >> 3) as u16;

    moss_packets
        .last_mut()
        .unwrap()
        .hits
        .last_mut()
        .unwrap() // col position [8:6]
        .column = ((data1 & 0x07) as u16) << 6;
}

/// Convenience function that takes a slice of [MossPacket]s and a DATA_2 byte and adds the information it contains to the last hit
#[inline]
pub fn add_data2(moss_packets: &mut [MossPacket], data2: u8) {
    moss_packets
        .last_mut()
        .unwrap()
        .hits
        .last_mut()
        .unwrap()
        .column |= (data2 & 0x3F) as u16;
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::MossPacket;

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
        let mut event_data_packet = fake_event_simple();
        event_data_packet.append(&mut fake_event_simple());
        let mut moss_fsm = MossFsm::new();

        let mut moss_packets = Vec::new();
        let mut current_region: u8 = 0xFF; // Placeholder
        for byte in event_data_packet {
            match moss_fsm.advance(byte) {
                MossWord::Idle => println!("IDLE"),
                MossWord::UnitFrameHeader => {
                    println!("UNIT FRAME HEADER");
                    moss_packets.push(MossPacket::new(byte & 0x0F));
                }
                MossWord::UnitFrameTrailer => {
                    println!("UNIT FRAME TRAILER");
                }
                MossWord::RegionHeader => {
                    println!("REGION HEADER");
                    current_region = byte & 0x03;
                }
                MossWord::Data0 => {
                    println!("DATA 0");
                    add_data0(&mut moss_packets, byte, current_region)
                }
                MossWord::Data1 => {
                    println!("DATA 1");
                    add_data1(&mut moss_packets, byte);
                }
                MossWord::Data2 => {
                    println!("DATA 2");
                    add_data2(&mut moss_packets, byte)
                }
                MossWord::Delimiter => println!("DELIMITER"),
                MossWord::ProtocolError => println!("PROTOCOL ERROR"),
            }
        }

        println!("{:?}", moss_packets);
        assert_eq!(2, moss_packets.len());
    }

    #[test]
    fn test_fsm_read_file_decode() {
        let time = std::time::Instant::now();

        println!("Reading file...");
        let f = std::fs::read(std::path::PathBuf::from("tests/moss_noise.raw")).unwrap();
        println!(
            "Read file in: {t:?}. Bytes: {cnt}",
            t = time.elapsed(),
            cnt = f.len()
        );

        println!("Decoding content...");
        let (p, last_trailer_idx) = crate::slower_impls::decode_multiple_events_fsm(&f).unwrap();
        println!("Decoded in: {t:?}\n", t = time.elapsed());

        println!("Got: {packets} packets", packets = p.len());
        println!("Last trailer at index: {last_trailer_idx}");

        assert_eq!(
            last_trailer_idx,
            f.len() - 2,
            "All bytes were not processed!"
        );
        assert_eq!(p.len(), 100000, "Expected 100k packets, got {}", p.len());
    }
}
