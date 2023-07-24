use pyo3::prelude::*;
use std::fmt::write;
use std::fmt::Display;

#[pyclass(get_all)]
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct MossHit {
    pub region: u8,
    pub row: u16,
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
