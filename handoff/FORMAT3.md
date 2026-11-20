# Formats of 0.11: protocol 3 and `SAVE_VERSION` 3

One note for steps 6, 16, 17 and 17b to 17d of `PLANETS.md`. The net crate, the saves and the map agree on these shapes.

## Positions on the wire (step 6)
- `PROTOCOL` is 3. Older clients are refused by the join check that already compares it.
- A position in `Frame::Surface` is three cells and three millimetre offsets. A cell is a signed whole number of 2,000,000 m per axis (`pack::CELL`). The offset is an `i32` count of millimetres from the cell centre, so it never passes 1e9.
- Cells travel as zigzag LEB128 varints, one byte while the position is within 126,000 km of the origin on that axis. Then three little-endian `i32`. A robot body is 44 bytes, a ship 40, a companion 28.
- `Frame::Around` and `Frame::System` stay three `f64`.
- No `f32` chart position is ever sent.

## Explored ground (step 17b)
- The grid of 512 x 256 bits is gone. Each body has a quadtree on the cube-sphere (six faces). A leaf is a `u64` key, always under 2^63 so TOML can hold it:
  - bits 62..60: the face, 0 to 5;
  - bits 59..54: the depth, 1 to 27;
  - bits 53..0: the Morton code of the leaf's column and row, `x` in the even bits.
- The game picks one depth per body so a leaf is at most 500 m across: `ceil(log2(pi * R / 2 / 500))`.
- Leaves live in a set of runs (`abstractions_net::leaves::Leaves`). A run is `Run { gap, len }`: `gap` is the distance from the end of the previous run (from 0 for the first), `len` is the number of consecutive keys. Runs are what the wire, `explored3.bin` and the save carry.
- Wire: `ClientMsg::Explored { body, runs }`, `ServerMsg::Explored { body, runs }` and `Welcome.explored: Vec<(BodyId, Vec<Run>)>`.
- Server file: `explored3.bin`, postcard `Vec<(BodyId, Vec<Run>)>`. The old `explored.bin` is ignored.
- Save: `[[explored]]` with `body` and `runs`. `EXPLORED_COLUMNS` and `EXPLORED_ROWS` are removed.

## Worlds (step 16)
- `SAVE_VERSION` is 3. `WorldSave.game_version` holds the version of the game that made the world (`CARGO_PKG_VERSION`).
- A file with `version < 3` is an old world. It lists as "Made with 0.10", it is never opened by the new generator, and the 0.10 to 0.9 upgrade code is deleted with it. The label is `WorldSave::made_with()`, which returns `0.10` for a file with no `game_version`.
- A world made by this game is written with its `game_version`. A world with a newer `version` than this game knows keeps its own label.

## Older releases (step 17)
- `src/update/older.rs` plans "Open with 0.10": it asks the release host for the tag of the release, takes the asset for this platform, downloads it to `<data>/abstractions/older/<version>/`, checks the SHA-256 against the release's checksum, and launches the binary with `--world <id>`.
- The game accepts `--world <id>` from 0.11.0: it opens that world from the worlds folder instead of the title screen. The id is the file stem.
