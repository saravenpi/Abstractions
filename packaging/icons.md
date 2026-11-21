# Icon font

`assets/fonts/material-symbols-sharp-subset.ttf` is Material Symbols Sharp (Apache-2.0, `LICENSES/MaterialSymbols-Apache-2.0.txt`), instanced at weight 400, FILL 0, GRAD 0, optical size 24, and cut to the glyphs of `src/ui/icon.rs`. Sharp corners, no round caps.

Source: https://github.com/google/material-design-icons at commit `10ecc14790612ca8fd164e3f4ffe9c16fc50d5b7` (2026-10-23, font version 2.978): `variablefont/MaterialSymbolsSharp[FILL,GRAD,opsz,wght].ttf` (8 918 100 bytes, SHA-256 `1fd72f99ec1f44ec461c74f99341c1dcfa8c140d7087a2902e747d4ee01e379b`) and the `.codepoints` file beside it. The licence file is the `LICENSE` of that repository, byte for byte.

Rebuild, with fontTools installed:

```sh
python3 packaging/icons.py          # writes the font and src/ui/icon/codes.rs
python3 packaging/icons.py --check  # exits 1 when either is stale
```

The script downloads the two source files once to `~/.cache/abstractions-icons/`, checks the SHA-256, instances the font at the axis values above and cuts it to the icons of the enum. It does what these two commands do:

```sh
fonttools varLib.instancer MaterialSymbolsSharp.ttf FILL=0 GRAD=0 opsz=24 wght=400 -o instance.ttf
pyftsubset instance.ttf --unicodes="$UNICODES" --layout-features= --no-hinting --desubroutinize \
  --output-file=assets/fonts/material-symbols-sharp-subset.ttf
```

To add an icon: add its variant to `Icon` in `src/ui/icon.rs` with one doc line that gives its Material Symbols name between backticks, then run the script. It refuses a name the font lacks and two icons on one glyph. `icon_tests.rs` checks that every codepoint has a glyph.

Sizes: 53 glyphs took 8 140 bytes; 132 glyphs take 22 124 bytes.

The four ship icons use the closest Material glyphs: `rocket` parked, `navigation` flying, `fast_forward` in warp, `build` damaged. Replace them with white-mask PNGs in `assets/icons/` if the owner wants the game's own ship.
