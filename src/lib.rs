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
mod debug_decode;
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

type LastTrailerIdx = usize;
type InvalidWordMsgs = Vec<String>;

#[allow(non_camel_case_types)]
type List_MossPackets = Vec<MossPacket>;

#[allow(non_camel_case_types)]
type Tuple_MossPacket_LastTrailerIdx = (MossPacket, LastTrailerIdx);

#[allow(non_camel_case_types)]
type Tuple_List_MossPackets_LastTrailerIdx = (List_MossPackets, LastTrailerIdx);

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
            Err(e) => {
                return Err(PyAssertionError::new_err(format!(
                    "Decoding packet #{packet_cnt} failed with: {e}",
                    packet_cnt = moss_packets.len() + 1
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
/// Decodes as many MOSS events as possible into a list of [MossPacket]s.
/// Doesn't check for invalid state transitions. Runs over errors when possible and instead returns a list of invalid words.
///
/// Useful for attempting to extract as many packets and debug based on packet analysis.
pub fn debug_decode_all_events(
    bytes: &[u8],
) -> PyResult<(List_MossPackets, LastTrailerIdx, InvalidWordMsgs)> {
    let approx_moss_packets = rust_only::calc_prealloc_val(bytes)?;
    let mut moss_packets: Vec<MossPacket> = Vec::with_capacity(approx_moss_packets);

    // To prevent creating so many error message strings that the user runs out of memory we limit the number of errors.
    // If all errors are reported at indexes higher than 1 GiB then the error message size is 76 bytes or more.
    // 14,128,181 error messages can fit in 1 GiB (not counting string size and any extra capacity).
    // That's a very high numbers so let's round down to a more reasonable number and make it one that has a nice binary representation.
    const MAX_REPORT_ERRORS: usize = 0xFFFFF; // 1,048,575 errors

    let mut last_trailer_idx = 0;
    let mut invalid_words = Vec::new();

    while let Ok((new_moss_packet, trailer_idx, new_invalid_words)) =
        debug_decode::debug_decode_event(&bytes[last_trailer_idx..])
    {
        new_invalid_words.into_iter().for_each(|mut invalid_word| {
            invalid_word.set_index_offset(last_trailer_idx);
            invalid_words.push(invalid_word);
        });
        last_trailer_idx += trailer_idx + 1;
        moss_packets.push(new_moss_packet);

        if invalid_words.len() > MAX_REPORT_ERRORS {
            Err(PyAssertionError::new_err(format!(
                "Too many errors to report: {num_errors}",
                num_errors = invalid_words.len()
            )))?;
        }
    }

    if moss_packets.is_empty() {
        Err(PyAssertionError::new_err("No MOSS Packets in events"))
    } else {
        Ok((
            moss_packets,
            last_trailer_idx - 1,
            invalid_words
                .into_iter()
                .map(|invalid_word| invalid_word.to_error_msg())
                .collect(),
        ))
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

        // Extend bytes_to_decode with the new data
        bytes_to_decode.extend_from_slice(&buf[..bytes_read]);

        // Decode the bytes one event at a time until there's no more events to decode
        match rust_only::get_all_packets_from_buf(&bytes_to_decode) {
            Ok((extracted_packets, last_trailer_idx)) => {
                moss_packets.extend(extracted_packets);
                // Remove the processed bytes from bytes_to_decode (it now contains the remaining bytes that could did not form a complete event)
                bytes_to_decode = bytes_to_decode[last_trailer_idx..].to_vec();
            }
            Err((e, failed_packet_num)) if e.kind() == ParseErrorKind::EndOfBufferNoTrailer => {
                return Err(PyBytesWarning::new_err(format!(
                    "Failed decoding packet #{packet_cnt}: {e}",
                    packet_cnt = moss_packets.len() + failed_packet_num
                )));
            }
            Err((e, failed_packet_num)) => {
                return Err(PyAssertionError::new_err(format!(
                    "Failed decoding packet #{packet_cnt}: {e}",
                    packet_cnt = moss_packets.len() + failed_packet_num
                )))
            }
        }
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
                    "Decoding packet #{packet_cnt} failed with: {e}",
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

/// Decodes N events from the given file.
/// Optionally allows for either (not both):
/// - skipping `skip` events before decoding.
/// - prepending `prepend_buffer` to the bytes before decoding.
///
/// Arguments: path: `str`, take: `int`, skip: `Optional[int]`, prepend_buffer: `Optional[bytes]`
/// Returns: `List[MossPacket]`
#[pyfunction]
pub fn decode_n_events_from_file(
    path: std::path::PathBuf,
    take: usize,
    skip: Option<usize>,
    mut prepend_buffer: Option<Vec<u8>>,
) -> PyResult<List_MossPackets> {
    // Skip N events
    if skip.is_some_and(|s| s == 0) {
        return Err(PyValueError::new_err("skip value must be greater than 0"));
    } else if skip.is_some() && prepend_buffer.is_some() {
        return Err(PyValueError::new_err(
            "skip and prepend_buffer cannot be used together",
        ));
    }
    // Open file (get file descriptor)
    let file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(e) => return Err(PyFileNotFoundError::new_err(e.to_string())),
    };

    // Create buffered reader with 1MB capacity to minimize syscalls to read
    let mut reader = std::io::BufReader::with_capacity(READER_BUFFER_CAPACITY, file);

    let mut moss_packets: Vec<MossPacket> = Vec::with_capacity(take);

    let mut buf = vec![0; READER_BUFFER_CAPACITY];
    let mut bytes_to_decode = Vec::with_capacity(READER_BUFFER_CAPACITY);
    if let Some(prepend_buffer) = prepend_buffer.take() {
        bytes_to_decode.extend_from_slice(&prepend_buffer);
    }
    let mut packets_to_skip = skip.unwrap_or(0);

    while let Ok(bytes_read) = reader.read(&mut buf) {
        if bytes_read == 0 {
            break;
        }

        // Extend bytes_to_decode with the new data
        bytes_to_decode.extend_from_slice(&buf[..bytes_read]);

        // Decode the bytes one event at a time until there's no more events to decode
        match rust_only::get_all_packets_from_buf(&bytes_to_decode) {
            Ok((mut extracted_packets, last_trailer_idx)) => {
                if packets_to_skip > 0 {
                    if packets_to_skip > extracted_packets.len() {
                        packets_to_skip -= extracted_packets.len();
                        bytes_to_decode = bytes_to_decode[last_trailer_idx..].to_vec();
                        continue;
                    } else {
                        for _ in 0..packets_to_skip {
                            _ = extracted_packets.remove(0);
                        }
                        packets_to_skip = 0;
                    }
                }
                moss_packets.extend(extracted_packets);
                if moss_packets.len() >= take {
                    break;
                }
                // Remove the processed bytes from bytes_to_decode (it now contains the remaining bytes that could did not form a complete event)
                bytes_to_decode = bytes_to_decode[last_trailer_idx..].to_vec();
            }
            Err((e, failed_packet_num)) if e.kind() == ParseErrorKind::EndOfBufferNoTrailer => {
                return Err(PyBytesWarning::new_err(format!(
                    "Failed decoding packet #{packet_cnt}: {e}",
                    packet_cnt = moss_packets.len() + failed_packet_num
                )));
            }
            Err((e, failed_packet_num)) => {
                return Err(PyAssertionError::new_err(format!(
                    "Failed decoding packet #{packet_cnt}: {e}",
                    packet_cnt = moss_packets.len() + failed_packet_num
                )))
            }
        }
    }

    if moss_packets.is_empty() {
        Err(PyAssertionError::new_err("No MOSS Packets in events"))
    } else if moss_packets.len() < take {
        Err(PyBytesWarning::new_err(format!(
            "Taking {take} events failed, got {decoded_cnt} events",
            decoded_cnt = moss_packets.len()
        )))
    } else {
        moss_packets.truncate(take); // Truncate to the requested number of events
        Ok(moss_packets)
    }
}

/// Decodes all events from the given file, skipping the first `skip` events
///  and returns the remainder bytes if a partial event was found in it.
///
/// Arguments: path: `str`, skip: `Optional[int]`
/// Returns: `Tuple[Optional[List[MossPacket]], Optional[bytes]]`
#[pyfunction]
pub fn skip_n_take_all_from_file(
    path: std::path::PathBuf,
    mut skip: usize,
) -> PyResult<(Option<List_MossPackets>, Option<Remainder_Bytes>)> {
    let mut moss_packets: Vec<MossPacket> = Vec::new();
    let mut remainder: Option<Vec<u8>> = None;
    // Open file (get file descriptor)
    let file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(e) => return Err(PyFileNotFoundError::new_err(e.to_string())),
    };

    // Create buffered reader with 1MB capacity to minimize syscalls to read
    let mut reader = std::io::BufReader::with_capacity(READER_BUFFER_CAPACITY, file);

    let mut buf = vec![0; READER_BUFFER_CAPACITY];
    let mut bytes_to_decode = Vec::with_capacity(READER_BUFFER_CAPACITY);

    while let Ok(bytes_read) = reader.read(&mut buf) {
        if bytes_read == 0 {
            break;
        }

        // Extend bytes_to_decode with the new data
        bytes_to_decode.extend_from_slice(&buf[..bytes_read]);

        // Decode the bytes one event at a time until there's no more events to decode
        match rust_only::get_all_packets_from_buf(&bytes_to_decode) {
            Ok((mut extracted_packets, last_trailer_idx)) => {
                if skip > 0 {
                    if skip > extracted_packets.len() {
                        skip -= extracted_packets.len();
                        bytes_to_decode = bytes_to_decode[last_trailer_idx..].to_vec();
                        continue;
                    } else {
                        for _ in 0..skip {
                            _ = extracted_packets.remove(0);
                        }
                        skip = 0;
                    }
                }
                moss_packets.extend(extracted_packets);
                // Remove the processed bytes from bytes_to_decode (it now contains the remaining bytes that could did not form a complete event)
                bytes_to_decode = bytes_to_decode[last_trailer_idx..].to_vec();
            }
            Err((e, _)) if e.kind() == ParseErrorKind::EndOfBufferNoTrailer => {
                break;
            }
            Err((e, _)) if e.kind() == ParseErrorKind::NoHeaderFound => {
                break;
            }
            Err((e, failed_packet_num)) => {
                return Err(PyAssertionError::new_err(format!(
                    "Failed decoding packet #{packet_cnt}: {e}",
                    packet_cnt = moss_packets.len() + failed_packet_num
                )))
            }
        }
    }

    if !bytes_to_decode.is_empty()
        && bytes_to_decode
            .iter()
            .any(|&b| moss_protocol::MossWord::UNIT_FRAME_HEADER_RANGE.contains(&b))
    {
        remainder = Some(bytes_to_decode.to_vec());
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

    /// If a prepend buffer is given, it is prepended to `bytes` and the packet is extracted from the combined buffer.
    /// If no prepend buffer is given, the packet is extracted from `bytes`.
    #[inline]
    pub(crate) fn extract_packet_from_buf(
        bytes: &[u8],
        prepend_bytes: Option<Vec<u8>>,
    ) -> Result<Tuple_MossPacket_LastTrailerIdx, ParseError> {
        // Collect bytes from `bytes` until a header is seen
        if let Some(mut prepend) = prepend_bytes {
            let prepend_count = prepend.len();
            prepend.extend(
                bytes
                    .iter()
                    .take_while(|b| **b != MossWord::UNIT_FRAME_TRAILER),
            );
            prepend.push(MossWord::UNIT_FRAME_TRAILER); // Add the trailer back since `take_while` is EXCLUSIVE
            extract_packet(&prepend, prepend_count)
        } else {
            extract_packet(bytes, 0)
        }
    }

    /// Advances the iterator until a Unit Frame Header is encountered, saves the unit ID,
    /// and extracts the hits with the [extract_hits] function, before returning a MossPacket if one is found.
    #[inline]
    fn extract_packet(
        bytes: &[u8],
        prepend_byte_cnt: usize,
    ) -> Result<Tuple_MossPacket_LastTrailerIdx, ParseError> {
        // Check that everything before the first header is delimiter bytes
        //
        // Takes bytes while they are equal to the delimiter byte
        // and checks that the first byte that is not equal to the delimiter byte is a valid header byte.
        let header_idx = find_header_index(bytes)?;

        let mut bytes_iter = bytes.iter().skip(header_idx + 1);
        match extract_hits(&mut bytes_iter) {
            Ok(hits) => Ok((
                MossPacket {
                    unit_id: bytes[header_idx] & 0xF,
                    hits,
                },
                bytes.len() - bytes_iter.len() - 1 - prepend_byte_cnt,
            )),
            Err(e) => Err(ParseError::new(
                e.kind(),
                &format_error_msg(e.message(), e.err_index() + 1, &bytes[header_idx..]),
                header_idx + e.err_index() + 1,
            )),
        }
    }

    // Check that everything before the first header is delimiter bytes
    //
    // Takes bytes while they are equal to the delimiter byte
    // and checks that the first byte that is not equal to the delimiter byte is a valid header byte.
    // Allows the first byte to be the trailer byte, e.g. from a previous event.
    #[inline]
    fn find_header_index(bytes: &[u8]) -> Result<usize, ParseError> {
        for (i, &b) in bytes.iter().enumerate() {
            // Allow the first byte to be the trailer byte, e.g. from a previous event.
            if b == MossWord::DELIMITER || (i == 0 && b == MossWord::UNIT_FRAME_TRAILER) {
                continue;
            } else if MossWord::UNIT_FRAME_HEADER_RANGE.contains(&b) {
                return Ok(i);
            } else {
                return Err(ParseError::new(
                    ParseErrorKind::InvalidDelimiter,
                    &format_error_msg("Invalid delimiter", i, bytes),
                    i,
                ));
            }
        }
        let byte_count = bytes.len();
        Err(ParseError::new(
            ParseErrorKind::NoHeaderFound,
            "No Unit Frame Header found",
            byte_count,
        ))
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

    // On error, returns the error and the number of the packet that failed to decode
    // i.e. if 8 packets are decoded successfully and the 9th packet fails, the error will be returned with 9.
    pub(crate) fn get_all_packets_from_buf(
        buf: &[u8],
    ) -> Result<(Vec<MossPacket>, usize), (ParseError, usize)> {
        let prealloc = if buf.len() / 1024 > MIN_PREALLOC {
            buf.len() / 1024
        } else {
            MIN_PREALLOC
        };
        let mut moss_packets = Vec::with_capacity(prealloc);
        let mut last_trailer_idx = 0;
        loop {
            match extract_packet_from_buf(&buf[last_trailer_idx..], None) {
                Ok((moss_packet, trailer_idx)) => {
                    moss_packets.push(moss_packet);
                    last_trailer_idx += trailer_idx + 1;
                }
                Err(e) if e.kind() == ParseErrorKind::EndOfBufferNoTrailer => {
                    if moss_packets.is_empty() {
                        return Err((e, 0));
                    } else {
                        break;
                    }
                }
                Err(e) if e.kind() == ParseErrorKind::NoHeaderFound => {
                    if moss_packets.is_empty() {
                        return Err((e, 0));
                    } else {
                        break;
                    }
                }
                Err(e) => {
                    if moss_packets.is_empty() {
                        return Err((e, 0));
                    } else {
                        // Return the number of the packet that failed to decode
                        Err((e, moss_packets.len() + 1))?
                    }
                }
            }
        }
        Ok((moss_packets, last_trailer_idx))
    }
}
