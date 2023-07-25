//! struct representation of a single hit from a MOSS region.
use pyo3::prelude::*;
use pyo3::pyclass::CompareOp;
use std::fmt::write;
use std::fmt::Display;

#[pyclass(get_all)]
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
/// A single hit from a MOSS region.
pub struct MossHit {
    /// The region ID of the hit.
    pub region: u8,
    /// The row of the hit.
    pub row: u16,
    /// The column of the hit.
    pub column: u16,
}

#[pymethods]
impl MossHit {
    #[new]
    fn new(region: u8, row: u16, column: u16) -> Self {
        Self {
            region,
            row,
            column,
        }
    }

    fn __repr__(slf: &PyCell<Self>) -> PyResult<String> {
        let class_name: &str = slf.get_type().name()?;
        Ok(format!(
            "{}({} {} {})",
            class_name,
            slf.borrow().region,
            slf.borrow().row,
            slf.borrow().column
        ))
    }

    /// Returns a string representation of the [MossHit] instance.
    pub fn __str__(&self) -> String {
        self.to_string()
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> bool {
        op.matches(self.region.cmp(&other.region))
            && op.matches(self.row.cmp(&other.row))
            && op.matches(self.column.cmp(&other.column))
    }
}

impl Display for MossHit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write(
            f,
            format_args!(
                "reg: {reg} row: {row} col: {col}",
                reg = self.region,
                row = self.row,
                col = self.column,
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
        println!("{str}", str = moss_hit.__str__());
    }
}
