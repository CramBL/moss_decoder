"""Integration tests. Uses the `moss_decoder` package
from python and allows benchmarks."""
import sys  # Don't want to depend on `argparse`
import time
from pathlib import Path
import moss_decoder
from moss_decoder import MossPacket, MossHit
from moss_decoder import decode_event, decode_event_noexcept

FILE_PATH = Path("tests/moss_noise.raw")


def read_bytes_from_file(file_path: Path) -> bytes:
    """Open file at `file_path` and read as binary, return `bytes`"""
    with open(file_path, "rb") as readout_file:
        raw_bytes = readout_file.read()

    return raw_bytes


def make_simple_moss_event_packet() -> bytes:
    """Make a complete simple MOSS packet containing
    Unit 0 and 1 hit in region 1 row 2 col 8"""
    unit_frame_header_0 = b"\xD0"
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
        unit_frame_header_0
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


def decode_multi_event(raw_bytes: bytes, fsm: bool = False) -> tuple[list["MossPacket"], int]:
    """Takes `bytes` and decodes it as `MossPacket`s.
    returns a tuple of `list[MossPackets]` and an int that indicates the
    index where the last MOSS trailer was seen
    """
    if fsm is True:
        packets, last_trailer_idx = moss_decoder.decode_multiple_events_fsm(
        raw_bytes)
    else:
        packets, last_trailer_idx = moss_decoder.decode_multiple_events(
        raw_bytes)

    return packets, last_trailer_idx


def test_decode_multi_event(fsm: bool = False):
    """Test that multiple events are correctly decoded from raw bytes"""
    print(
        (
            "=== Test that multiple events"
            "are correctly decoded from raw bytes ==="
        )
    )
    raw_bytes = read_bytes_from_file(FILE_PATH)
    byte_count = len(raw_bytes)
    last_byte_idx = byte_count - 1

    print(f"Read {byte_count} bytes")

    packets, last_trailer_idx = decode_multi_event(raw_bytes=raw_bytes, fsm=fsm)

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


def test_100k_single_decodes(noexcept=False):
    """Tests 100k calls to decode_event (single event decoding)"""

    if noexcept:
        print(("=== Test 100k calls to decode_event with noexcept ==="))
    else:
        print(("=== Test 100k calls to decode_event ==="))

    raw_bytes = read_bytes_from_file(FILE_PATH)
    byte_count = len(raw_bytes)
    last_byte_idx = byte_count - 1

    print(f"Read {byte_count} bytes")

    packets = []
    last_trailer_idx = 0

    if noexcept is False:
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

    if noexcept is True:
        res = 1
        packets = []
        while res != 0:
            packet, res = decode_event_noexcept(raw_bytes[last_trailer_idx:])
            if res != 0:
                last_trailer_idx = last_trailer_idx + res + 1
                packets.append(packet)

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
            test_decode_multi_event()
            sys.exit(0)

    test_fundamental_class_comparisons()

    start = time.time()
    test_decode_multi_event()
    print(f"Done in: {time.time()-start:.3f} s\n")
    start = time.time()
    test_decode_multi_event(fsm=True)
    print(f"Done in: {time.time()-start:.3f} s\n")
    start = time.time()
    test_moss_packet_print()
    print(f"Done in: {time.time()-start:.3f} s\n")
    start = time.time()
    test_100k_single_decodes()
    print(f"Done in: {time.time()-start:.3f} s\n")
    start = time.time()
    test_100k_single_decodes(noexcept=True)
    print(f"Done in: {time.time()-start:.3f} s\n")
