//! A Python module for decoding raw MOSS data implemented in Rust.
#![forbid(unused_extern_crates)]
#![deny(missing_docs)]
#![warn(missing_copy_implementations)]
#![warn(trivial_casts, trivial_numeric_casts)]
#![warn(unused_results)]
#![warn(unused_import_braces)]
#![warn(variant_size_differences)]
#![warn(
    clippy::option_filter_map,
    clippy::manual_filter_map,
    clippy::if_not_else,
    clippy::nonminimal_bool
)]
// Performance lints
#![warn(
    clippy::needless_pass_by_value,
    clippy::unnecessary_wraps,
    clippy::mutex_integer,
    clippy::mem_forget,
    clippy::maybe_infinite_iter
)]

use std::io::Read;

pub use moss_protocol::MossPacket;
use pyo3::exceptions::{PyAssertionError, PyFileNotFoundError, PyValueError};
use pyo3::prelude::*;

pub mod moss_protocol;
pub use moss_protocol::MossHit;
pub mod moss_protocol_nested_fsm;

/// A Python module for decoding raw MOSS data in Rust.
#[pymodule]
fn moss_decoder(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(decode_event, m)?)?;
    m.add_function(wrap_pyfunction!(decode_multiple_events, m)?)?;
    m.add_function(wrap_pyfunction!(decode_from_file, m)?)?;

    m.add_class::<MossHit>()?;
    m.add_class::<MossPacket>()?;

    Ok(())
}

const READER_BUFFER_CAPACITY: usize = 10 * 1024 * 1024; // 10 MiB

/// Decodes a single MOSS event into a [MossPacket] and the index of the trailer byte with an FSM based decoder.
/// This function returns an error if no MOSS packet is found, therefor if there's any chance the argument does not contain a valid `MossPacket`
/// the call should be enclosed in a try/catch.
#[pyfunction]
pub fn decode_event(bytes: &[u8]) -> PyResult<(MossPacket, usize)> {
    let byte_cnt = bytes.len();

    if byte_cnt < 6 {
        return Err(PyValueError::new_err(
            "Received less than the minimum event size",
        ));
    }

    match moss_protocol_nested_fsm::extract_packet(bytes) {
        Ok((moss_packet, trailer_idx)) => Ok((moss_packet, trailer_idx)),
        Err(e) => Err(PyAssertionError::new_err(format!(
            "Decoding failed with: {e}",
        ))),
    }
}

#[pyfunction]
/// Decodes multiple MOSS events into a list of [MossPacket]s based on an FSM decoder.
/// This function is optimized for speed and memory usage.
pub fn decode_multiple_events(bytes: &[u8]) -> PyResult<(Vec<MossPacket>, usize)> {
    let approx_moss_packets = rust_only::calc_prealloc_val(bytes)?;

    let mut moss_packets: Vec<MossPacket> = Vec::with_capacity(approx_moss_packets);

    let mut last_trailer_idx = 0;

    while let Ok((moss_packet, trailer_idx)) =
        moss_protocol_nested_fsm::extract_packet(&bytes[last_trailer_idx..])
    {
        moss_packets.push(moss_packet);
        last_trailer_idx += trailer_idx + 1;
    }

    if moss_packets.is_empty() {
        Err(PyAssertionError::new_err("No MOSS Packets in events"))
    } else {
        Ok((moss_packets, last_trailer_idx - 1))
    }
}

#[pyfunction]
/// Decodes a file containing raw MOSS data into a list of [MossPacket]s using an FSM based decoder.
///
/// The file is read in chunks of 10 MiB until the end of the file is reached.
/// If any errors are encountered while reading the file, any successfully decoded events are returned.
/// There's no attempt to run over errors.
pub fn decode_from_file(path: std::path::PathBuf) -> PyResult<Vec<MossPacket>> {
    // Open file (get file descriptor)
    let file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(e) => return Err(PyFileNotFoundError::new_err(e.to_string())),
    };

    // Create buffered reader with 1MB capacity to minimize syscalls to read
    let mut reader = std::io::BufReader::with_capacity(READER_BUFFER_CAPACITY, file);

    let mut moss_packets = Vec::new();

    let mut buf = vec![0; READER_BUFFER_CAPACITY];
    let mut bytes_to_decode = Vec::with_capacity(READER_BUFFER_CAPACITY);

    while let Ok(bytes_read) = reader.read(&mut buf) {
        if bytes_read == 0 {
            break;
        }

        let mut last_trailer_idx = 0;

        // Extend bytes_to_decode with the new data
        bytes_to_decode.extend_from_slice(&buf[..bytes_read]);

        // Decode the bytes one event at a time until there's no more events to decode
        while let Ok((moss_packet, current_trailer_idx)) =
            moss_protocol_nested_fsm::extract_packet(&bytes_to_decode[last_trailer_idx..])
        {
            moss_packets.push(moss_packet);
            last_trailer_idx += current_trailer_idx + 1;
        }

        // Remove the processed bytes from bytes_to_decode (it now contains the remaining bytes that could did not form a complete event)
        bytes_to_decode = bytes_to_decode[last_trailer_idx..].to_vec();
    }

    if moss_packets.is_empty() {
        Err(PyAssertionError::new_err("No MOSS Packets in events"))
    } else {
        Ok(moss_packets)
    }
}

mod rust_only {
    use pyo3::exceptions::PyValueError;
    use pyo3::PyResult;

    /// Functions that are only used in Rust and not exposed to Python.

    const MIN_PREALLOC: usize = 10;
    #[inline]
    pub(super) fn calc_prealloc_val(bytes: &[u8]) -> PyResult<usize> {
        let byte_cnt = bytes.len();

        if byte_cnt < 6 {
            return Err(PyValueError::new_err(
                "Received less than the minimum event size",
            ));
        }

        let prealloc = if byte_cnt / 1024 > MIN_PREALLOC {
            byte_cnt / 1024
        } else {
            MIN_PREALLOC
        };
        Ok(prealloc)
    }
}
