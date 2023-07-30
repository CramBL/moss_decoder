"""Performant decoding of MOSS readout data implemented in Rust"""

from pathlib import Path

class MossHit:
    """A MOSS hit instance"""

    region: int
    column: int
    row: int

    def __init__(self, region: int, row: int, column: int) -> MossHit:
        self.region = region
        self.column = column
        self.row = row

class MossPacket:
    """A decoded MOSS event packet with a `Unit ID` and a list of `MossHit`s"""

    unit_id: int
    hits: list[MossHit]

    def __init__(self, unit_id: int) -> MossPacket:
        self.unit_id = unit_id
        self.hits = []

def decode_event(raw_bytes: bytes) -> tuple[MossPacket, int]: ...
def decode_event_noexcept(raw_bytes: bytes) -> tuple[MossPacket, int]: ...
def decode_multiple_events(raw_bytes: bytes) -> tuple[list[MossPacket], int]: ...
def decode_from_file(path: str | Path) -> list[MossPacket]: ...

def decode_event_fsm(raw_bytes: bytes) -> tuple[MossPacket, int]: ...
def decode_multiple_events_fsm(raw_bytes: bytes) -> tuple[list[MossPacket], int]: ...
def decode_from_file_fsm(path: str | Path) -> list[MossPacket]: ...
