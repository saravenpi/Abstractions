# Icon font

`assets/fonts/material-symbols-sharp-subset.ttf` is Material Symbols Sharp (Apache-2.0, `LICENSES/MaterialSymbols-Apache-2.0.txt`), instanced at weight 400, FILL 0, GRAD 0, optical size 24, and cut to the glyphs of `src/ui/icon.rs`. Sharp corners, no round caps.

Source: https://github.com/google/material-design-icons (`variablefont/MaterialSymbolsSharp[FILL,GRAD,opsz,wght].ttf` and the `.codepoints` file beside it).

Rebuild, with fontTools installed. `UNICODES` is the comma list of `U+xxxx` codepoints of `CODEPOINTS` in `icon.rs`:

```sh
fonttools varLib.instancer MaterialSymbolsSharp.ttf FILL=0 GRAD=0 opsz=24 wght=400 -o instance.ttf
pyftsubset instance.ttf --unicodes="$UNICODES" --layout-features= --no-hinting --desubroutinize \
  --output-file=assets/fonts/material-symbols-sharp-subset.ttf
```

To add an icon: find its name in the `.codepoints` file, add the variant and codepoint to `icon.rs`, rebuild with the new list. `icon_tests.rs` checks that every codepoint has a glyph.

The four ship icons use the closest Material glyphs: `rocket` parked, `navigation` flying, `fast_forward` in warp, `build` damaged. Replace them with white-mask PNGs in `assets/icons/` if the owner wants the game's own ship.
