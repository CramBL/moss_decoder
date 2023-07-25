"""Performant decoding of MOSS readout data implemented in Rust"""

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
