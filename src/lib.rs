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

pub use moss_protocol::MossPacket;
use parse_error::ParseErrorKind;
use parse_util::find_trailer_n_idx;
use pyo3::exceptions::{PyAssertionError, PyBytesWarning, PyFileNotFoundError, PyValueError};
use pyo3::prelude::*;
use std::io::Read;

pub mod moss_protocol;
pub use moss_protocol::MossHit;
pub mod decode_hits_fsm;
pub(crate) mod parse_error;
pub(crate) mod parse_util;

/// A Python module for decoding raw MOSS data effeciently in Rust.
#[pymodule]
fn moss_decoder(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(decode_event, m)?)?;
    m.add_function(wrap_pyfunction!(decode_all_events, m)?)?;
    m.add_function(wrap_pyfunction!(decode_from_file, m)?)?;
    m.add_function(wrap_pyfunction!(decode_n_events, m)?)?;
    m.add_function(wrap_pyfunction!(skip_n_take_all, m)?)?;

    m.add_class::<MossHit>()?;
    m.add_class::<MossPacket>()?;

    Ok(())
}

#[allow(non_camel_case_types)]
type List_MossPackets = Vec<MossPacket>;

#[allow(non_camel_case_types)]
type Tuple_MossPacket_LastTrailerIdx = (MossPacket, usize);

#[allow(non_camel_case_types)]
type Tuple_List_MossPackets_LastTrailerIdx = (List_MossPackets, usize);

const READER_BUFFER_CAPACITY: usize = 10 * 1024 * 1024; // 10 MiB
const MINIMUM_EVENT_SIZE: usize = 2;

/// Decodes a single MOSS event into a [MossPacket] and the index of the trailer byte.
/// This function returns an error if no MOSS packet is found, therefor if there's any chance the argument does not contain a valid `MossPacket`
/// the call should be enclosed in a try/except.
#[pyfunction]
pub fn decode_event(bytes: &[u8]) -> PyResult<Tuple_MossPacket_LastTrailerIdx> {
    let byte_cnt = bytes.len();

    if byte_cnt < MINIMUM_EVENT_SIZE {
        return Err(PyValueError::new_err(
            "Received less than the minimum event size",
        ));
    }

    match rust_only::extract_packet_from_buf(bytes, None) {
        Ok((moss_packet, trailer_idx)) => Ok((moss_packet, trailer_idx)),
        Err(e) => Err(PyAssertionError::new_err(format!("Decoding failed: {e}",))),
    }
}

#[pyfunction]
/// Decodes as many MOSS events as possible into a list of [MossPacket]s.
/// Optimized for speed and memory usage.
pub fn decode_all_events(bytes: &[u8]) -> PyResult<Tuple_List_MossPackets_LastTrailerIdx> {
    let approx_moss_packets = rust_only::calc_prealloc_val(bytes)?;

    let mut moss_packets: Vec<MossPacket> = Vec::with_capacity(approx_moss_packets);

    let mut last_trailer_idx = 0;

    while last_trailer_idx < bytes.len() - MINIMUM_EVENT_SIZE - 1 {
        match rust_only::extract_packet_from_buf(&bytes[last_trailer_idx..], None) {
            Ok((moss_packet, trailer_idx)) => {
                moss_packets.push(moss_packet);
                last_trailer_idx += trailer_idx + 1;
            }
            Err(e) if e.kind() == ParseErrorKind::EndOfBufferNoTrailer => {
                return Err(PyBytesWarning::new_err(format!(
                    "Failed decoding packet #{packet_cnt}: {e}",
                    packet_cnt = moss_packets.len() + 1
                )));
            }
            Err(e) => return Err(PyAssertionError::new_err(format!("Decoding failed: {e}",))),
        }
    }

    if moss_packets.is_empty() {
        Err(PyAssertionError::new_err("No MOSS Packets in events"))
    } else {
        Ok((moss_packets, last_trailer_idx - 1))
    }
}

/// Decodes a file containing raw MOSS data into a list of [MossPacket]s.
///
/// The file is read in chunks of 10 MiB until the end of the file is reached.
/// If any errors are encountered while reading the file, an exception is thrown.
/// There's no attempt to run over errors.
#[pyfunction]
pub fn decode_from_file(path: std::path::PathBuf) -> PyResult<List_MossPackets> {
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
        loop {
            match rust_only::extract_packet_from_buf(&bytes_to_decode[last_trailer_idx..], None) {
                Ok((moss_packet, trailer_idx)) => {
                    moss_packets.push(moss_packet);
                    last_trailer_idx += trailer_idx + 1;
                }
                Err(e) if e.kind() == ParseErrorKind::EndOfBufferNoTrailer => {
                    break;
                }
                Err(e) if e.kind() == ParseErrorKind::NoHeaderFound => {
                    if moss_packets.is_empty() {
                        return Err(PyAssertionError::new_err(format!("Decoding failed: {e}")));
                    } else {
                        break;
                    }
                }
                Err(e) => return Err(PyAssertionError::new_err(format!("Decoding failed: {e}",))),
            }
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

/// Decodes N events from the given bytes.
/// Optionally allows for either (not both):
/// - skipping `skip` events before decoding.
/// - prepending `prepend_buffer` to the bytes before decoding.
#[pyfunction]
pub fn decode_n_events(
    bytes: &[u8],
    take: usize,
    skip: Option<usize>,
    mut prepend_buffer: Option<Vec<u8>>,
) -> PyResult<Tuple_List_MossPackets_LastTrailerIdx> {
    let mut moss_packets: Vec<MossPacket> = Vec::with_capacity(take);

    // Skip N events
    if skip.is_some_and(|s| s == 0) {
        return Err(PyValueError::new_err("skip value must be greater than 0"));
    } else if skip.is_some() && prepend_buffer.is_some() {
        return Err(PyValueError::new_err(
            "skip and prepend_buffer cannot be used together",
        ));
    }

    let mut last_trailer_idx = if let Some(skip) = skip {
        find_trailer_n_idx(bytes, skip)?
    } else {
        0
    };

    for i in 0..take {
        match rust_only::extract_packet_from_buf(&bytes[last_trailer_idx..], prepend_buffer.take())
        {
            Ok((moss_packet, trailer_idx)) => {
                moss_packets.push(moss_packet);
                last_trailer_idx += trailer_idx + 1;
            }
            Err(e) if e.kind() == ParseErrorKind::EndOfBufferNoTrailer => {
                return Err(PyBytesWarning::new_err(format!(
                    "Failed decoding packet #{packet_cnt}: {e}",
                    packet_cnt = moss_packets.len() + 1
                )))
            }
            Err(e) => {
                return Err(PyAssertionError::new_err(format!(
                    "Decoding packet {packet_cnt} failed with: {e}",
                    packet_cnt = i + 1
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

#[allow(non_camel_case_types)]
type Remainder_Bytes = Vec<u8>;

/// Skips N events in the given bytes and decode as many packets as possible until end of buffer,
/// If any packets are decoded, they are returned as a list of MOSS Packets.
/// if the end of the buffer contains a partial event, those bytes are returned as a remainder.
///
/// Arguments: bytes: `bytes`, skip: `int`
///
/// Returns: `Tuple[Optional[List[MossPacket]], Optional[bytes]]`
#[pyfunction]
pub fn skip_n_take_all(
    bytes: &[u8],
    skip: usize,
) -> PyResult<(Option<List_MossPackets>, Option<Remainder_Bytes>)> {
    let mut moss_packets: Vec<MossPacket> = Vec::new();
    let mut remainder: Option<Vec<u8>> = None;

    // Skip N events
    let mut last_trailer_idx = if skip > 0 {
        find_trailer_n_idx(bytes, skip)?
    } else {
        0
    };

    while last_trailer_idx < bytes.len() - MINIMUM_EVENT_SIZE - 1 {
        match rust_only::extract_packet_from_buf(&bytes[last_trailer_idx..], None) {
            Ok((moss_packet, trailer_idx)) => {
                moss_packets.push(moss_packet);
                last_trailer_idx += trailer_idx + 1;
            }
            Err(e) if e.kind() == ParseErrorKind::EndOfBufferNoTrailer => {
                remainder = Some(bytes[last_trailer_idx..].to_vec());
                break;
            }
            Err(e) => {
                return Err(PyAssertionError::new_err(format!(
                    "Decoding packet {packet_cnt} failed with: {e}",
                    packet_cnt = moss_packets.len() + 1
                )))
            }
        }
    }

    if moss_packets.is_empty() {
        Ok((None, remainder))
    } else {
        Ok((Some(moss_packets), remainder))
    }
}

mod rust_only {
    use pyo3::exceptions::PyValueError;
    use pyo3::PyResult;

    use crate::decode_hits_fsm::extract_hits;
    use crate::moss_protocol::MossWord;
    use crate::parse_error::{ParseError, ParseErrorKind};
    use crate::{MossPacket, Tuple_MossPacket_LastTrailerIdx};

    /// Functions that are only used in Rust and not exposed to Python.

    const MIN_PREALLOC: usize = 10;
    #[inline]
    pub(super) fn calc_prealloc_val(bytes: &[u8]) -> PyResult<usize> {
        let byte_cnt = bytes.len();

        if byte_cnt < crate::MINIMUM_EVENT_SIZE {
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
    fn extract_packet(bytes: &[u8]) -> Result<Tuple_MossPacket_LastTrailerIdx, ParseError> {
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
                Err(e) => Err(ParseError::new(
                    e.kind(),
                    &format_error_msg(e.message(), e.err_index() + 1, &bytes[header_idx..]),
                    header_idx + e.err_index() + 1,
                )),
            }
        } else {
            Err(ParseError::new(
                ParseErrorKind::NoHeaderFound,
                "No Unit Frame Header found",
                0,
            ))
        }
    }

    #[inline]
    /// If a prepend buffer is given, it is prepended to `bytes` and the packet is extracted from the combined buffer.
    /// If no prepend buffer is given, the packet is extracted from `bytes`.
    pub(crate) fn extract_packet_from_buf(
        bytes: &[u8],
        prepend_bytes: Option<Vec<u8>>,
    ) -> Result<Tuple_MossPacket_LastTrailerIdx, ParseError> {
        // Collect bytes from `bytes` until a header is seen

        if let Some(mut prepend) = prepend_bytes {
            prepend.extend(
                bytes
                    .iter()
                    .take_while(|b| **b != MossWord::UNIT_FRAME_TRAILER),
            );
            prepend.push(MossWord::UNIT_FRAME_TRAILER); // Add the trailer back since `take_while` is EXCLUSIVE
            extract_packet(&prepend)
        } else {
            extract_packet(bytes)
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
            .join(" "))
    }
}
