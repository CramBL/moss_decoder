//! Module containing the MOSS readout protocol and basic structures to analyze the data.
pub mod moss_hit;
pub mod moss_packet;
pub use moss_hit::MossHit;
pub use moss_packet::MossPacket;

#[derive(Debug, PartialEq)]
pub(crate) enum MossWord {
    Idle,
    UnitFrameHeader,
    UnitFrameTrailer,
    RegionHeader,
    Data0,
    Data1,
    Data2,
    Delimiter,
    ProtocolError,
}

impl MossWord {
    pub(super) const IDLE: u8 = 0xFF; // 1111_1111 (default)
    pub(super) const UNIT_FRAME_HEADER: u8 = 0b1101_0000; // 1101_<unit_id[3:0]>
    pub(super) const UNIT_FRAME_TRAILER: u8 = 0b1110_0000; // 1110_0000
    pub(super) const REGION_HEADER: u8 = 0b1100_0000; // 1100_00_<region_id[1:0]>
    pub(super) const DATA_0: u8 = 0b0000_0000; // 00_<hit_row_pos[8:3]>
    pub(super) const DATA_1: u8 = 0b0100_0000; // 01_<hit_row_pos[2:0]>_<hit_col_pos[8:6]>
    pub(super) const DATA_2: u8 = 0b1000_0000; // 10_<hit_col_pos[5:0]>
    pub(super) const DELIMITER: u8 = 0xFA; // subject to change (FPGA implementation detail)

    pub fn from_byte(b: u8) -> MossWord {
        match b {
            // Exact matches
            Self::IDLE => MossWord::Idle,
            Self::UNIT_FRAME_TRAILER => MossWord::UnitFrameTrailer,
            six_msb if six_msb & 0xFC == Self::REGION_HEADER => MossWord::RegionHeader,
            four_msb if four_msb & 0xF0 == Self::UNIT_FRAME_HEADER => MossWord::UnitFrameHeader,
            Self::DELIMITER => Self::Delimiter,
            two_msb if two_msb & 0b1100_0000 == Self::DATA_0 => MossWord::Data0,
            two_msb if two_msb & 0b1100_0000 == Self::DATA_1 => MossWord::Data1,
            two_msb if two_msb & 0b1100_0000 == Self::DATA_2 => MossWord::Data2,
            _ => MossWord::ProtocolError,
        }
    }
}
