pub mod moss_hit;
pub mod moss_packet;
pub use moss_hit::MossHit;
pub use moss_packet::MossPacket;

pub(crate) enum MossWord {
    Idle,
    UnitFrameHeader,
    UnitFrameTrailer,
    RegionHeader,
    Data0,
    Data1,
    Data2,
    Delimiter,
}

impl MossWord {
    const IDLE: u8 = 0xFF; // 1111_1111 (default)
    const UNIT_FRAME_HEADER: u8 = 0b1101_0000; // 1101_<unit_id[3:0]>
    const UNIT_FRAME_TRAILER: u8 = 0b1110_0000; // 1110_0000
    const REGION_HEADER: u8 = 0b1100_0000; // 1100_00_<region_id[1:0]>
    const DATA_0: u8 = 0b0000_0000; // 00_<hit_row_pos[8:3]>
    const DATA_1: u8 = 0b0100_0000; // 01_<hit_row_pos[2:0]>_<hit_col_pos[8:6]>
    const DATA_2: u8 = 0b1000_0000; // 10_<hit_col_pos[5:0]>
    const DELIMITER: u8 = 0xFA; // subject to change (FPGA implementation detail)

    pub fn from_byte(b: u8) -> MossWord {
        match b {
            // Exact matches
            Self::IDLE => Self::Idle,
            Self::UNIT_FRAME_TRAILER => Self::UnitFrameTrailer,
            six_msb if six_msb & 0b1111_1100 == Self::REGION_HEADER => Self::RegionHeader,
            four_msb if four_msb & 0b1111_0000 == Self::UNIT_FRAME_HEADER => Self::UnitFrameHeader,
            Self::DELIMITER => Self::Delimiter,
            two_msb if two_msb & 0b1100_0000 == Self::DATA_0 => Self::Data0,
            two_msb if two_msb & 0b1100_0000 == Self::DATA_1 => Self::Data1,
            two_msb if two_msb & 0b1100_0000 == Self::DATA_2 => Self::Data2,
            val => unreachable!("Unreachable: {val}"),
        }
    }
}
