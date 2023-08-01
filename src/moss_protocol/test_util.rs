//! Utility for testing Rust code that uses the MOSS protocol.
#![allow(dead_code, missing_docs)]

pub const IDLE: u8 = 0xFF;
pub const UNIT_FRAME_TRAILER: u8 = 0xE0;
pub const UNIT_FRAME_HEADER_1: u8 = 0xD1;
pub const REGION_HEADER_0: u8 = 0xC0;
pub const REGION_HEADER_1: u8 = 0xC1;
pub const REGION_HEADER_2: u8 = 0xC2;
pub const REGION_HEADER_3: u8 = 0xC3;

pub fn fake_event_simple() -> Vec<u8> {
    vec![
        UNIT_FRAME_HEADER_1,
        REGION_HEADER_0,
        // Hit row 2, col 8
        0x00,
        0x50,
        0x88,
        IDLE,
        0x01,
        0x50,
        0x88,
        REGION_HEADER_1,
        // Hit row 301, col 433
        0x25,
        0x6E,
        0xB1,
        REGION_HEADER_2,
        REGION_HEADER_3,
        // Hit row 2, col 8
        0x00,
        0x50,
        0x88,
        UNIT_FRAME_TRAILER,
    ]
}

pub fn fake_multiple_events() -> Vec<u8> {
    vec![
        UNIT_FRAME_HEADER_1,
        REGION_HEADER_0,
        // Hit row 2, col 8
        0x00,
        0x50,
        0x88,
        REGION_HEADER_1,
        // Hit row 301, col 433
        0x25,
        0x6E,
        0xB1,
        REGION_HEADER_2,
        REGION_HEADER_3,
        // Hit row 2, col 8
        0x00,
        0x50,
        0x88,
        UNIT_FRAME_TRAILER,
        0xD1, // Unit 1, otherwise identical event
        REGION_HEADER_0,
        // Hit row 2, col 8
        0x00,
        0x50,
        0x88,
        IDLE,
        REGION_HEADER_1,
        // Hit row 301, col 433
        0x25,
        0x6E,
        0xB1,
        IDLE,
        IDLE,
        REGION_HEADER_2,
        REGION_HEADER_3,
        // Hit row 2, col 8
        0x00,
        0x50,
        0x88,
        UNIT_FRAME_TRAILER,
        0xD2, // Unit 2, empty
        REGION_HEADER_0,
        REGION_HEADER_1,
        REGION_HEADER_2,
        IDLE,
        REGION_HEADER_3,
        UNIT_FRAME_TRAILER,
        0xD3, // Unit 3, simple hits
        REGION_HEADER_0,
        0x00,
        0b0100_0000, // row 0
        0b1000_0000, // col 0
        REGION_HEADER_1,
        0x00,
        0b0100_1000, // row 1
        0b1000_0001, // col 1
        REGION_HEADER_2,
        0x00,
        0b0101_0000, // row 2
        0b1000_0010, // col 2
        REGION_HEADER_3,
        0x00,
        0b0101_1000, // row 3
        0b1000_0011, // col 3
        IDLE,
        UNIT_FRAME_TRAILER,
    ]
}

pub fn fake_event_protocol_error() -> Vec<u8> {
    vec![
        UNIT_FRAME_HEADER_1,
        REGION_HEADER_0,
        // Hit row 2, col 8
        0x00,
        0xF0, // Protocol error
        0x88,
        IDLE,
        REGION_HEADER_1,
        // Hit row 301, col 433
        0x25,
        0x6E,
        0xB1,
        IDLE,
        REGION_HEADER_2,
        REGION_HEADER_3,
        // Hit row 2, col 8
        0x00,
        0x50,
        0x88,
        UNIT_FRAME_TRAILER,
    ]
}
