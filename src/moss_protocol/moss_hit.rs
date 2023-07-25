//! struct representation of a single hit from a MOSS region.
use pyo3::prelude::*;
use std::fmt::write;
use std::fmt::Display;

#[pyclass(get_all)]
#[derive(Debug, Default, Clone, Copy, PartialEq)]
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

    /// Returns a string representation of the [MossHit] instance.
    pub fn __str__(&self) -> String {
        self.to_string()
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
