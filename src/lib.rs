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
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;

pub mod moss_protocol;
pub use moss_protocol::MossHit;
use moss_protocol::MossWord;

/// A Python module for decoding raw MOSS data in Rust.
#[pymodule]
fn moss_decoder(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(decode_event, m)?)?;
    m.add_function(wrap_pyfunction!(decode_multiple_events, m)?)?;
    m.add_function(wrap_pyfunction!(decode_multiple_events, m)?)?;

    m.add_class::<MossHit>()?;
    m.add_class::<MossPacket>()?;

    Ok(())
}

const INVALID_NO_HEADER_SEEN: u8 = 0xFF;
/// Decodes a single MOSS event into a [MossPacket]

const MIN_PREALLOC: usize = 10;

/// Decodes multiple MOSS events into a list of [MossPacket]s
#[pyfunction]
pub fn decode_multiple_events(bytes: &[u8]) -> PyResult<(Vec<MossPacket>, usize)> {
    let byte_cnt = bytes.len();

    if byte_cnt < 6 {
        return Err(PyTypeError::new_err(
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
        Err(PyTypeError::new_err("No MOSS Packets in events"))
    } else {
        Ok((moss_packets, last_trailer_idx - 1))
    }
}

#[pyfunction]
pub fn decode_event(bytes: &[u8]) -> PyResult<(MossPacket, usize)> {
    let byte_cnt = bytes.len();

    if byte_cnt < 6 {
        return Err(PyTypeError::new_err(
            "Received less than the minimum event size",
        ));
    }

    let mut moss_packet = MossPacket {
        unit_id: INVALID_NO_HEADER_SEEN, // placeholder
        hits: Vec::new(),
    };

    let mut trailer_idx = 0;
    let mut is_moss_packet = false;
    let mut current_region: u8 = 0xff; // placeholder

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
                    "Region header seen before header, next 10 bytes: {:#X?}",
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
        }
    }
    if moss_packet.unit_id == INVALID_NO_HEADER_SEEN {
        Err(PyTypeError::new_err("No MOSS Packets in event"))
    } else {
        Ok((moss_packet, trailer_idx))
    }
}
