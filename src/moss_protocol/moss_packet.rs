//! MOSS packet structure implementation.
use pyo3::{prelude::*, pyclass::CompareOp};
use std::fmt::{write, Display};

use super::MossHit;

#[pyclass(get_all)]
#[derive(Debug, Default, Clone, PartialEq)]
/// A single MOSS packet with the associated [MossHit]s.
pub struct MossPacket {
    /// The unit ID of the packet.
    pub unit_id: u8,
    /// The hits in the packet.
    pub hits: Vec<MossHit>,
}

#[pymethods]
impl MossPacket {
    #[new]
    pub(crate) fn new(unit_id: u8) -> Self {
        Self {
            unit_id,
            hits: Vec::new(),
        }
    }

    fn __repr__(slf: &PyCell<Self>) -> PyResult<String> {
        let class_name: &str = slf.get_type().name()?;
        Ok(format!(
            "{} ({} {:?})",
            class_name,
            slf.borrow().unit_id,
            slf.borrow().hits,
        ))
    }

    fn __str__(&self) -> String {
        self.to_string()
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> bool {
        op.matches(self.unit_id.cmp(&other.unit_id)) && op.matches(self.hits.cmp(&other.hits))
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
