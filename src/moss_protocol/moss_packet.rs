use pyo3::prelude::*;
use std::fmt::{write, Display};

use super::MossHit;

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

    fn __str__(&self) -> String {
        self.to_string()
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
    use pretty_assertions::assert_eq;

    #[test]
    fn test_moss_packets_iter() {
        let packets = vec![
            MossPacket::default(),
            MossPacket::new(1),
            MossPacket::new(2),
        ];

        packets.into_iter().enumerate().for_each(|(i, p)| {
            println!("{p}");

            assert_eq!(p.unit_id, i as u8);
        });
    }
}
