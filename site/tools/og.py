"""Render site/og.png from the hero screenshot and report the text contrast.

Needs ImageMagick 7 (`magick`) and fontTools with brotli. Run from anywhere:
    python3 site/tools/og.py
"""

import subprocess
import sys
import tempfile
from pathlib import Path

from fontTools.ttLib import TTFont

SITE = Path(__file__).resolve().parent.parent
WIDTH, HEIGHT = 1200, 630
MARGIN = 72
TITLE = "ABSTRACTIONS"
TITLE_SIZE = 120
SUBTITLE = "A gigantic universe, and every world is its own"
SUBTITLE_SIZE = 30
SUBTITLE_COLOR = "#c8c8c8"
CAP = 0.7
ADVANCE = 0.6


def magick(*args):
    """Run ImageMagick and return its standard output as bytes."""
    return subprocess.run(["magick", *args], check=True, capture_output=True).stdout


def font_file(folder):
    """Decompress the site's woff2 font into a TrueType file ImageMagick can read."""
    font = TTFont(SITE / "fonts" / "undefined-medium.woff2")
    font.flavor = None
    path = Path(folder) / "undefined-medium.ttf"
    font.save(path)
    return path


def layout():
    """Return the mark box and the baseline and box of each text line."""
    title_cap = round(TITLE_SIZE * CAP)
    sub_cap = round(SUBTITLE_SIZE * CAP)
    sub_base = HEIGHT - MARGIN
    title_base = sub_base - sub_cap - 44
    mark = (MARGIN, title_base - title_cap, title_cap, title_cap)
    title_x = MARGIN + title_cap + 36
    title = (title_x, title_base, (title_x, title_base - title_cap, round(len(TITLE) * TITLE_SIZE * ADVANCE), title_cap))
    subtitle = (MARGIN, sub_base, (MARGIN, sub_base - sub_cap, round(len(SUBTITLE) * SUBTITLE_SIZE * ADVANCE), sub_cap))
    return mark, title, subtitle


def background(out):
    """Crop the screenshot to fill the card and darken it towards the text."""
    shade = (
        "-size", f"{WIDTH}x{HEIGHT}", "gradient:",
        "-fx", "0.02+0.8*pow(min(1,max(0,(j/h-0.42)/0.3)),1.2)",
        "-alpha", "off",
    )
    magick(
        str(SITE / "hero.jpg"), "-resize", f"{WIDTH}x{HEIGHT}^", "-gravity", "center", "-extent", f"{WIDTH}x{HEIGHT}", "+gravity",
        "(", "-size", f"{WIDTH}x{HEIGHT}", "xc:black", "(", *shade, ")", "-compose", "CopyOpacity", "-composite", ")",
        "-compose", "over", "-composite", str(out),
    )


def luminance(path, box):
    """Return the highest and mean WCAG relative luminance inside a box."""
    x, y, w, h = box
    raw = magick(str(path), "-crop", f"{w}x{h}+{x}+{y}", "+repage", "-depth", "8", "rgb:-")

    def linear(c):
        c /= 255
        return c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4

    values = [
        0.2126 * linear(raw[i]) + 0.7152 * linear(raw[i + 1]) + 0.0722 * linear(raw[i + 2])
        for i in range(0, len(raw), 3)
    ]
    return max(values), sum(values) / len(values)


def contrast(color, lum):
    """Return the WCAG contrast ratio of a grey hex colour over a luminance."""
    c = int(color[1:3], 16) / 255
    fg = c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4
    return (max(fg, lum) + 0.05) / (min(fg, lum) + 0.05)


def main():
    """Build og.png and print the worst and mean contrast of each text line."""
    mark, title, subtitle = layout()
    with tempfile.TemporaryDirectory() as folder:
        font = font_file(folder)
        base = Path(folder) / "base.png"
        background(base)
        mx, my, mw, mh = mark
        magick(
            str(base), "-fill", "white", "-draw", f"rectangle {mx},{my} {mx + mw - 1},{my + mh - 1}",
            "-font", str(font), "-kerning", "0",
            "-pointsize", str(TITLE_SIZE), "-annotate", f"+{title[0]}+{title[1]}", TITLE,
            "-fill", SUBTITLE_COLOR, "-pointsize", str(SUBTITLE_SIZE), "-annotate", f"+{subtitle[0]}+{subtitle[1]}", SUBTITLE,
            "-strip", "-define", "png:compression-level=9", str(SITE / "og.png"),
        )
        for name, color, box in (("title", "#ffffff", title[2]), ("subtitle", SUBTITLE_COLOR, subtitle[2])):
            worst, mean = luminance(base, box)
            print(f"{name}: {contrast(color, worst):.1f}:1 worst pixel, {contrast(color, mean):.1f}:1 mean")
            if contrast(color, worst) < 4.5:
                sys.exit(f"{name} contrast below 4.5:1")


if __name__ == "__main__":
    main()
