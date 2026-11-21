#!/usr/bin/env python3
"""Builds the icon font of the game and the table of its codepoints.

Usage: icons.py [--check] [--font PATH --codepoints PATH]

The source of truth is the list in src/ui/icon/names.rs: one line an icon, its
variant of `Icon` and the Material Symbols name of its glyph. This script reads
those lines, looks every name up in the `.codepoints` file of the font, and
writes two files:

  src/ui/icon/codes.rs                           the codepoint of every icon
  assets/fonts/material-symbols-sharp-subset.ttf the glyphs of those icons only

To add an icon: add its line to names.rs, then run this script.

The font is Material Symbols Sharp (Apache-2.0, LICENSES/MaterialSymbols-Apache-2.0.txt),
the variable font of Google's repository at the commit pinned below, instanced at the
axis values of AXES. It is downloaded once to ~/.cache/abstractions-icons/ and checked
against SHA256. Pass --font and --codepoints to use local copies instead.

With --check nothing is written: the script exits 1 when codes.rs or the glyphs of the
committed font differ from what it would generate. Needs fontTools.
"""

import hashlib
import io
import re
import sys
import urllib.request
from pathlib import Path

from fontTools import subset
from fontTools.ttLib import TTFont
from fontTools.varLib import instancer

ROOT = Path(__file__).resolve().parent.parent
NAMES = ROOT / "src/ui/icon/names.rs"
CODES = ROOT / "src/ui/icon/codes.rs"
SUBSET = ROOT / "assets/fonts/material-symbols-sharp-subset.ttf"

COMMIT = "10ecc14790612ca8fd164e3f4ffe9c16fc50d5b7"
BASE = f"https://raw.githubusercontent.com/google/material-design-icons/{COMMIT}/variablefont/"
STEM = "MaterialSymbolsSharp%5BFILL%2CGRAD%2Copsz%2Cwght%5D"
SHA256 = "1fd72f99ec1f44ec461c74f99341c1dcfa8c140d7087a2902e747d4ee01e379b"
AXES = {"FILL": 0, "GRAD": 0, "opsz": 24, "wght": 400}
CACHE = Path.home() / ".cache/abstractions-icons" / COMMIT


def fetch(suffix):
    """Returns the bytes of the source file with `suffix`, from the cache or the network."""
    kept = CACHE / f"sharp{suffix}"
    if not kept.exists():
        CACHE.mkdir(parents=True, exist_ok=True)
        with urllib.request.urlopen(BASE + STEM + suffix) as answer:
            kept.write_bytes(answer.read())
    return kept.read_bytes()


def sources(args):
    """Returns the bytes of the variable font and the text of its codepoints file."""
    if "--font" in args:
        font = Path(args[args.index("--font") + 1]).read_bytes()
        names = Path(args[args.index("--codepoints") + 1]).read_text()
        return font, names
    font = fetch(".ttf")
    if hashlib.sha256(font).hexdigest() != SHA256:
        sys.exit(f"the font in {CACHE} does not match SHA256: delete it and run again")
    return font, fetch(".codepoints").decode()


def icons():
    """Returns the variants of `Icon` with the Material name of each, in declaration order."""
    lines = [line.strip() for line in NAMES.read_text().split("icons! {", 1)[1].splitlines()]
    lines = [line for line in lines if line and line != "}"]
    found = [re.fullmatch(r'([A-Z][A-Za-z0-9]*) => "([a-z0-9_]+)",', line) for line in lines]
    if not all(found):
        sys.exit("every line of names.rs must read: Variant => \"material_name\",")
    return [(match[1], match[2]) for match in found]


def table(pairs, names):
    """Returns the icons with their codepoints, refusing a name the font lacks or a shared glyph."""
    known = dict(line.split() for line in names.splitlines() if line.strip())
    missing = [name for _, name in pairs if name not in known]
    if missing:
        sys.exit(f"not in the font: {', '.join(missing)}")
    rows = [(variant, known[name]) for variant, name in pairs]
    codes = [code for _, code in rows]
    shared = sorted({code for code in codes if codes.count(code) > 1})
    if shared:
        sys.exit(f"two icons share a codepoint: {', '.join(shared)}")
    return rows


def rust(rows):
    """Returns the text of codes.rs for `rows`, as cargo fmt leaves it."""
    points = "".join(f"    (Icon::{variant}, '\\u{{{code}}}'),\n" for variant, code in rows)
    return (
        "//! The codepoint of every icon in the icon font. Written by `packaging/icons.py`: do not\n"
        "//! edit by hand.\n\n"
        "use super::Icon;\n\n"
        "/// Every icon with its codepoint in the icon font, in declaration order.\n"
        f"pub const CODEPOINTS: [(Icon, char); {len(rows)}] = [\n{points}];\n"
    )


def cut(font, rows):
    """Returns the bytes of the font instanced at AXES and cut to the glyphs of `rows`."""
    instance = instancer.instantiateVariableFont(TTFont(io.BytesIO(font)), AXES)
    options = subset.Options()
    options.layout_features = []
    options.hinting = False
    options.desubroutinize = True
    cutter = subset.Subsetter(options)
    cutter.populate(unicodes=[int(code, 16) for _, code in rows])
    cutter.subset(instance)
    lacking = [variant for variant, code in rows if int(code, 16) not in instance.getBestCmap()]
    if lacking:
        sys.exit(f"no glyph in the font for: {', '.join(lacking)}")
    out = io.BytesIO()
    instance.save(out)
    return out.getvalue()


def outlines(data):
    """Returns the outline of every mapped glyph of the font in `data`, by codepoint."""
    font = TTFont(io.BytesIO(data))
    glyphs = font["glyf"]
    return {
        code: list(glyphs[name].getCoordinates(glyphs)[0])
        for code, name in font.getBestCmap().items()
    }


def main(args):
    """Builds both files, or compares them with the committed ones under --check."""
    font, names = sources(args)
    rows = table(icons(), names)
    code, data = rust(rows), cut(font, rows)
    if "--check" in args:
        stale = CODES.read_text() != code or outlines(SUBSET.read_bytes()) != outlines(data)
        sys.exit("codes.rs or the icon font is stale: run packaging/icons.py" if stale else 0)
    before = SUBSET.stat().st_size if SUBSET.exists() else 0
    CODES.write_text(code)
    SUBSET.write_bytes(data)
    print(f"{len(rows)} icons, font {before} -> {len(data)} bytes")


if __name__ == "__main__":
    main(sys.argv[1:])
