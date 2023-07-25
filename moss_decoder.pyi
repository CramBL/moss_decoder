"""Performant decoding of MOSS readout data implemented in Rust"""

class MossHit:
    """A MOSS hit instance"""

    region: int
    column: int
    row: int

    def __init__(self, region: int, row: int, column: int) -> "MossHit":
        self.region = region
        self.column = column
        self.row = row
    def region(self) -> int:  # Get the MOSS region ID
        """Get the Region ID"""
        return self.region
    def row(self) -> int:
        """Get the Row"""
        return self.row
    def column(self) -> int:
        """Get the Column"""
        return self.column
