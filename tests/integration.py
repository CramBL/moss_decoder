"""Integration tests. Uses the `moss_decoder` package
from python and allows benchmarks."""
import sys  # Don't want to depend on `argparse`
import time
from pathlib import Path
from typing import Optional
import moss_decoder
from moss_decoder import MossPacket, MossHit
from moss_decoder import decode_event

FILE_MOSS_NOISE = Path("tests/test-data/moss_noise.raw")
FILE_MOSS_NOISE_ALL_REGION = Path("tests/test-data/noise_all_regions.raw")
FILE_NOISE_RANDOM_REGION = Path("tests/test-data/noise_random_region.raw")
FILE_PATTERN_ALL_REGIONS = Path("tests/test-data/pattern_all_regions.raw")
FILE_4_EVENTS_PARTIAL_END = Path("tests/test-data/moss_noise_0-499b.raw")
FILE_3_EVENTS_PARTIAL_START = Path("tests/test-data/moss_noise_500-999b.raw")


class MockMossDecoder:
    _data_files: Optional[list[Path]] = None
    _current_file_idx = 0
    _current_file_events_decoded = 0

    def __init__(self, data_files: Optional[list[Path]] = None) -> "MockMossDecoder":
        self._data_files = data_files

    def get_next_n_events(self, events: int) -> list[MossPacket]:
        """Read N events from the current file, if it has less than N events,
        try to get the rest in the next file"""
        buf = read_bytes_from_file(self._data_files[self._current_file_idx])
        try:
            if self._current_file_events_decoded > 0:
                skip = self._current_file_events_decoded
            else:
                skip = None
            print(f"Taking {events}, skipping {skip}")
            packets, last_trailer_idx = moss_decoder.decode_n_events(
                bytes=buf, take=events, skip=skip
            )
            print(
                f"Decoded {len(packets)} events, last trailer at index {last_trailer_idx}"
            )
            self._current_file_events_decoded += events
            return packets
        except BytesWarning as warning:
            print(f"Got warning: {warning}")
            print(f"Taking all, skipping {self._current_file_events_decoded}")
            packets = []
            (
                remaining_packets,
                remainder,
            ) = moss_decoder.skip_n_take_all(
                bytes=buf, skip=self._current_file_events_decoded
            )

            if remaining_packets is not None:
                print(
                    f"Decoded {len(remaining_packets)} events, got {len(remainder)} bytes"
                )
                packets.extend(remaining_packets)
            else:
                print(f"Decoded 0 events, got {len(remainder)} bytes")
            self._current_file_idx += 1
            self._current_file_events_decoded = 0
            if self._current_file_idx == len(self._data_files):
                raise AssertionError(
                    f"Reached end of data before decoding {events} events"
                )
            print(f"Trying to get {events - len(packets)} from second file")
            buf = read_bytes_from_file(self._data_files[self._current_file_idx])

            rest_of_packets, last_trailer_idx = moss_decoder.decode_n_events(
                bytes=buf,
                take=events - len(packets),
                prepend_buffer=remainder,
            )
            if len(rest_of_packets) != 0:
                self._current_file_events_decoded += len(rest_of_packets) - 1
            print(f"Second file read, got: {len(rest_of_packets)}")
            packets.extend(rest_of_packets)
            return packets

    def decode_from_file(self, file_path: Path) -> list[MossPacket]:
        """Decode raw MOSS readout data `MossPacket` objects from a file
        and return list of `MossPacket` objects"""
        assert isinstance(
            file_path, Path
        ), f'Argument must be type Path, got {type(file_path)}. \
    Supply the file path with: Path("path/to/file")'
        assert file_path.is_file(), f"File does not exist: {file_path}"

        packets = moss_decoder.decode_from_file(file_path)

        return packets


def read_bytes_from_file(file_path: Path) -> bytes:
    """Open file at `file_path` and read as binary, return `bytes`"""
    print(f"Reading from file {file_path}")
    with open(file_path, "rb") as readout_file:
        raw_bytes = readout_file.read()

    return raw_bytes


def make_simple_moss_event_packet() -> bytes:
    """Make a complete simple MOSS packet containing
    Unit 0 and 1 hit in region 1 row 2 col 8"""
    unit_frame_header_1 = b"\xD1"
    padding = b"\xFA"
    unit_frame_trailer = b"\xE0"
    region_header_0 = b"\xC0"
    region_header_1 = b"\xC1"
    region_header_2 = b"\xC2"
    region_header_3 = b"\xC3"
    data_0 = b"\x00"
    data_1 = b"\x50"  # row 2
    data_2 = b"\x88"  # col 8

    simple_packet = (
        unit_frame_header_1
        + region_header_0
        + region_header_1
        + data_0
        + data_1
        + data_2
        + region_header_2
        + region_header_3
        + unit_frame_trailer
        + padding
    )
    return simple_packet


def test_decode_1GB_file(file_path: Path, expect_packets: int):
    big_file = Path("1GB-data.raw")
    if big_file.exists():
        big_file.unlink()
    # Read the content of the input binary file
    with open(file_path, "rb") as input_file:
        content = input_file.read()

    # Append the content multiple times to the output binary file
    with open(big_file, "ab") as output_file:
        for _ in range(100):
            output_file.write(content)
    start = time.time()
    test_decode_all_from_file(file_path=big_file, expect_packets=expect_packets * 100)
    print(f"Done in: {time.time()-start:.3f} s\n")
    big_file.unlink()


def test_decode_all_from_file(file_path: Path, expect_packets: int):
    decoder = MockMossDecoder()
    print(f"Decoding all from file {file_path}")
    packets = decoder.decode_from_file(file_path)
    assert (
        len(packets) == expect_packets
    ), f"Expected {expect_packets}, got {len(packets)}"
    print(f"Got {len(packets)} packets")
    print("==> Test OK")


def test_decode_partial_events_from_two_files():
    decoder = MockMossDecoder([FILE_4_EVENTS_PARTIAL_END, FILE_3_EVENTS_PARTIAL_START])

    packets = decoder.get_next_n_events(2)
    assert len(packets) == 2, f"Expected 2 packets, got {len(packets)}: {packets}"
    packets = decoder.get_next_n_events(2)
    assert len(packets) == 2, f"Expected 2 packets, got {len(packets)}: {packets}"
    packets = decoder.get_next_n_events(2)
    assert len(packets) == 2, f"Expected 2 packets, got {len(packets)}: {packets}"
    packets = decoder.get_next_n_events(1)
    assert len(packets) == 1, f"Expected 1 packets, got {len(packets)}: {packets}"
    packets = decoder.get_next_n_events(1)
    assert len(packets) == 1, f"Expected 1 packets, got {len(packets)}: {packets}"
    try:
        _ = decoder.get_next_n_events(1)
        assert False, "expected decoding to fail but it didn't"
    except AssertionError as exc:
        assert "Reached end" in str(exc), f"Got unexpected error: {exc}"

    print("==> Test OK\n\n")


def test_decode_multi_event(path: Path, expect_remainder_bytes: int):
    """Test that multiple events are correctly decoded from raw bytes"""
    print("=== Test multiple events are correctly decoded from raw bytes ===")
    raw_bytes = read_bytes_from_file(path)
    byte_count = len(raw_bytes)
    last_byte_idx = byte_count - 1

    print(f"Read {byte_count} bytes")

    packets, last_trailer_idx = moss_decoder.decode_all_events(raw_bytes)

    print(f"Decoded {len(packets)} packets")

    print(f"Last trailer at index: {last_trailer_idx}/{last_byte_idx}")
    remainder_count = last_byte_idx - last_trailer_idx
    print(f"Remainder: {remainder_count} byte(s)")

    if byte_count > last_trailer_idx:
        print(f"Remainder byte(s): {raw_bytes[last_trailer_idx+1:]}")

    assert (
        remainder_count == expect_remainder_bytes
    ), f"Expected last trailer found {expect_remainder_bytes} bytes before last byte, got: {remainder_count}"
    print("==> Test OK\n\n")


def test_moss_packet_print():
    """Test that the `MossPacket` class can be printed as expected in python"""
    print("=== Test printing of MossPacket class ===")
    moss_event = make_simple_moss_event_packet()
    moss_packet, _rest = decode_event(moss_event)
    print(f"type of MossPacket: {type(moss_packet)}")
    print(f"Print MossPacket: {moss_packet}")
    print("Print MossPacket attributes")
    print(f"\tUnit ID: {moss_packet.unit_id}")
    print("Iterate over hits of the MOSS packet and print the hits")
    for hit in moss_packet.hits:
        print(f"\tHits: {hit}")

    print("Print MOSS Hit attributes")
    for hit in moss_packet.hits:
        print(f"\tHits: {hit}")
        print(f"\t\tHit region: {hit.region}")
        print(f"\t\tHit row: {hit.row}")
        print(f"\t\tHit column: {hit.column}")

    print("==> Test OK\n\n")


def test_100k_single_decodes():
    """Tests 100k calls to decode_event (single event decoding)"""

    print(("=== Test 100k calls to decode_event ==="))

    raw_bytes = read_bytes_from_file(FILE_MOSS_NOISE)
    byte_count = len(raw_bytes)
    last_byte_idx = byte_count - 1

    print(f"Read {byte_count} bytes")

    packets = []
    last_trailer_idx = 0

    more_data = True
    while more_data:
        try:
            pack, tmp_trailer_idx = moss_decoder.decode_event(
                raw_bytes[last_trailer_idx:]
            )
            packets.append(pack)
            last_trailer_idx = last_trailer_idx + tmp_trailer_idx + 1
        except ValueError as exc:
            print(f"Decode event returned value error: {exc}")
            more_data = False
        except AssertionError as exc:
            print(f"Decode event returned assertion error: {exc}")
            more_data = False
            raise exc

    last_trailer_idx = last_trailer_idx - 1

    print(f"Decoded {len(packets)} packets")
    print(f"Last trailer at index: {last_trailer_idx}/{last_byte_idx}")
    remainder_count = last_byte_idx - last_trailer_idx
    print(f"Remainder: {remainder_count} byte(s)")

    if byte_count > last_trailer_idx:
        print(f"Remainder byte(s): {raw_bytes[last_trailer_idx+1:]}")

    assert (
        remainder_count == 1
    ), f"Expected last trailer found at index 1, got: {remainder_count}"
    print("==> Test OK\n\n")


def test_fundamental_class_comparisons():
    """Test fundamental class functionality"""

    print("=== Comparing MossHit attributes ===\n")
    hit_a = MossHit(0, 1, 2)
    hit_b = MossHit(0, 1, 2)
    assert hit_a == hit_b, f"{hit_a} != {hit_b}"
    print("__eq__ is OK")

    print(repr(hit_a) + " == " + repr(hit_a))
    assert repr(hit_a) == repr(hit_b)
    print("__repr__ is OK")

    print(str(hit_a) + " == " + str(hit_b))
    assert str(hit_a) == str(hit_b)

    print("__str__ is OK")
    print("==> MossHit is OK\n\n")

    print("=== Comparing MossPacket attributes ===\n")
    pack_a = MossPacket(1)
    pack_b = MossPacket(1)
    assert pack_a == pack_b, f"{pack_a} != {pack_b}"
    print("__eq__ is OK")

    print(repr(pack_a) + " == " + repr(pack_b))
    assert repr(pack_a) == repr(pack_b)
    print("__repr__ is OK")

    print(str(pack_a) + " == " + str(pack_b))
    assert str(pack_a) == str(pack_b)
    print("__str__ is OK")

    print("==> MossPacket is OK\n\n")


if __name__ == "__main__":
    args = sys.argv

    if len(args) > 1:
        if args[1] == "benchmark":
            # Just run this and then exit
            test_decode_multi_event(path=FILE_MOSS_NOISE, expect_remainder_bytes=1)
            sys.exit(0)

    test_fundamental_class_comparisons()
    test_decode_partial_events_from_two_files()

    start = time.time()
    test_decode_all_from_file(file_path=FILE_MOSS_NOISE, expect_packets=100000)
    print(f"Done in: {time.time()-start:.3f} s\n")

    start = time.time()
    test_decode_all_from_file(file_path=FILE_NOISE_RANDOM_REGION, expect_packets=1044)
    print(f"Done in: {time.time()-start:.3f} s\n")

    start = time.time()
    test_decode_all_from_file(file_path=FILE_PATTERN_ALL_REGIONS, expect_packets=1000)
    print(f"Done in: {time.time()-start:.3f} s\n")

    start = time.time()
    test_decode_all_from_file(file_path=FILE_MOSS_NOISE_ALL_REGION, expect_packets=1000)
    print(f"Done in: {time.time()-start:.3f} s\n")

    start = time.time()
    test_decode_multi_event(path=FILE_MOSS_NOISE, expect_remainder_bytes=1)
    print(f"Done in: {time.time()-start:.3f} s\n")

    start = time.time()
    test_decode_multi_event(path=FILE_MOSS_NOISE_ALL_REGION, expect_remainder_bytes=1)
    print(f"Done in: {time.time()-start:.3f} s\n")

    start = time.time()
    test_decode_multi_event(path=FILE_NOISE_RANDOM_REGION, expect_remainder_bytes=3)
    print(f"Done in: {time.time()-start:.3f} s\n")

    start = time.time()
    test_decode_multi_event(path=FILE_PATTERN_ALL_REGIONS, expect_remainder_bytes=2)
    print(f"Done in: {time.time()-start:.3f} s\n")

    start = time.time()
    test_moss_packet_print()
    print(f"Done in: {time.time()-start:.3f} s\n")

    test_decode_1GB_file(file_path=FILE_MOSS_NOISE, expect_packets=100000)

    start = time.time()
    test_100k_single_decodes()
    print(f"Done in: {time.time()-start:.3f} s\n")
