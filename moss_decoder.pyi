"""Performant decoding of MOSS readout data implemented in Rust"""

from pathlib import Path
from typing import Optional

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

def decode_event(bytes: bytes) -> tuple[MossPacket, int]: ...
def decode_all_events(bytes: bytes) -> tuple[list[MossPacket], int]: ...
def decode_from_file(path: str | Path) -> list[MossPacket]: ...
def decode_n_events(
    bytes: bytes,
    take: int,
    skip: Optional[int] = None,
    prepend_buffer: Optional[bytes] = None,
) -> tuple[list[MossPacket], int]: ...
def skip_n_take_all(
    bytes: bytes, skip: int = None
) -> tuple[list[MossPacket], Optional[bytes]]: ...
