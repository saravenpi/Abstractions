#!/usr/bin/env python3
"""Reads every committed icon back and checks its size, colours, centring and padding.

Usage: test.py

Exits 1 on the first file that breaks a rule. Standard library only.
"""

import struct
import sys
import zlib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MAC_SIZES = (16, 32, 64, 128, 256, 512, 1024)
WIN_SIZES = [16, 24, 32, 48, 64, 256]
LINUX_SIZES = (16, 32, 48, 64, 128, 256, 512)
TILE_SHARE = (0.37, 0.45)
TILE_PADDING = 0.25
MAC_BODY = 824 / 1024
MAC_SHARE = (0.30, 0.41)


def decode_png(data):
    """Size and RGBA rows of an 8-bit RGBA PNG whose scanlines use filter 0."""
    assert data[:8] == b"\x89PNG\r\n\x1a\n", "not a PNG"
    width, height, depth, colour = struct.unpack(">IIBB", data[16:26])
    assert (depth, colour) == (8, 6), "not 8-bit RGBA"
    idat, at = b"", 8
    while at < len(data):
        length, kind = struct.unpack(">I4s", data[at:at + 8])
        body = data[at + 8:at + 8 + length]
        assert zlib.crc32(kind + body) == struct.unpack(">I", data[at + 8 + length:at + 12 + length])[0], "bad CRC"
        idat += body if kind == b"IDAT" else b""
        at += 12 + length
    raw = zlib.decompress(idat)
    stride = width * 4 + 1
    assert len(raw) == stride * height, "wrong pixel count"
    assert all(raw[y * stride] == 0 for y in range(height)), "unexpected filter"
    return width, height, [raw[y * stride + 1:(y + 1) * stride] for y in range(height)]


def decode_bitmap(data):
    """Size and RGBA rows of an ICO bitmap entry."""
    head, width, height, planes, bits, compression = struct.unpack("<IiiHHI", data[:20])
    assert (head, planes, bits, compression) == (40, 1, 32, 0), "unexpected bitmap header"
    assert height == width * 2, "bitmap height must cover the mask"
    mask = ((width + 31) // 32) * 4 * width
    assert len(data) == 40 + width * width * 4 + mask, "wrong bitmap length"
    assert not any(data[40 + width * width * 4:]), "mask must be empty"
    rows = []
    for y in reversed(range(width)):
        row = data[40 + y * width * 4:40 + (y + 1) * width * 4]
        rows.append(b"".join(bytes((row[i + 2], row[i + 1], row[i], row[i + 3])) for i in range(0, len(row), 4)))
    return width, width, rows


def decode_ico(data):
    """Every image of an ICO file as (declared size, stored as PNG, decoded image)."""
    reserved, kind, count = struct.unpack("<HHH", data[:6])
    assert (reserved, kind) == (0, 1), "not an ICO"
    out = []
    for n in range(count):
        width, height, _, _, planes, bits, length, offset = struct.unpack("<BBBBHHII", data[6 + n * 16:22 + n * 16])
        assert (planes, bits, width) == (1, 32, height), "unexpected directory entry"
        image = data[offset:offset + length]
        packed = image[:4] == b"\x89PNG"
        out.append((width or 256, packed, decode_png(image) if packed else decode_bitmap(image)))
    assert offset + length == len(data), "trailing bytes"
    return out


def square(rows):
    """Left, top and width of the white square after checking it is one filled, centred square."""
    size = len(rows)
    white = [(x, y) for y, row in enumerate(rows) for x in range(size) if row[x * 4:x * 4 + 4] == b"\xff\xff\xff\xff"]
    assert white, "no white pixel"
    left, top = min(x for x, _ in white), min(y for _, y in white)
    width = max(x for x, _ in white) - left + 1
    assert len(white) == width * width, "white pixels are not one filled square"
    assert max(y for _, y in white) - top + 1 == width, "white area is not square"
    assert left == top == size - left - width, "square is not centred"
    return left, top, width


def check_tile(image, size):
    """A full-bleed tile: two opaque colours and at least 3 px and a quarter of the width of black on every side."""
    width, height, rows = image
    assert (width, height) == (size, size), f"size is {width}x{height}"
    colours = {row[i:i + 4] for row in rows for i in range(0, len(row), 4)}
    assert colours == {b"\x00\x00\x00\xff", b"\xff\xff\xff\xff"}, "expected exactly black and white"
    pad, _, side = square(rows)
    assert pad >= 3 and pad >= size * TILE_PADDING, f"padding {pad} px is too small"
    assert TILE_SHARE[0] <= side / size <= TILE_SHARE[1], f"square share {side / size:.3f}"
    return f"{size} px: square {side} px ({side / size:.1%}), padding {pad} px"


def check_mac(image, size):
    """A macOS icon: transparent margin, opaque black body, centred white square with black around it."""
    width, height, rows = image
    assert (width, height) == (size, size), f"size is {width}x{height}"
    pad, _, side = square(rows)
    margin = size * (1 - MAC_BODY) / 2
    assert all(rows[y][x * 4 + 3] == 0 for x, y in ((0, 0), (size - 1, 0), (0, size - 1), (size - 1, size - 1)))
    assert all(row[i:i + 3] in (b"\x00\x00\x00", b"\xff\xff\xff") for row in rows for i in range(0, len(row), 4))
    edge = int(margin) + 1
    middle = rows[size // 2]
    assert middle[edge * 4:edge * 4 + 4] == b"\x00\x00\x00\xff", "body edge is not opaque black"
    assert middle[3] == 0 or margin < 1, "margin is not transparent"
    assert pad - margin >= 3, f"only {pad - margin:.1f} px of black beside the square"
    share = side / (size * MAC_BODY)
    assert MAC_SHARE[0] <= share <= MAC_SHARE[1], f"square share of body {share:.3f}"
    return f"{size} px: square {side} px ({share:.1%} of the body), black beside it {pad - margin:.1f} px"


def mac_files():
    """The iconset file name of each pixel size, 1x and 2x."""
    names = {}
    for points in (16, 32, 128, 256, 512):
        names[f"icon_{points}x{points}.png"] = points
        names[f"icon_{points}x{points}@2x.png"] = points * 2
    return names


def run():
    """Checks every file and prints one line per image."""
    iconset = ROOT / "macos/AppIcon.iconset"
    assert sorted(p.name for p in iconset.iterdir()) == sorted(mac_files()), "iconset file list"
    assert sorted(set(mac_files().values())) == list(MAC_SIZES)
    for name, size in mac_files().items():
        print(f"macos {name}:", check_mac(decode_png((iconset / name).read_bytes()), size))
    entries = decode_ico((ROOT / "windows/abstractions.ico").read_bytes())
    assert [size for size, _, _ in entries] == WIN_SIZES, "ico sizes"
    for size, packed, image in entries:
        assert packed == (size == 256), "only the 256 px image is stored as PNG"
        print("windows ico", "png" if packed else "bmp", check_tile(image, size))
    for size in LINUX_SIZES:
        path = ROOT / f"linux/share/icons/hicolor/{size}x{size}/apps/abstractions.png"
        print("linux", check_tile(decode_png(path.read_bytes()), size))


if __name__ == "__main__":
    try:
        run()
    except AssertionError as error:
        sys.exit(f"icon test failed: {error}")
    print("all icons pass")
