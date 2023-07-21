use pyo3::prelude::*;

pub mod moss_protocol;
use moss_protocol::MossHit;

/// A Python module for decoding raw MOSS data in Rust.
#[pymodule]
fn moss_decoder(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(decode_hit, m)?)?;

    m.add_class::<MossHit>()?;

    Ok(())
}

/// Formats the sum of two numbers as string.
#[pyfunction]
fn decode_hit(bytes: Vec<u8>) -> PyResult<MossHit> {
    let mut hits = Vec::new();
    for b in bytes {
        if b == 255 {
            hits.push(MossHit {
                region: 1,
                column: 2,
                row: 3,
            });
        }
    }
    if !hits.is_empty() {
        Ok(*hits.first().take().unwrap())
    } else {
        Ok(MossHit {
            region: 0,
            column: 0,
            row: 0,
        })
    }
}
