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
use moss_protocol::MossWord;
use pyo3::exceptions::{PyAssertionError, PyFileNotFoundError, PyValueError};
use pyo3::prelude::*;

pub mod moss_protocol;
pub use moss_protocol::MossHit;
pub mod decode_hits_fsm;

/// A Python module for decoding raw MOSS data effeciently in Rust.
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
const MINIMUM_EVENT_SIZE: usize = 6;

/// Decodes a single MOSS event into a [MossPacket] and the index of the trailer byte.
/// This function returns an error if no MOSS packet is found, therefor if there's any chance the argument does not contain a valid `MossPacket`
/// the call should be enclosed in a try/catch.
#[pyfunction]
pub fn decode_event(bytes: &[u8]) -> PyResult<(MossPacket, usize)> {
    let byte_cnt = bytes.len();

    if byte_cnt < MINIMUM_EVENT_SIZE {
        return Err(PyValueError::new_err(
            "Received less than the minimum event size",
        ));
    }

    match rust_only::extract_packet(bytes) {
        Ok((moss_packet, trailer_idx)) => Ok((moss_packet, trailer_idx)),
        Err(e) => Err(PyAssertionError::new_err(format!(
            "Decoding failed with: {e}",
        ))),
    }
}

#[pyfunction]
/// Decodes multiple MOSS events into a list of [MossPacket]s.
/// This function is optimized for speed and memory usage.
pub fn decode_multiple_events(bytes: &[u8]) -> PyResult<(Vec<MossPacket>, usize)> {
    let approx_moss_packets = rust_only::calc_prealloc_val(bytes)?;

    let mut moss_packets: Vec<MossPacket> = Vec::with_capacity(approx_moss_packets);

    let mut last_trailer_idx = 0;

    while last_trailer_idx < bytes.len() - MINIMUM_EVENT_SIZE - 1 {
        match rust_only::extract_packet(&bytes[last_trailer_idx..]) {
            Ok((moss_packet, trailer_idx)) => {
                moss_packets.push(moss_packet);
                last_trailer_idx += trailer_idx + 1;
            }
            Err(e) => {
                return Err(PyAssertionError::new_err(format!(
                    "Decoding failed with: {e}",
                )))
            }
        }
    }

    if moss_packets.is_empty() {
        Err(PyAssertionError::new_err("No MOSS Packets in events"))
    } else {
        Ok((moss_packets, last_trailer_idx - 1))
    }
}

#[pyfunction]
/// Decodes a file containing raw MOSS data into a list of [MossPacket]s.
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
            decode_event(&bytes_to_decode[last_trailer_idx..])
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

#[pyfunction]
/// Skips N events in the given bytes and decodes the next M events.
pub fn decode_events_skip_n_take_m(
    bytes: &[u8],
    skip: usize,
    take: usize,
) -> PyResult<(Vec<MossPacket>, usize)> {
    let mut moss_packets: Vec<MossPacket> = Vec::with_capacity(take);

    let mut last_trailer_idx = 0;

    // Skip N events
    for i in 0..skip {
        if let Some(header_idx) = bytes[last_trailer_idx..]
            .iter()
            .position(|b| MossWord::UNIT_FRAME_HEADER_RANGE.contains(b))
        {
            if let Some(trailer_idx) = &bytes[last_trailer_idx + header_idx..]
                .iter()
                .position(|b| *b == MossWord::UNIT_FRAME_TRAILER)
            {
                last_trailer_idx += header_idx + trailer_idx + 1;
            } else {
                return Err(PyAssertionError::new_err(format!(
                    "No Unit Frame Trailer found for packet {i}",
                )));
            }
        } else {
            return Err(PyAssertionError::new_err(format!(
                "No Unit Frame Header found for packet {i}"
            )));
        }
    }

    for i in 0..take {
        match rust_only::extract_packet(&bytes[last_trailer_idx..]) {
            Ok((moss_packet, trailer_idx)) => {
                moss_packets.push(moss_packet);
                last_trailer_idx += trailer_idx + 1;
            }
            Err(e) => {
                return Err(PyAssertionError::new_err(format!(
                    "Decoding packet {i} failed with: {e}",
                )))
            }
        }
    }

    if moss_packets.is_empty() {
        Err(PyAssertionError::new_err("No MOSS Packets in events"))
    } else {
        Ok((moss_packets, last_trailer_idx - 1))
    }
}

mod rust_only {
    use pyo3::exceptions::PyValueError;
    use pyo3::PyResult;

    use crate::decode_hits_fsm::extract_hits;
    use crate::moss_protocol::MossWord;
    use crate::MossPacket;

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

    /// Advances the iterator until a Unit Frame Header is encountered, saves the unit ID,
    /// and extracts the hits with the [extract_hits] function, before returning a MossPacket if one is found.
    #[inline]
    pub(crate) fn extract_packet(bytes: &[u8]) -> Result<(MossPacket, usize), Box<str>> {
        if let Some(header_idx) = bytes
            .iter()
            .position(|b| MossWord::UNIT_FRAME_HEADER_RANGE.contains(b))
        {
            let mut bytes_iter = bytes.iter().skip(header_idx + 1);
            match extract_hits(&mut bytes_iter) {
                Ok(hits) => Ok((
                    MossPacket {
                        unit_id: bytes[header_idx] & 0xF,
                        hits,
                    },
                    bytes.len() - bytes_iter.len() - 1,
                )),
                Err((err_str, err_idx)) => {
                    Err(format_error_msg(err_str, err_idx + 1, &bytes[header_idx..]).into())
                }
            }
        } else {
            Err("No Unit Frame Header found".into())
        }
    }

    /// Formats an error message with an error description and the byte that triggered the error.
    ///
    /// Also includes a dump of the bytes from the header and 10 bytes past the error.
    fn format_error_msg(err_str: &str, err_idx: usize, bytes: &[u8]) -> String {
        format!(
        "{err_str}, got: 0x{error_byte:02X}. Dump from header and 10 bytes past error: {prev} [ERROR = {error_byte:02X}] {next}",
        prev = bytes
            .iter()
            .take(err_idx)
            .map(|b| format!("{b:02X}"))
            .collect::<Vec<_>>()
            .join(" "),
        error_byte = bytes[err_idx],
        next = bytes
            .iter()
            .skip(err_idx+1)
            .take(10)
            .map(|b| format!("{b:02X}"))
            .collect::<Vec<_>>()
            .join(" ")
    )
    }
}
