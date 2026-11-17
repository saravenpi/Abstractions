#!/usr/bin/env python3
"""Generates every Abstractions icon file: a white square on black.

Usage: make.py [--check]

Without a flag it rewrites the icon files under packaging/. With --check it
writes nothing and exits 1 when a committed file differs from what it generates.
Standard library only. Run test.py afterwards to read the files back.

Each platform has two inputs: its list of canvas sizes and the white square's
share. On macOS the share is of the icon body, which Apple's grid fixes at
824/1024 of the canvas. On Windows and Linux it is of the full black tile.
"""

import math
import struct
import sys
import zlib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

MAC_SHARE = 0.38
WIN_SHARE = 0.42
LINUX_SHARE = 0.42

MAC_BODY = 824 / 1024
MAC_RADIUS = 185 / 1024
MAC_SIZES = (16, 32, 128, 256, 512)
WIN_SIZES = (16, 24, 32, 48, 64, 256)
LINUX_SIZES = (16, 32, 48, 64, 128, 256, 512)

BLACK = b"\x00\x00\x00\xff"
WHITE = b"\xff\xff\xff\xff"


def side(size, share):
    """Width of the white square: nearest to size * share with the parity of size, so it centres on whole pixels."""
    odd = size % 2
    return max(2, math.floor((size * share - odd) / 2 + 0.5) * 2 + odd)


def tile(size, share):
    """Rows of a full-bleed black tile with the centred white square."""
    width = side(size, share)
    pad = (size - width) // 2
    assert pad >= 3, (size, pad)
    plain = BLACK * size
    marked = BLACK * pad + WHITE * width + BLACK * pad
    return [marked if pad <= y < pad + width else plain for y in range(size)]


def distance(x, y, size):
    """Signed distance in pixels from a point to the edge of the macOS icon body, negative inside."""
    half = size * MAC_BODY / 2
    radius = size * MAC_RADIUS
    qx = abs(x - size / 2) - (half - radius)
    qy = abs(y - size / 2) - (half - radius)
    return math.hypot(max(qx, 0), max(qy, 0)) + min(max(qx, qy), 0) - radius


def coverage(x, y, size):
    """Alpha of one pixel of the macOS icon body, antialiased with 8 by 8 samples on the edge."""
    centre = distance(x + 0.5, y + 0.5, size)
    if centre <= -0.75:
        return 255
    if centre >= 0.75:
        return 0
    steps = [(i + 0.5) / 8 for i in range(8)]
    inside = sum(distance(x + dx, y + dy, size) <= 0 for dy in steps for dx in steps)
    return math.floor(inside * 255 / 64 + 0.5)


def mac(size):
    """Rows of the macOS icon: a black rounded body on a transparent canvas, white square in the centre."""
    width = side(size, MAC_SHARE * MAC_BODY)
    pad = (size - width) // 2
    rows = []
    for y in range(size):
        row = bytearray()
        for x in range(size):
            row += b"\x00\x00\x00" + bytes([coverage(x, y, size)])
        if pad <= y < pad + width:
            row[pad * 4:(pad + width) * 4] = WHITE * width
        rows.append(bytes(row))
    return rows


def chunk(kind, data):
    """One PNG chunk."""
    return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data))


def png(rows):
    """An 8-bit RGBA PNG from rows of RGBA bytes."""
    size = len(rows)
    header = struct.pack(">IIBBBBB", size, size, 8, 6, 0, 0, 0)
    raw = b"".join(b"\x00" + row for row in rows)
    return b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", header) + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b"")


def bitmap(rows):
    """An ICO bitmap entry: 32-bit BGRA, bottom-up, followed by an empty AND mask."""
    size = len(rows)
    header = struct.pack("<IiiHHIIiiII", 40, size, size * 2, 1, 32, 0, size * size * 4, 0, 0, 0, 0)
    pixels = bytearray()
    for row in reversed(rows):
        for i in range(0, len(row), 4):
            pixels += bytes((row[i + 2], row[i + 1], row[i], row[i + 3]))
    mask = bytes(((size + 31) // 32) * 4) * size
    return header + bytes(pixels) + mask


def ico(sizes, share):
    """An ICO container: bitmaps below 256 px, the 256 px image stored as PNG."""
    images = [png(tile(s, share)) if s >= 256 else bitmap(tile(s, share)) for s in sizes]
    offset = 6 + 16 * len(sizes)
    out = struct.pack("<HHH", 0, 1, len(sizes))
    for size, image in zip(sizes, images):
        out += struct.pack("<BBBBHHII", size % 256, size % 256, 0, 0, 1, 32, len(image), offset)
        offset += len(image)
    return out + b"".join(images)


def files():
    """Every generated file as a map from its path under packaging/ to its bytes."""
    out = {"windows/abstractions.ico": ico(WIN_SIZES, WIN_SHARE)}
    for size in MAC_SIZES:
        out[f"macos/AppIcon.iconset/icon_{size}x{size}.png"] = png(mac(size))
        out[f"macos/AppIcon.iconset/icon_{size}x{size}@2x.png"] = png(mac(size * 2))
    for size in LINUX_SIZES:
        out[f"linux/share/icons/hicolor/{size}x{size}/apps/abstractions.png"] = png(tile(size, LINUX_SHARE))
    return out


def pixels(data):
    """Decoded scanlines of a PNG, so a check does not depend on the zlib build that compressed it."""
    idat, at = b"", 8
    while at < len(data):
        length, kind = struct.unpack(">I4s", data[at:at + 8])
        if kind == b"IDAT":
            idat += data[at + 8:at + 8 + length]
        at += 12 + length
    return data[8:33], zlib.decompress(idat)


def same(name, want, have):
    """True when a committed file holds the same image as the generated one."""
    if want == have:
        return True
    if not name.endswith(".png") or not have.startswith(b"\x89PNG"):
        return False
    try:
        return pixels(want) == pixels(have)
    except (zlib.error, struct.error):
        return False


def main():
    """Writes the files, or with --check compares them and reports the ones that drifted."""
    check = sys.argv[1:] == ["--check"]
    if sys.argv[1:] and not check:
        sys.exit(__doc__)
    stale = []
    for name, data in files().items():
        path = ROOT / name
        if check:
            if not path.is_file() or not same(name, data, path.read_bytes()):
                stale.append(name)
            continue
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)
        print(f"wrote packaging/{name}")
    if stale:
        sys.exit("out of date, run packaging/icon/make.py:\n" + "\n".join(f"  packaging/{n}" for n in stale))
    if check:
        print("icons are up to date")


if __name__ == "__main__":
    main()
