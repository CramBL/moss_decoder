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
    m.add_function(wrap_pyfunction!(decode_multiple_events_alt, m)?)?;

    m.add_class::<MossHit>()?;
    m.add_class::<MossPacket>()?;

    Ok(())
}

const INVALID_NO_HEADER_SEEN: u8 = 0xFF;
/// Decodes a single MOSS event into a [MossPacket]
#[pyfunction]
pub fn decode_event(bytes: &[u8]) -> PyResult<(MossPacket, Vec<u8>)> {
    let mut hits = Vec::new();

    let mut packet = MossPacket {
        unit_id: INVALID_NO_HEADER_SEEN, // placeholder
        hits: Vec::new(),
    };
    let mut processed_bytes_idx = 0;

    let mut is_moss_packet = false;
    let mut current_region: u8 = 0xff; // placeholder

    for (i, byte) in bytes.iter().enumerate() {
        match MossWord::from_byte(*byte) {
            MossWord::Idle => (),
            MossWord::UnitFrameHeader => {
                debug_assert!(!is_moss_packet);
                is_moss_packet = true;
                packet.unit_id = *byte & 0x0F
            }
            MossWord::UnitFrameTrailer => {
                debug_assert!(is_moss_packet);
                processed_bytes_idx = i + 1;
                break;
            }
            MossWord::RegionHeader => {
                debug_assert!(is_moss_packet);
                current_region = *byte & 0x03;
            }
            MossWord::Data0 => {
                debug_assert!(is_moss_packet);
                hits.push(MossHit {
                    region: current_region,            // region id
                    row: ((*byte & 0x3F) as u16) << 3, // row position [8:3]
                    column: 0,                         // placeholder
                });
            }
            MossWord::Data1 => {
                debug_assert!(is_moss_packet);
                // row position [2:0]
                hits.last_mut().unwrap().row |= ((*byte & 0x38) >> 3) as u16;
                // col position [8:6]
                hits.last_mut().unwrap().column = ((*byte & 0x07) as u16) << 6;
            }
            MossWord::Data2 => {
                debug_assert!(is_moss_packet);
                hits.last_mut().unwrap().column |= (*byte & 0x3F) as u16; // col position [5:0]
            }
            MossWord::Delimiter => {
                debug_assert!(!is_moss_packet);
            }
        }
    }

    if packet.unit_id == INVALID_NO_HEADER_SEEN {
        return Err(PyTypeError::new_err("No MOSS Packets in event"));
    }
    packet.hits.append(&mut hits);
    let (_processed, unprocessed) = bytes.split_at(processed_bytes_idx);
    Ok((packet, unprocessed.to_vec()))
}

/// Decodes multiple MOSS events into a list of [MossPacket]s
#[pyfunction]
pub fn decode_multiple_events(mut bytes: Vec<u8>) -> PyResult<Vec<MossPacket>> {
    let mut moss_packets: Vec<MossPacket> = Vec::new();

    while let Ok((packet, unprocessed_data)) = decode_event(&bytes) {
        moss_packets.push(packet);
        bytes = unprocessed_data;
    }

    if moss_packets.is_empty() {
        Err(PyTypeError::new_err("No MOSS Packets in events"))
    } else {
        Ok(moss_packets)
    }
}

const MIN_PREALLOC: usize = 10;

/// Decodes multiple MOSS events into a list of [MossPacket]s
#[pyfunction]
pub fn decode_multiple_events_alt(bytes: &[u8]) -> PyResult<(Vec<MossPacket>, usize)> {
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

    let mut is_moss_packet = false;
    let mut current_region: u8 = 0xff; // placeholder

    for (i, byte) in bytes.iter().enumerate() {
        match MossWord::from_byte(*byte) {
            MossWord::Idle => (),
            MossWord::UnitFrameHeader => {
                debug_assert!(!is_moss_packet);
                is_moss_packet = true;
                moss_packets.push(MossPacket {
                    unit_id: *byte & 0x0F,
                    hits: Vec::new(),
                });
            }
            MossWord::UnitFrameTrailer => {
                debug_assert!(is_moss_packet);
                is_moss_packet = false;
                last_trailer_idx = i;
            }
            MossWord::RegionHeader => {
                debug_assert!(is_moss_packet);
                current_region = *byte & 0x03;
            }
            MossWord::Data0 => {
                debug_assert!(is_moss_packet);
                moss_packets.last_mut().unwrap().hits.push(MossHit {
                    region: current_region,            // region id
                    row: ((*byte & 0x3F) as u16) << 3, // row position [8:3]
                    column: 0,                         // placeholder
                });
            }
            MossWord::Data1 => {
                debug_assert!(is_moss_packet);
                // row position [2:0]
                moss_packets
                    .last_mut()
                    .unwrap()
                    .hits
                    .last_mut()
                    .unwrap()
                    .row |= ((*byte & 0x38) >> 3) as u16;
                // col position [8:6]
                moss_packets
                    .last_mut()
                    .unwrap()
                    .hits
                    .last_mut()
                    .unwrap()
                    .column = ((*byte & 0x07) as u16) << 6;
            }
            MossWord::Data2 => {
                debug_assert!(is_moss_packet);
                moss_packets
                    .last_mut()
                    .unwrap()
                    .hits
                    .last_mut()
                    .unwrap()
                    .column |= (*byte & 0x3F) as u16; // col position [5:0]
            }
            MossWord::Delimiter => {
                debug_assert!(!is_moss_packet);
            }
        }
    }

    if moss_packets.is_empty() {
        Err(PyTypeError::new_err("No MOSS Packets in events"))
    } else {
        Ok((moss_packets, last_trailer_idx))
    }
}
