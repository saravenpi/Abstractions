# Formats of 0.11: protocol 3 and `SAVE_VERSION` 3

One note for steps 6, 16, 17 and 17b to 17d of `PLANETS.md`. The net crate, the saves and the map agree on these shapes.

## Positions on the wire (step 6)
- `PROTOCOL` is 3. Older clients are refused by the join check that already compares it.
- A position in `Frame::Surface` is three cells and three millimetre offsets. A cell is a signed whole number of 2,000,000 m per axis (`pack::CELL`). The offset is an `i32` count of millimetres from the cell centre, so it never passes 1e9.
- Cells travel as zigzag LEB128 varints, one byte while the position is within 126,000 km of the origin on that axis. Then three little-endian `i32`. A robot body is 44 bytes, a ship 40, a companion 28.
- `Frame::Around` and `Frame::System` stay three `f64`. A galaxy frame position (saved as three `f64` too, `ShipState.pos`) is lossless to the step of a 64-bit float: 6e4 m at 5e20 m and 4e6 m at 2.4e22 m from the galaxy centre. The wire has no galaxy frame: between stars a record is in the frame of the last star.
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

## Hunting and the field on the wire (wave H4, `net-h4`)

Still protocol 3: nothing of it was released. An old client is refused at the join check, and a robot or ship record of the old length does not decode (a body must have exactly its length).

### State records (datagrams)
- Robot: 46 bytes on a surface (was 44), 55 in the other frames. New: `swing` (1 byte: act rank in bits 0 to 2 as ready, wind-up, swing, recover, guard, draw, stun; side in bits 3 and 4 as right, left, up, down) and one byte with `crouch` (high 4 bits, steps of 1/15) and `phase` (low 4 bits, seconds into the act, 0 to 2 s in steps of 0.13 s). Sixteen robot records fit one datagram of 1,100 bytes (test `sixteen_robots_fit_one_datagram`).
- Ship: 41 bytes on a surface (was 40), 50 elsewhere. New: `heat`, the glow of an entry, 1 byte.
- New kind `Vitals` (11 bytes in every frame, about twice a second, sent in the frame of the robot record): the harm of the ten parts, one byte each, 0 whole to 255 destroyed, in the order head, torso, left arm, right arm, left leg, right leg, left wing, right wing, left boot, right boot, then the entry glow of the robot (1 byte). It travels apart so that the robot record stays small.
- The room keeps four kinds per player now (`KINDS` is 4).

### Reliable messages (stream)
`ClientMsg::Hunt(Act)` (number 8) and `ServerMsg::Hunt(Event)` (number 10) are the only new messages. The server stamps the sending player and never lets a client speak for another. A player may send 30 acts a second.

| Act (client) | Event (server) | What the server does | Size on the wire, worst case, with the 4-byte length |
| --- | --- | --- | --- |
| `Hit` animal, part, joules, blunt | `Hit` by, hit | remembers who hurt the animal, tells the others | 40 |
| `Broke` animal, part | `Broke` | keeps the broken parts of the living animal, tells the others | 36 |
| `Kill` animal, species, direction, time, grade, lasts | `Killed` carcass (to everyone, the killer too) | lays the carcass once, with the broken parts and the players who hurt it in the 30 s before; a time more than 5 s from the clock of the server is replaced by it | 80, carcass 110 |
| `Carve` animal, part | `Carved` | takes the part once | 36 |
| `Stick` spear (key, animal, point, direction, item) | `Stuck` | keeps it until pulled or until its carcass is gone | item + 60 |
| `Pull` key | `Pulled` | removes it | 40 |
| `Held` item or nothing | `Held` | keeps it with the seat, tells the others | item + 8 |
| `Blow` victim, point, joules, cut, direction, momentum | `Blow` (to the victim only) | relays; the victim decides what it does and says it in its records | 50 |
| `Wound` animal, wound | `Wound` | keeps it until the animal dies or 30 minutes pass | 60 |
| `Plug` animal, part | `Plug` | marks the wounds of the part plugged | 36 |
| `Impact` body, point, joules, cut, outer, tool, result | `Impact` | relays only | 70 |
| `Attack` animal, kind, target, start time | `Attack` | relays only | 50 |
| `Rock` planet, id, grams, whole | `Rock` | adds the grams to what was taken of the rock, never past `whole`, and tells the others; a rock with nothing left is not told | 48 |

- A `Rock` is a piece, in grams, with the mass of the whole rock in grams (`u32` each, so a rock may weigh up to 4,294 tonnes). In `HuntWorld.rocks` one record holds the total taken of its rock. The server keeps the total of each rock, so a joining player tops up what its own world already took.
- In an `Impact`, `outer` is the position of the layer in `[flesh, hide, shell, plate, bone, horn, robot plate, ground]`, and `tool` is one byte: the stiffness of the head in the high four bits and its mass in the low four, each as a power of two (stiffness `2^(n-4)` GPa, mass `2^(n-6)` kg). `result` is `CLEAN` or `DEFLECTED`. The point is in metres in the turning frame of `body`.
- In an `Attack`, `kind` is the position in `[trample, sweep, swipe, lunge, bite, charge]`.
- An item is the bytes of its craft recipe in the codec of the streams (about 72 bytes), at most 96. The server never reads it.
- After the welcome, a server sends `Event::World` when there is anything to tell: the carcasses with their taken and broken parts, the stuck spears, what each player holds, the open wounds and every rock taken. It sends nothing when all is empty.
- A carcass lasts the `lasts` its killer gave, clamped from 60 s to a day, and goes with the spears stuck in it. At most 1,024 carcasses, 256 spears, 2,048 wounds and 65,536 rocks are kept.
- The server file `hunt3.bin` (postcard of `HuntWorld`, without the held items) keeps carcasses, spears, wounds and rocks beside `world.toml`. The server writes the world only when something changed since the last save.
- Open choice: loot. The shared mask of taken parts makes the first cutter win a part. The list of credited players travels in `Killed` so that a later change can give each of them a personal harvest.
