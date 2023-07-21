use pyo3::prelude::*;
use std::fmt::{write, Display};

#[pyclass(get_all)]
#[derive(Debug, Default, Clone, Copy)]
pub struct MossHit {
    pub region: u8,
    pub row: u8,
    pub column: u8,
}

#[pymethods]
impl MossHit {
    #[new]
    fn new(region: u8, row: u8, column: u8) -> Self {
        Self {
            region,
            row,
            column,
        }
    }
}

impl Display for MossHit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write(
            f,
            format_args!(
                "reg: {reg}, row: {row} col: {col}",
                reg = self.region,
                row = self.row,
                col = self.column,
            ),
        )
    }
}
