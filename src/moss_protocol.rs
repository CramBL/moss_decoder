use pyo3::prelude::*;
use std::fmt::{write, Display};

pub use moss_hit::MossHit;

pub mod moss_hit;

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

    pub fn from_byte(b: u8) -> Result<MossWord, ()> {
        match b {
            // Exact matches
            Self::IDLE => Ok(Self::Idle),
            Self::UNIT_FRAME_TRAILER => Ok(Self::UnitFrameTrailer),
            six_msb if six_msb & 0b1111_1100 == Self::REGION_HEADER => Ok(Self::RegionHeader),
            four_msb if four_msb & 0b1111_0000 == Self::UNIT_FRAME_HEADER => {
                Ok(Self::UnitFrameHeader)
            }
            Self::DELIMITER => Ok(Self::Delimiter),
            two_msb if two_msb & 0b1100_0000 == Self::DATA_0 => Ok(Self::Data0),
            two_msb if two_msb & 0b1100_0000 == Self::DATA_1 => Ok(Self::Data1),
            two_msb if two_msb & 0b1100_0000 == Self::DATA_2 => Ok(Self::Data2),
            val => unreachable!("Unreachable: {val}"),
        }
    }
}

#[pyclass(get_all)]
#[derive(Debug, Default, Clone, PartialEq)]
pub struct MossPacket {
    pub unit_id: u8,
    pub hits: Vec<MossHit>,
}

#[pymethods]
impl MossPacket {
    #[new]
    fn new(unit_id: u8) -> Self {
        Self {
            unit_id,
            hits: Vec::new(),
        }
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }
}

impl Display for MossPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write(
            f,
            format_args!(
                "Unit ID: {id} Hits: {cnt}\n {hits:?}",
                id = self.unit_id,
                cnt = self.hits.len(),
                hits = self.hits
            ),
        )
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_moss_hit() {
        let moss_hit = MossHit::default();

        println!("{moss_hit}");
        println!("{str}", str = moss_hit.__str__().unwrap());
    }

    #[test]
    fn test_moss_packets_iter() {
        let packets = vec![
            MossPacket::default(),
            MossPacket::new(1),
            MossPacket::new(2),
        ];

        packets.into_iter().for_each(|p| println!("{p}"));
    }
}
