use crate::{moss_protocol::MossWord, MossHit, MossPacket};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct InvalidWordInfo {
    invalid_byte: u8,
    in_packet: bool,
    region: Option<u8>,
    index: usize,
}

impl InvalidWordInfo {
    pub(crate) fn new(invalid_byte: u8, in_packet: bool, region: u8, index: usize) -> Self {
        Self {
            invalid_byte,
            in_packet,
            region: if in_packet { Some(region) } else { None },
            index,
        }
    }

    pub(crate) fn set_index_offset(&mut self, idx_offset: usize) {
        self.index += idx_offset;
    }

    pub(crate) fn to_error_msg(self) -> String {
        let (describe_decode_state, region_description) = if self.in_packet {
            (
                "in MOSS packet",
                Some(format!(", region {}", self.region.unwrap())), // safe to unwrap because in_packet is true
            )
        } else {
            ("before header seen", None)
        };
        format!(
            "Invalid word=0x{byte:02X} at index={i} {describe_decode_state}{opt_region}",
            byte = self.invalid_byte,
            i = self.index,
            describe_decode_state = describe_decode_state,
            opt_region = region_description.unwrap_or_default(),
        )
    }
}

/// Decodes a single MOSS event into a [MossPacket] and the index of the trailer byte (Rust only)
#[inline]
pub(crate) fn debug_decode_event(
    bytes: &[u8],
) -> std::io::Result<(MossPacket, usize, Vec<InvalidWordInfo>)> {
    const INVALID_NO_HEADER_SEEN: u8 = 0xFF;
    let mut moss_packet = MossPacket {
        unit_id: INVALID_NO_HEADER_SEEN, // placeholder
        hits: Vec::new(),
    };

    let mut trailer_idx = 0;
    let mut current_region: u8 = 0xff; // placeholder

    let mut is_moss_packet = false;
    let mut invalid_words: Vec<InvalidWordInfo> = Vec::new();

    for (i, byte) in bytes.iter().enumerate() {
        match MossWord::from_byte(*byte) {
            MossWord::Idle => {
                if !is_moss_packet {
                    invalid_words.push(record_protocol_error(InvalidWordInfo::new(
                        *byte,
                        is_moss_packet,
                        current_region,
                        i,
                    )));
                }
            }
            MossWord::UnitFrameHeader => {
                if is_moss_packet {
                    invalid_words.push(record_protocol_error(InvalidWordInfo::new(
                        *byte,
                        is_moss_packet,
                        current_region,
                        i,
                    )));
                } else {
                    is_moss_packet = true;
                    moss_packet.unit_id = *byte & 0x0F
                }
            }
            MossWord::UnitFrameTrailer => {
                if is_moss_packet {
                    trailer_idx = i;
                } else {
                    invalid_words.push(record_protocol_error(InvalidWordInfo::new(
                        *byte,
                        is_moss_packet,
                        current_region,
                        i,
                    )));
                }
                break;
            }
            MossWord::RegionHeader => {
                if is_moss_packet {
                    current_region = *byte & 0x03;
                } else {
                    invalid_words.push(record_protocol_error(InvalidWordInfo::new(
                        *byte,
                        is_moss_packet,
                        current_region,
                        i,
                    )));
                }
            }
            MossWord::Data0 => {
                if is_moss_packet {
                    moss_packet.hits.push(MossHit {
                        region: current_region,            // region id
                        row: ((*byte & 0x3F) as u16) << 3, // row position [8:3]
                        column: 0,                         // placeholder
                    });
                } else {
                    invalid_words.push(record_protocol_error(InvalidWordInfo::new(
                        *byte,
                        is_moss_packet,
                        current_region,
                        i,
                    )));
                }
            }
            MossWord::Data1 => {
                if is_moss_packet {
                    // row position [2:0]
                    moss_packet.hits.last_mut().unwrap().row |= ((*byte & 0x38) >> 3) as u16;
                    // col position [8:6]
                    moss_packet.hits.last_mut().unwrap().column = ((*byte & 0x07) as u16) << 6;
                } else {
                    invalid_words.push(record_protocol_error(InvalidWordInfo::new(
                        *byte,
                        is_moss_packet,
                        current_region,
                        i,
                    )));
                }
            }
            MossWord::Data2 => {
                if is_moss_packet {
                    moss_packet.hits.last_mut().unwrap().column |= (*byte & 0x3F) as u16;
                } else {
                    invalid_words.push(record_protocol_error(InvalidWordInfo::new(
                        *byte,
                        is_moss_packet,
                        current_region,
                        i,
                    )));
                }
                // col position [5:0]
            }
            MossWord::Delimiter => {
                if is_moss_packet {
                    invalid_words.push(record_protocol_error(InvalidWordInfo::new(
                        *byte,
                        is_moss_packet,
                        current_region,
                        i,
                    )));
                }
            }
            MossWord::ProtocolError => {
                invalid_words.push(record_protocol_error(InvalidWordInfo::new(
                    *byte,
                    is_moss_packet,
                    current_region,
                    i,
                )));
            }
        }
    }
    if moss_packet.unit_id == INVALID_NO_HEADER_SEEN || trailer_idx == 0 {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No MOSS packet found",
        ))
    } else {
        Ok((moss_packet, trailer_idx, invalid_words))
    }
}

// Single place to record protocol errors
// useful for a single place to decide whether to print to stderr or not (or something else in the future)
#[inline]
fn record_protocol_error(new_invalid_word: InvalidWordInfo) -> InvalidWordInfo {
    eprintln!("{}", new_invalid_word.to_error_msg());
    new_invalid_word
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moss_protocol::test_util::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_debug_decode_simple_event() {
        pyo3::prepare_freethreaded_python();
        let event_data_packet = fake_event_simple();

        let res = debug_decode_event(&event_data_packet);
        assert!(res.is_ok());
        let (moss_packet, trailer_idx, invalid_words) = res.unwrap();
        assert_eq!(moss_packet.unit_id, 1);
        assert_eq!(moss_packet.hits.len(), 4);
        assert_eq!(trailer_idx, 18);
        assert_eq!(invalid_words.len(), 0);
    }

    #[test]
    fn test_debug_decode_protocol_error() {
        let event_data_packet = fake_event_protocol_error_fb_in_idle();

        let res = debug_decode_event(&event_data_packet);
        assert!(res.is_ok());

        let (moss_packet, trailer_idx, invalid_words) = res.unwrap();

        assert_eq!(moss_packet.unit_id, 1);
        assert_eq!(moss_packet.hits.len(), 8);
        assert_eq!(trailer_idx, 34);
        assert_eq!(invalid_words.len(), 1);
        assert_eq!(invalid_words[0].invalid_byte, 0xFB);
        assert_eq!(invalid_words[0].in_packet, true);
        assert_eq!(invalid_words[0].region, Some(0));
        assert_eq!(invalid_words[0].index, 13);
    }

    #[test]
    fn test_debug_decode_protocol_error_2_events() {
        let mut data_two_packets = fake_event_simple();
        data_two_packets.extend(vec![0xFA, 0xFA]); // Add padding delimiter bytes
        data_two_packets.extend(fake_event_protocol_error_fb_in_idle());
        data_two_packets.extend(vec![0xFA, 0xFA]); // Add padding delimiter bytes
        println!("data_two_packets: {:#X?}", &data_two_packets);

        let mut moss_packets = Vec::new();
        let mut invalid_words = Vec::new();
        let mut last_trailer_idx = 0;

        while let Ok((new_moss_packet, trailer_idx, new_invalid_words)) =
            debug_decode_event(&data_two_packets[last_trailer_idx..])
        {
            new_invalid_words.into_iter().for_each(|mut invalid_word| {
                invalid_word.set_index_offset(last_trailer_idx);
                invalid_words.push(invalid_word);
            });
            last_trailer_idx += trailer_idx + 1;
            moss_packets.push(new_moss_packet);
        }

        let invalid_words_msgs: Vec<String> = invalid_words
            .iter()
            .map(|invalid_word| invalid_word.to_error_msg())
            .collect();

        println!("{}", invalid_words_msgs.join("\n"));
        assert_eq!(moss_packets.len(), 2);
        assert_eq!(invalid_words.len(), 1);
        assert_eq!(
            invalid_words_msgs[0],
            "Invalid word=0xFB at index=34 in MOSS packet, region 0",
        );
        assert_eq!(invalid_words[0].index, 34);
        assert_eq!(data_two_packets[invalid_words[0].index], 0xFB);
    }

    #[test]
    fn test_debug_decode_protocol_error_3_events() {
        let mut data_two_packets = fake_event_simple();
        data_two_packets.extend(vec![0xFA, 0xFA]); // Add padding delimiter bytes
        data_two_packets.extend(fake_event_simple());
        data_two_packets.extend(vec![0xFA, 0xFA]); // Add padding delimiter bytes
        data_two_packets.extend(fake_event_protocol_error_fb_in_idle());
        data_two_packets.extend(vec![0xFA, 0xFA]); // Add padding delimiter bytes

        let mut moss_packets = Vec::new();
        let mut invalid_words = Vec::new();
        let mut last_trailer_idx = 0;

        while let Ok((new_moss_packet, trailer_idx, new_invalid_words)) =
            debug_decode_event(&data_two_packets[last_trailer_idx..])
        {
            new_invalid_words.into_iter().for_each(|mut invalid_word| {
                invalid_word.set_index_offset(last_trailer_idx);
                invalid_words.push(invalid_word);
            });
            last_trailer_idx += trailer_idx + 1;
            moss_packets.push(new_moss_packet);
        }

        let invalid_words_msgs: Vec<String> = invalid_words
            .iter()
            .map(|invalid_word| invalid_word.to_error_msg())
            .collect();

        println!("{}", invalid_words_msgs.join("\n"));

        assert_eq!(moss_packets.len(), 3);
        assert_eq!(invalid_words.len(), 1);
        assert_eq!(data_two_packets[invalid_words[0].index], 0xFB);
        assert_eq!(invalid_words[0].index, 55);
        assert_eq!(
            invalid_words_msgs[0],
            "Invalid word=0xFB at index=55 in MOSS packet, region 0",
        );
    }
}
