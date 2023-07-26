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
use moss_protocol::MossWord;

/// A Python module for decoding raw MOSS data in Rust.
#[pymodule]
fn moss_decoder(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(decode_event, m)?)?;

    m.add_function(wrap_pyfunction!(decode_event_noexcept, m)?)?;

    m.add_function(wrap_pyfunction!(decode_multiple_events, m)?)?;

    m.add_function(wrap_pyfunction!(decode_from_file, m)?)?;

    m.add_class::<MossHit>()?;
    m.add_class::<MossPacket>()?;

    Ok(())
}

/// Decodes a single MOSS event into a [MossPacket] and the index of the trailer byte
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

    match rust_only::raw_decode_event(bytes) {
        Ok((moss_packet, trailer_idx)) => Ok((moss_packet, trailer_idx)),
        Err(e) => Err(PyAssertionError::new_err(format!(
            "No MOSS packet found: {e}",
        ))),
    }
}

/// Decodes a single MOSS event into a [MossPacket] and the index of the trailer byte.
/// This function does not return an error if no MOSS packet is found, instead the last_trailer_idx is returned as 0.
/// In the case no MOSS packet is found, the a `MossPacket` is still returned with default values which is unit ID 0
/// and no hits, but this result is invalid and should be discarded.
#[pyfunction]
pub fn decode_event_noexcept(bytes: &[u8]) -> (MossPacket, usize) {
    let byte_cnt = bytes.len();

    if byte_cnt < 6 {
        return (MossPacket::default(), 0);
    }

    if let Ok((moss_packet, trailer_idx)) = rust_only::raw_decode_event(bytes) {
        (moss_packet, trailer_idx)
    } else {
        (MossPacket::default(), 0)
    }
}

const MIN_PREALLOC: usize = 10;
/// Decodes multiple MOSS events into a list of [MossPacket]s
/// This function is optimized for speed and memory usage.
#[pyfunction]
pub fn decode_multiple_events(bytes: &[u8]) -> PyResult<(Vec<MossPacket>, usize)> {
    let byte_cnt = bytes.len();

    if byte_cnt < 6 {
        return Err(PyValueError::new_err(
            "Received less than the minimum event size",
        ));
    }

    let approx_moss_packets = if byte_cnt / 1024 > MIN_PREALLOC {
        byte_cnt / 1024
    } else {
        MIN_PREALLOC
    };

    let mut moss_packets: Vec<MossPacket> = Vec::with_capacity(approx_moss_packets);

    let mut last_trailer_idx = 0;

    while let Ok((moss_packet, trailer_idx)) = decode_event(&bytes[last_trailer_idx..]) {
        moss_packets.push(moss_packet);
        last_trailer_idx += trailer_idx + 1;
    }

    if moss_packets.is_empty() {
        Err(PyAssertionError::new_err("No MOSS Packets in events"))
    } else {
        Ok((moss_packets, last_trailer_idx - 1))
    }
}

const READER_BUFFER_CAPACITY: usize = 10 * 1024 * 1024; // 10 MiB

#[pyfunction]
/// Decodes a file containing raw MOSS data into a list of [MossPacket]s
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

    while let Ok(bytes_read) = reader.read(&mut buf[..]) {
        if bytes_read == 0 {
            break;
        }

        // Extend bytes_to_decode with the new data
        bytes_to_decode.extend_from_slice(&buf[..bytes_read]);

        let mut last_trailer_idx = 0;

        // Decode the bytes one event at a time until there's no more events to decode
        while let Ok((moss_packet, current_trailer_idx)) =
            rust_only::raw_decode_event(&bytes_to_decode[last_trailer_idx..])
        {
            moss_packets.push(moss_packet);
            last_trailer_idx += current_trailer_idx + 1; // +1 to account for the trailer byte
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
    /// Functions that are only used in Rust and not exposed to Python.
    use super::MossHit;
    use super::MossPacket;
    use super::MossWord;

    const INVALID_NO_HEADER_SEEN: u8 = 0xFF;
    /// Decodes a single MOSS event into a [MossPacket] and the index of the trailer byte (Rust only)
    pub(crate) fn raw_decode_event(bytes: &[u8]) -> std::io::Result<(MossPacket, usize)> {
        let mut moss_packet = MossPacket {
            unit_id: INVALID_NO_HEADER_SEEN, // placeholder
            hits: Vec::new(),
        };

        let mut trailer_idx = 0;
        let mut current_region: u8 = 0xff; // placeholder

        let mut is_moss_packet = false;
        for (i, byte) in bytes.iter().enumerate() {
            match MossWord::from_byte(*byte) {
                MossWord::Idle => (),
                MossWord::UnitFrameHeader => {
                    debug_assert!(!is_moss_packet);
                    is_moss_packet = true;
                    moss_packet.unit_id = *byte & 0x0F
                }
                MossWord::UnitFrameTrailer => {
                    debug_assert!(
                        is_moss_packet,
                        "Trailer seen before header, next 10 bytes: {:#X?}",
                        &bytes[i..i + 10]
                    );
                    trailer_idx = i;
                    break;
                }
                MossWord::RegionHeader => {
                    debug_assert!(
                    is_moss_packet,
                    "Region header seen before frame header at index {i}, current and next 9 bytes:\n {:#X?}",
                &bytes[i..i + 10]
                );
                    current_region = *byte & 0x03;
                }
                MossWord::Data0 => {
                    debug_assert!(is_moss_packet);
                    moss_packet.hits.push(MossHit {
                        region: current_region,            // region id
                        row: ((*byte & 0x3F) as u16) << 3, // row position [8:3]
                        column: 0,                         // placeholder
                    });
                }
                MossWord::Data1 => {
                    debug_assert!(is_moss_packet);
                    // row position [2:0]
                    moss_packet.hits.last_mut().unwrap().row |= ((*byte & 0x38) >> 3) as u16;
                    // col position [8:6]
                    moss_packet.hits.last_mut().unwrap().column = ((*byte & 0x07) as u16) << 6;
                }
                MossWord::Data2 => {
                    debug_assert!(is_moss_packet);
                    moss_packet.hits.last_mut().unwrap().column |= (*byte & 0x3F) as u16;
                    // col position [5:0]
                }
                MossWord::Delimiter => {
                    debug_assert!(!is_moss_packet);
                }
                MossWord::ProtocolError => {
                    let describe_decode_state = if is_moss_packet {
                        "in MOSS packet"
                    } else {
                        "before header seen"
                    };
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!(
                        "Protocol error {describe_decode_state}, at index {i} with byte {byte:#X} "
                    ),
                    ));
                }
            }
        }
        if moss_packet.unit_id == INVALID_NO_HEADER_SEEN || trailer_idx == 0 {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No MOSS packet found",
            ))
        } else {
            Ok((moss_packet, trailer_idx))
        }
    }
}
