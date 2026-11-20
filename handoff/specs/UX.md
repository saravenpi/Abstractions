# Map, computer and HUD: UX review and redesign spec

Worktree: /home/yann/Code/Saravenpi/wt-int (branch `integrate`, HEAD fbfe95c). Read-only review. Line numbers are from that HEAD.

## 1. Findings, ranked by how much they confuse a new player

1. Going somewhere takes two commands plus a flight lesson. "Set course" (Enter, card button, right-click row) only stores a target (`computer/keys.rs:145`, `map/deeds.rs:39`). "Engage" (G) is a second command that appears only when the route is `Idle` (`deeds.rs:44`, `map/action.rs:12`). The player must also guess that the two are separate.
2. The common first try is blocked, and the words are wrong or jargon.
   - On foot, with the ship parked, the route status is `Blocked(Landed)`, so the card says "Landed: take off first" (`computer/route.rs:66`). The player is not in the ship. The real fix is "board your ship".
   - In the air of another planet the card says "climb to orbit first". The ship already knows how to climb (`ship/route/ascent.rs`), but only when the target is the planet it stands on (`ascent.rs:52`). Going to Mars from Earth air is a manual flight lesson, going to Earth is automatic.
   - "Inside 1.2 planet radii (now 0.8): fly out first" asks the player to compute radii.
3. The state is invisible off the ship. The course row of the HUD shows only while seated (`hud/prompt/course.rs:74`). On foot, a set course shows nothing on the compass, the minimap marks it only in space (`minimap/space.rs`), and the world has no marker. The compass has no course marker at all (`hud/compass/aim.rs`).
4. Hidden keys and modes, none taught anywhere in play.
   - M, Tab (seated only), G, B, N, Home (go home), Delete (clear course), `/` (search), F2 (sort), Backspace (up one level), Left/Right (switch System/Galaxy/Universe, but only when the pointer is not over the chart, `keys.rs:68`).
   - The map has no key legend. The only teaching text is `NO_COURSE` (`computer/course.rs:24`). No first-run hints exist in `src/menus/hud`.
   - The tabs have no buttons: nothing on screen says Galaxy and Universe exist.
   - Home does not mean "go home". It means "next leg toward home" (`computer/home.rs:23` `toward`). A trip from another galaxy needs Tab, Home, Esc, G, three times. A laptop has no Home key.
5. Two systems for one idea. Waypoint (B, one only, toggles, overwritten silently by the next B, no name, compass and minimap only) and Favourite (star button, up to unlimited, name editing, five shown, compass only within 2 km, `aim.rs:24`). A favourite cannot be followed past 2 km. A waypoint cannot be the destination of a flight. Both flights stop at the body, not the pin (`menu/act.rs:63` maps `Mine`, `Favourite`, `Player` to `Target::Body`).
6. Favourites 6 and up are saved but unreachable: the panel shows `FAVOURITES = 5` rows at most (`map/side.rs:9`, `computer/chrome.rs:21`). Silent data hiding.
7. Right-click menu with nine choices (`map/menu/mod.rs:95`): Set course, Focus, Set waypoint, Clear waypoint, Save as favourite, Copy coordinates, Centre here, Rename, Delete. Different rows and the ground offer different subsets. It is the only way to reach some actions (Focus, Copy). Left click selects, double click dives, slow second click deselects (`glance/mod.rs:58`, `pointer/click.rs:48`). Three click behaviours, none shown.
8. Icon-only buttons that look like decoration: favourite row has Send, Pencil, Trash with no text (`side.rs:80-84`). "Send" means "set course", a meaning no player guesses. The Back button duplicates the breadcrumb and Backspace (`nav/bar.rs`).
9. Cancel is hidden and unclear. Manual warp key J cancels the autopilot (`ship/helm.rs:140`), shown nowhere. Delete in the map clears `Destination`, but nothing in `route.rs` reads that to stop the flight, so the HUD row vanishes while the ship may keep flying. There is no "Stop" on the HUD.
10. Friends: go-to-friend is a Players list that appears only when someone is connected (`side_show.rs`), then a click that focuses the map or, off-planet, silently just changes level (`side_act.rs:84`), then right-click, Set course, which leads to orbit of their planet. Landing near them is manual.
11. HUD jargon and clutter. Ten instrument rows: ALT, SPD, THR, ASSIST, FRAME ("planet", "star system", "between stars"), APO, PERI, WARP, DEST, GEAR (`hud/flight.rs:21`). Route labels "blocked", "dropping out", "spooling", "aligning" (`route.rs:51`). Warp and destination are duplicated in the course row and the DEST row.
12. Overlap and edges. Frame-integrity corner sits at left 16, bottom 16 (`hud/frame.rs:19`). The instrument block sits at left 32, bottom 32 (`hud/panel.rs:81`). With two or more damaged parts and the ship seated they collide. Four edge rules coexist: 16 px (`GAP_M`), 32 px (`EDGE`), 22 % (prompt, `prompt.rs:45`), 14 % (banner, `banner.rs:50`).
13. Contrast. Compass labels, coordinates, climate line and frame lines are 14 px MUTED (white at 65 %) with no backing strip. Over a white sky or snow they vanish. Even with the strip (black 50 %), white text over a pure white sky is 4.0:1 and MUTED text 2.6:1, below the 4.5:1 target that `theme::TINT` states for menus. The compass rule is HAIRLINE (25 % white).
14. Hierarchy. 66 uses of `FONT_SMALL` (14), 24 of `FONT_BODY` (18), 6 of `FONT_HEADING`. In the map everything is 14 px, so nothing is primary: the card has a title at 18 and then 2 rows of equal buttons. No element is the one thing to do next.
15. Rule exceptions to confirm with the owner: a border and a full radius on orbit rings (`computer/orbits.rs:51-52`) and a bottom border on table rows (`ui/table.rs:20`). Both break "no borders, no rounded corners" unless the map picture is exempt.
16. Minor: B places the waypoint at the feet, not "where the player looks" as the key doc says (`settings/model.rs:179`, `map/me.rs:spot`). B toggles: with a waypoint set, B clears it even if the player meant to move it (`waypoint.rs:54`). Frame, climate and coords all use the same muted 14 px style, so weather and place cannot be told apart at a glance.

## 2. Task traces (today, from code)

"Action" = one key press or click. Manual flight is counted separately.

| Task | Steps today | Where a new player gets lost |
| --- | --- | --- |
| a. Other planet | Walk to ship, F, Tab (opens on the ground map), click the planet row (the list always shows the system), click "Set course" (or Enter), see "Landed: take off first" or "climb to orbit first", Esc, fly up manually (Space, Shift, steer, 30 to 60 s), watch HUD, press G. About 7 actions plus manual climb. | Set course vs Engage. Wrong reason on foot. Engage button has no key. The G prompt appears only after the orbit is reached, as a small line at 22 % height. |
| b. Moon | Same as a. Moons are indented rows under the planet in the System list. | Moon of the current planet = `moons::goal` standoff, no word says so. |
| c. Other star | Same as a, plus Left/Right to reach the Galaxy list (only with the pointer off the chart) or Back plus breadcrumb. Row says "Visit this system to chart its planets" for unknown stars. About 9 actions. | No tab buttons. "Visit to chart" is advice that cannot be followed from the card. |
| d. Other galaxy | As c with two tab steps. About 10 actions. | Same. |
| e. Home | Tab, Home (sets the next leg), Esc, G; repeat per leg (galaxy, star, planet). Up to 12 actions plus manual landing. | Home key unlabelled and missing on laptops. Autopilot ends in orbit, landing is manual. |
| f. Waypoint on ground | Near: B (at the feet). Elsewhere: M, hover the ground, B, Esc. Follow: watch compass (MapPin glyph plus distance, no name) and minimap dot. | No prompt for B. No name. Only one. |
| g. Remove waypoint | B (toggle) or right-click pin, "Clear waypoint". | B toggles, so to move it you press B twice. |
| h. Favourite | Save: M, click ground, click star "Save place", type name, Enter (about 5). Return: M, click row name (focus only), click Send icon (course to body only), then as a. | Five rows max. Send icon. Compass shows it only within 2 km. |
| i. Friend | M, click the player row (focus), right-click row, "Set course" (planet), then as a. | Player list hidden when alone. Ends in orbit of their planet. |
| j. Why blocked | Seated only: HUD course row `[ ] Landed: take off first`. The map card shows the same sentence in muted 14 px under the buttons. | No key, no fix, numbers (radii). Not shown on foot. |
| k. Cancel | J in the ship, nowhere shown. Delete in map clears the target. | Both hidden. |

## 3. Visual review per principle

- Hierarchy: no primary action. Fix: one inverted white block "Go" (18 px) on the card.
- Consistency: the same act has different keys and places. Course set: Enter, button, right-click. Home: Home key only. Cancel: J or Delete. Open map: M (everywhere), Tab (seated). One key per act below.
- Feedback: card shows route status as one muted line. HUD shows it only seated. No sound or banner when a course is accepted, blocked or finished (the banner shows planet names only).
- Recognition over recall: zero key legends in the map, keys on only 2 HUD prompts.
- Grid: HUD corner margins 16 and 32 px plus 22 % and 14 %. Map side panel 320 px with 16 px margin on an 8 px base: fine. Fix the HUD edge to one value.
- Sizes: 4 sizes (64, 28, 18, 14) plus the 14 px icon glyph. The scale is fine, usage is flat: keep 3 sizes in play (14, 18, 28).
- Contrast: see finding 13. STRIP at 0.6 gives 5.7:1 for TEXT over white sky. MUTED never reaches 4.5:1 over bright sky, so HUD text over the live game uses TEXT only.
- Density: 10 instrument rows, 9 menu choices, 4 deed buttons. Cut to 7 rows, 0 menu, 1 primary button plus 2 secondary.
- Empty states: Players block hides when empty (good), pins and favourites have no empty text, search shows "None charted yet".

## 4. Research (only pages opened)

- Elite Dangerous: choose a system on the galaxy map, "plot route", the line is orange and dotted when unreachable, then J charges the drive, a hollow marker means turn around until it is solid. Source https://roguey.co.uk/elite-dangerous/help/navigate. Applies: a reachable/unreachable cue before commit, one key to start, an on-HUD marker to align to. Elite needs many more steps, which is the cautionary case.
- No Man's Sky: select the target system on the galaxy map, "hold the button to start the jump", waypoints on systems. Source https://www.magicgameworld.com/no-mans-sky-next-the-galaxy-map-warping/. Applies: select then one hold, no separate "set course". Forum reports (not opened) of unremovable waypoints that are overwritten match our B problem.
- Breath of the Wild pins: pins are one-time beacons that dock on the minimap, vanish on arrival, can be converted to permanent stamps, and can be dropped by looking at a spot or from the map. Source https://stamen.com/cartographers-play-video-games-a-review-of-the-map-in-the-legend-of-zelda-breath-of-the-wild/ . Also https://gamedeveloper.com/art/a-ui-ux-analysis-of-zelda-breath-of-the-wilds (the map shows only discovered places and the quest marker). Applies: one pin kind, auto-removed on arrival, same pin on map, minimap and world.
- Nielsen's heuristics: visibility of status, error messages in plain words that name the fix, recognition over recall, consistency, minimalist design. Source https://www.nngroup.com/articles/ten-usability-heuristics/ .
- Game UI basics: button prompts so players need not memorize, immediate feedback, show information when it matters, same button to confirm everywhere, check that a new player can work out controls without help. Source https://unity.com/blog/video-game-ui-ux-basics .
- Four UI classes (non-diegetic, diegetic, spatial, meta): spatial markers reduce ambiguity without breaking immersion; non-diegetic need high contrast. Source https://www.toptal.com/designers/gui/game-ui . Applies: pins and the course are spatial (in the world as lights, already the owner's rule "every light is an object"), the card and the prompt line are non-diegetic.
- Unverifiable (opened none): Outer Wilds lock-on plus autopilot key (search snippets only; https://www.trueachievements.com/game/Outer-Wilds/walkthrough/2 returned HTTP 403), Star Citizen, Kerbal Space Program, Death Stranding. Not used as evidence.

## 5. The redesign

### 5.1 One flow: select, card, Go

1. Click a place (row, chart dot, pin, friend). The side card shows: name, kind line, distance, trip time, the one button, one sentence.
2. The card layout, top to bottom (320 px panel, 16 px margin, rows on the 8 px grid):
   - Title, 18 px, TEXT.
   - Kind line, 14 px, MUTED ("Moon of Mars").
   - Trip line, 14 px, TEXT: "1.2 Gkm - about 4 min" (`course::arrival`, `course.rs:31`).
   - Primary button, full width, 18 px, inverted (BLOCK white, INK text), label "Go" and the key cap `[G]` at its right. Disabled state: FAINT text, no white block.
   - Sentence, 14 px, TEXT, one line, wrapped to two at most: why not, or what happens next.
   - Facts below (existing `detail.rs` rows), unchanged.
3. Go does both steps: it sets `Destination` and sends `Engage`. Enter and G both press it. On success the map closes (`Computer::Closed`) and the HUD prompt line shows the progress (5.3). While a course runs, the same button reads "Stop" and G stops it (new `Cancel` event; autopilot `cancel()` at `pilot.rs`). J keeps cancelling as before.
4. On the ground level the same card shows for the picked row, so no "Back" is needed. Remove the Back button, keep Backspace and the breadcrumb (`nav/bar.rs`).
5. Level switching has one way: the breadcrumb (Universe > Galaxy > System > Planet), always visible, clickable. Delete the Left/Right tab switch and the tab state it needs (`keys.rs:63-68`, `Tab::step`).
6. Home: a button "Home" in the map header and key H (check `Keymap` for a free H; else keep Home). It selects the home body and shows the same card. The final target is `Body(home)` straight away if the autopilot can fly across systems; else the card says the leg ("Go to Sol first") and the HUD chains legs on `Phase::Arrived` (see WP-A risk).

### 5.2 Exact words for every block reason

All sentences are one line, plain, say the fix, use the live key names from settings (`key_name`). The card shows them under the disabled button. The HUD prompt line shows the same sentence without a key cap.

| Reason (code) | Today | New words |
| --- | --- | --- |
| New `Block::OnFoot` (player not seated, `Location.travel == OnFoot`), takes priority over every other block | "Landed: take off first" (wrong) | "Board your ship first: walk to the hatch and press F." (F = `keys.ship.board`) |
| `Block::Landed` (`block.rs:19`) | "Landed: take off first" | "Take off first: hold Space." (Space = `keys.ship.rise`) |
| `Block::Surface` (`block.rs:22`) | "In the air of a planet: climb to orbit first" | Delete the block. Go climbs to orbit by itself (WP-A). Sentence while climbing: "Climbing out of the air." |
| `Block::Inside { star: false }` (`block.rs:25`) | "Inside 1.2 planet radii (now 0.8): fly out first" | "Too close to the planet: fly away from it, then press G." |
| `Block::Inside { star: true }` | "Inside 5.0 star radii (now ...): fly out first" | "Too close to the star: fly away from it, then press G." |
| `Aligning(n)` | "Aligning 37%" | "Turning toward Mars." (progress bar, no percent) |
| `Spooling` | "Spooling the warp drive" | "Charging the warp drive." |
| `InWarp` | "In warp to Mars" | "Warping to Mars. G stops." |
| `Dropping` | "Dropping out of warp" | "Slowing down." |
| `Climbing` | "Climbing to orbit around Mars" | "Climbing out of the air." |
| `Arrived` | "Arrived at Mars" | "Arrived at Mars." (banner for 4.5 s, then the course clears: `route/done.rs`) |
| Idle, ready | "Engage autopilot to Mars" | Button "Go", sentence "About 4 min." |
| Target is where you are | none | Button disabled, "You are here." |
| Idle, no selection | "Right-click a place or press Enter to set a course" | "Click a place to see how to get there." |

Labels `RouteStatus::label` ("idle", "blocked", "aligning", ...) are deleted from the HUD: the sentence replaces them (`computer/route.rs:84`, `hud/flight.rs:55`).

### 5.3 Pins: one system for waypoints and favourites

Rename both to "Pin". One list, one icon (`Icon::MapPin`), stored in the world save (the existing `Favourite` records, `saves`); the single-slot `Waypoints.mine` is deleted. Friends stay a separate, read-only kind (`Icon::User`, their colour).

| Act | Input | Result |
| --- | --- | --- |
| Drop a pin | B on foot or seated (at the feet or the ship, as `Me::spot` does today) | New pin "Pin 1", "Pin 2", ... (next free number). Banner line "Pin 1 placed". |
| Drop a pin elsewhere | Map, ground level: right-click the ground | Same. Right-click is the only mouse action on the map besides left click. No menu. |
| Select a pin | Left click the pin or its list row | Card: title, planet, distance. Buttons: Go (primary), "Rename" and "Remove" (text, 14 px, secondary row, keys R and Delete). |
| Rename | Card "Rename" or R | Inline text box, Enter saves, Esc keeps the old name. Same code as `favourite::edit` (`favourite.rs:33`). |
| Remove | Card "Remove", Delete key with the pin selected, or B again within 10 m of a pin | Pin gone. No overwrite: B never removes a far pin. |
| Follow | Go on a pin on the same planet | Pin becomes the active pin: it shows on the compass at any distance, with its name. Sentence: "Follow it on your compass: 1.2 km." Others show only within 2 km, dimmed. |
| Go to a pin on another planet | Go | Course to the body, pin becomes active; on landing the compass leads on. Sentence: "Flies to Mars, then follow the pin." |
| Arrive | Within 12 m (the compass `HERE`) of the active pin | Pin stays; active flag clears; banner "At Pin 1". It is not deleted (owner keeps places). |

Cap: 8 pins (the list can show 8 in the 320 px panel; the 5-row limit that hides data goes away). At the cap B says "Pins full: remove one". Empty state in the pins list: "Press B or right-click the ground to drop a pin."

Where pins show (same icon everywhere, owner colour for friends, white for mine):
- Compass: active pin always, with name and distance; others within 2 km. Course target also shows as a marker (new `MarkerKind::Course`, `Icon::Send`), so a set course gives a heading on foot.
- Minimap: active pin on the rim when out of range, others as dots in range (`minimap/dots.rs:30-36`).
- Map: pins drawn as today (`map/pins_draw.rs`), the active one 12 px, others 8 px; label on hover or selection.
- World: a light column at the active pin, visible from far (spatial marker, a real object per the owner's rule). Out of scope for the UI work packages, note for the world owner.

Delete: `Waypoints`, `waypoint.rs` single-slot logic, `Under`, `Choice`, `ContextMenu`, `menu/*` (about 700 lines), `side_act::Desk::favourite` routing, `Do::Course`, `Do::Go` buttons, `WaypointChanged` stays only for the network (`link.rs`): send the active pin.

### 5.4 Keys and mouse, one table (map and world)

| Act | Key | Mouse | Shown where |
| --- | --- | --- | --- |
| Open or close map | M (Tab also, seated) | none | Prompt line first time; footer |
| Select | Up/Down in list | Left click | Footer "Click select" |
| Zoom into a place | Enter on a planet/star row with Shift? No: double click only | Double click | Footer |
| Go / Stop | G (Enter in the map) | Click the Go button | On the button |
| Home | H | "Home" button | On the button |
| Pin drop | B | Right-click ground | Footer, prompt |
| Search | / | Click the box | In the box |
| Up one level | Backspace | Click a breadcrumb | Breadcrumb |
| Close | Esc | none | Footer |

Removed: right-click menu, Left/Right tabs, Delete as clear-course (Delete now removes a selected pin), F2 sort (move to a "sort" word in the list head only if the owner asks; otherwise delete `SORT_KEY`), Back button, Copy coordinates, Centre here, slow second click to deselect (`tap` in `glance/mod.rs:58` and `pointer/click.rs:48`: click empty ground deselects; that is the only deselect).

Footer legend in the map (14 px, TEXT, one line, bottom of the side panel; replaces the empty `foot()` Action line, `computer/view.rs:118`): `Click select   G go   B pin   H home   / search   Esc close`. It is the recognition-over-recall line. It must fit `LINE_CHARS` (`layout.rs:14`, 33 chars at 14 px, so it wraps on two lines; compute with `LINE_CHARS`).

### 5.5 HUD by situation

Grid: one edge margin, `theme::EDGE` (32 px), for every corner and for the prompt line bottom; no percent offsets. Corners: TL place and coordinates, TC compass, TR minimap, BL instruments and frame, BC prompt line, BR empty (network notices). The banner sits at 72 px from the top, centered, under the compass.

| Situation | Shows | Hides |
| --- | --- | --- |
| On foot | Compass, minimap, place and weather (TL), prompt line | Instruments, frame lines unless damaged |
| Gliding | Same, plus two rows at BL: ALT, SPD (reuse `Gauge::Altitude`, `Gauge::Speed`) | Prompt line except warnings |
| In the ship, near ground | Compass, minimap, instruments: ALT SPD THR ASSIST GEAR, prompt line | APO, PERI (orbit only), WARP, DEST, FRAME |
| In the ship, in orbit | As above with APO and PERI instead of GEAR | Compass pins other than the active pin and the course |
| In warp | Compass with the course marker only, top-center line "Warp to Mars - about 3 min" (the prompt line), SPD | Minimap, ALT, APO, PERI, GEAR |
| Map open | Map screen only | All HUD (already: HUD root hidden when not `in_control`) |

Frame integrity: a first row of the instrument block, "Frame 84%", shown only when below 100 %, same style as the other rows (delete the separate corner `hud/frame.rs` placement; keep `frame::lines`). Instruments get `Gauge::Frame` removed (planet jargon), `Warp` and `Destination` removed (moved to the prompt line).

Prompt line, bottom center at `EDGE`, 14 px TEXT over STRIP, `[key] action`, exactly one row, highest priority wins:
1. Warning or block sentence (no key): the table in 5.2.
2. Course running: `[G] Stop` with "Warp to Mars - about 3 min" or the phase sentence.
3. Course set and ready: `[G] Go to Mars - about 4 min`.
4. Board and Leave (`prompt.rs:29`): `[F] Board`, `[F] Leave the ship`.
5. First-time hint (5.7).
6. Nothing.
This replaces the two rows (course row and board row, `prompt.rs:50`, `prompt/course.rs:25`) by one.

### 5.6 Text, size, spacing (SUPERSEDED by section 6 where they differ: colour, sizes, spacing, STRIP alpha)

- Sizes in play: 14 (all HUD text, list, hints), 18 (card title, primary button, key line of the prompt when it is a warning), 28 (banner, menu headings). 64 only on the title screen. No other size. Icons 24 px, compass glyph 14 px.
- Weight never bold. Hierarchy: size, TEXT vs MUTED, inversion (the one white block per screen).
- STRIP alpha 0.5 to 0.6 (`ui/theme.rs` STRIP, `PALETTE` test). HUD text over the live game uses TEXT, never MUTED or FAINT; MUTED stays in menus and map side panel, where the backdrop darkens (`TINT`). Every HUD text gets a STRIP backing (compass labels, coords, climate, frame lines). The compass rule uses TEXT at 50 % instead of HAIRLINE.
- Spacing: only `GAP_XS` 4, `GAP_S` 8, `GAP_M` 16, `GAP_L` 24, `GAP_XL` 40. Padding inside a strip: 8 horizontal, 4 vertical (as today). Rows 28 and 32.
- No borders, no rounded corners: remove `orbits.rs:51-52` rings by drawing them as thin rectangles or ask the owner for an exemption for the map picture; replace `table.rs:20` border by a 1 px node of HAIRLINE height.
- Disabled looks: FAINT text and no inversion. Enabled primary: see section 6 (accent fill). The 'never colour on chrome' line no longer applies.

### 5.7 First-time hints (no tutorial wall)

One persisted bitset `hints_seen` in `Persistent<Settings>` (new field `hints: u8`, serde default 0). The prompt line (5.5, row 5) shows at most one hint, only when nothing with higher priority shows, only while the player is in control, hides on the first matching action, and a shown hint stays at most 10 s per session. Hints never repeat once their action is done.

| Hint | Shown when | Disappears when |
| --- | --- | --- |
| `[F] Board` (already exists) | Within hatch reach on foot | Player boards |
| `[M] Open the map` | First time seated, after 5 s idle, map never opened | M or Tab pressed once |
| Map footer legend (5.4) | Always in the map | Never |
| Pins list empty text | Map open with 0 pins | First pin |
| `[B] Drop a pin` | Standing 60 s on the ground with no pin, map opened once | B pressed once |
| `[G] Go` pulsing is NOT used; the button is already the white block | n/a | n/a |

No modal tutorials, no arrows, no popups.

### 5.8 Files and functions to change (file:line at HEAD)

Go flow and words (computer, ship):
- `computer/route.rs:57-84`: add `Block::OnFoot`; rewrite `Block::words` per 5.2; delete `Block::Surface`; delete `RouteStatus::label` usages.
- `ship/route/block.rs:18-34`: `blocker` takes the travel mode; return `OnFoot` first; delete `Surface`; `status` (`block.rs:57`) unchanged shape.
- `ship/route.rs:136` `fly` and `ship/route/ascent.rs:52` `climbs`, `ascent.rs:climb`: make a surface-frame ship that is not landed climb for any target; at `Some(true)` fall through to the warp phases instead of `Autopilot::ARRIVED` when the target is not the body being left (`ascent.rs:~158`).
- `ship/route.rs:178` `report`: read `Location`; compute `OnFoot`.
- `ship/pilot.rs:84` add `Autopilot::stop()`; new `Cancel` event in `computer/route.rs` next to `Engage` (`route.rs:44`); `ship/route.rs:on_engage` (~line 160) adds `on_cancel`.
- `computer/course.rs:24-76`: replace `NO_COURSE`, `action`, `progress` with `card_line(target, course, status, seated, keys) -> (enabled, sentence)` and `hud_line(...)`. Delete `Act::Set`, `Act::Clear`.
- `computer/keys.rs:145-181`: `run` handles `Go` (set plus engage) and `Stop`; delete `Delete` and arrow-tab code; Home becomes H.
- `map/action.rs:12`: `engage` becomes `go` (same system, same key, runs when the map is closed too).
- `map/deeds.rs:39-120`: replace the four-button row by the card: Go button plus sentence plus pin buttons. Delete `offer`, `Deed::Choice`, `Reason`.
- `computer/detail.rs:advice` ("Visit this system to chart its planets"): move to a muted note under the facts, not the Go sentence; Go still works on uncharted places.
- `computer/home.rs:23` `toward`: keep; used by H.

Pins (map):
- Delete `map/menu/**`, `map/menu_tests.rs`, `map/waypoint.rs` single slot (keep `WaypointChanged` plumbing for `link.rs`), `map/side_act.rs` `Desk::favourite` routes.
- `map/favourite.rs`: `Favourites` becomes `Pins` data (name `Pins` collides with `map/pins.rs:Pins`; call it `Places`); add `active: Option<usize>`; `save` (line 18) numbers the next free "Pin n"; cap 8.
- `map/side.rs:9-84` and `map/side_show.rs`: rows show a pin icon, name, distance; text buttons replace Send, Pencil, Trash; `FAVOURITES` = 8 (`computer/chrome.rs:21`, `layout.rs:fixed`).
- `map/glance/mod.rs:58` `tap` and `computer/pointer/click.rs:48`: drop the slow second click; add right click = drop pin on ground (new system in `map/waypoint.rs:drop_key` area).
- `map/waypoint.rs:88` `drop_key`: B drops or removes within 10 m; never removes far pins.
- `map/nav/bar.rs`: remove `BackButton`.
- `map/pins_draw.rs:34-39`: active pin larger.
- `map/side_show.rs:save_words` and `SAVE` (`side.rs:35`): delete; "Save place" is B or right click.

HUD:
- `hud/prompt.rs:45-96` and `hud/prompt/course.rs`: merge into one `prompt::line` with the priority list of 5.5; remove `bottom: percent(22)`.
- `hud/flight.rs:21-30`: remove `Frame`, `Warp`, `Destination` gauges; `hud/panel.rs:81-82` anchor at `EDGE`; add frame row; `hud/frame.rs:14-24` becomes a row inside the instrument block (or hides the separate corner when seated).
- `hud/compass/aim.rs:70-95` and `markers.rs:7`: add `MarkerKind::Course` filled by a new `beacons::course` from `Destination` and `target_position` (works for `Target::Star` and `Galaxy` too, `compass/beacons.rs`); `compass/show.rs:30` icon; `map/pins_draw.rs:34`, `minimap/dots.rs:32` one arm each.
- `hud/compass/mod.rs:100-120`: rule colour; label backing STRIP (`show.rs:label`).
- `hud/coords.rs:36`, `hud/climate.rs`, `hud/banner.rs:50`: backing strip and TEXT colour; banner top 72 px.
- `ui/theme.rs` STRIP alpha 0.6; `ui/theme_tests.rs` add the contrast assertion (5.9).
- `settings/model.rs:130`: `hints: u8`; new `hud/hints.rs`.

### 5.9 Work packages (three implementers, disjoint files)

Contract between packages (write first, 20 lines, in `docs/handoff/ux-contract.md`): `computer::card_line(...) -> (bool, String)`, `Engage`, `Cancel`, `Block::OnFoot`, `MarkerKind::Course`, `Places` (pins) API: `Places::{drop, remove, rename, active, set_active}`. Merge order: A, then B and C in either order.

**WP-A: Go flow and route words** (owns `src/computer/{route,course,keys,home,detail,hold}.rs`, `src/ship/route.rs`, `src/ship/route/**`, `src/ship/pilot.rs`, `src/ship/helm.rs` only the cancel line, `src/map/{deeds,action}.rs` and their tests, `src/computer/tests_*`)
- Do: 5.1 steps 3 and 6, 5.2, `Cancel`, auto-climb for any target.
- Test asserts: (1) `Block::words` returns exactly the 5.2 strings for `OnFoot`, `Landed`, `Inside{star:false/true}`, using the key names passed in; (2) a ship on a planet surface, not landed, with a course to another body, reaches `Phase::Aligning` after the climb (`ship/route/ascent_tests.rs`); (3) a player on foot with a course gets `RouteStatus::Blocked(OnFoot)`; (4) G pressed with a course and status Idle triggers one `Engage`; pressed while running triggers `Cancel` and the autopilot target becomes `None`.
- Screenshot (probe example): map card for a moon: white "Go [G]" block, trip line, sentence; same card on foot with the sentence "Board your ship first: walk to the hatch and press F."

**WP-B: Pins and map interaction** (owns `src/map/**` except `deeds.rs`, `action.rs`; `src/computer/{view,layout,chrome,section,pointer/**,tab,typing}.rs`, `src/saves` pin model, `src/net` marker link if needed)
- Do: 5.3, 5.4, footer legend, breadcrumb-only level switch, delete the context menu, cap 8.
- Test asserts: (1) B twice near a pin removes it, B twice far away makes two pins, 9th pin refused; (2) pin names number from the next free number; (3) rename Esc keeps the old name; (4) panel `listed` returns 8 rows when 8 pins and a 540 px window (`layout::listed`); (5) no `ContextMenu` symbol remains (`rg ContextMenu src` empty); (6) a click on empty ground deselects; right-click on ground drops a pin; (7) saves load an old save with a waypoint and favourites into pins (migrate: favourites to pins, waypoint to a pin).
- Screenshot: map open on the ground with 3 pins, one selected: card with "Go", "Rename", "Remove"; footer legend line; breadcrumb with no Back button; empty pins text when none.

**WP-C: HUD, prompt line, theme, hints** (owns `src/menus/hud/**`, `src/ui/{theme,chrome,item}.rs`, `src/settings/model.rs` hints field, `src/menus/options/{binds,reference}.rs` for key text)
- Do: 5.5, 5.6, 5.7; the one-row prompt; compass course marker; backing strips; edge margin.
- Test asserts: (1) a pure `prompt::choose(state) -> Option<(Option<KeyCode>, String)>` returns the priority order of 5.5 for table-driven cases (block beats course beats board beats hint); (2) `MarkerKind::Course` shows on the compass for `Target::Star`; (3) instrument rows for the ship in orbit are exactly ALT SPD THR ASSIST APO PERI; (4) contrast test: TEXT over STRIP over a white backdrop is at least 4.5 (`ui::contrast`); (5) a hint is shown once and `hints` bit set persists in a settings round trip; (6) frame row and instrument block do not overlap (node rects from a headless layout, as `hud/tests_frame.rs` does).
- Screenshot: ship in orbit with a course: BL instruments (7 rows max) and frame row under them, BC one prompt line `[G] Stop  Warp to Mars - about 3 min`; on foot with a set course: compass shows the Course marker; over a white snow field, compass labels readable.

Note for the lead: Block removal (`Surface`) and the climb change touch ship physics tests (`ship/route/ascent_tests.rs`, `block_tests.rs`); rewrite them to the new behaviour and say so in the report (AGENTS.md). WP-A risk: chaining across stars for Home (5.1 step 6) depends on whether `goal::resolve` for a far `Body` works; if not, chain legs on `Phase::Arrived` by a small `Destination` queue in `ship/route/done.rs`.
Not checked (read-only, no build): that H is a free key, that a far `Target::Body` flight works, the glide HUD (no gauges exist for it today).

## 6. Owner direction of 2026-11-20 (overrides AGENTS.md "black and white chrome, pixelarticons only")

Still true: square corners everywhere, Undefined font only, never bold, no borders (separate with space, tone and translucency). New: translucent panels, a restrained accent system, real icons, empty-state text, no guessing, readable stats, generous margins. This section replaces 5.6 and the "no colour" statements of 3. AGENTS.md must be edited by the lead to match (design rules, `src/ui/icon.rs:1-7` doc).

### 6.1 What Bevy 0.20 UI can really do (checked in bevy_ui-0.20.1 and `src/ui/backdrop.rs`)

- Backdrop blur: NOT in Bevy UI (`backdrop.rs:1-2` says so). The repo has its own `BlurMaterial` (UiMaterial) that samples the low-res world image (`LowRes`) with a 5 % screen-height radius, plus darkening `TINT`. Marker component `Blurred` on any node. It is a wide box blur over a small texture, cheap, so every large panel (map side panel, card) may use it. Small HUD strips (prompt line, compass labels, instrument block) must NOT: use plain translucent fill, to avoid a material per element.
- Gradients: `BackgroundGradient` exists (`bevy_ui-0.20.1/src/gradients.rs:544`). Use it for the soft fade at a panel edge (opaque to transparent over 24 px) instead of any border or hairline.
- Shadows: `BoxShadow` (`ui_node.rs:3311`) and `TextShadow` (`widget/text.rs:163`) exist. Use a `TextShadow` (offset 1 px, black at 0.6) on any text drawn without a panel (compass labels, coordinates). No box shadows (they read as borders).
- Corners: `BorderRadius` exists and must stay at zero. A test asserts no `BorderRadius` other than ZERO in `src/ui`, `src/map`, `src/computer`, `src/menus` (violation today: `computer/orbits.rs:52`).
- Motion: no animation crate is needed. A small system lerps alpha and `UiTransform` translation (`ui_transform.rs` exists) with `Time`. Rules: fade 120 ms ease-out for hints, banners and tooltips; panel enter = fade 160 ms plus 8 px slide toward its edge; map level change = 200 ms cross-fade of the picture (today a black veil, `map/fade.rs:49`, 300 ms: make it a PANEL-tinted veil, not black); no bounce, no overshoot, no looping pulse except the warp spool bar. All durations in one `ui/motion.rs` of constants (seconds).

### 6.2 Tokens for `src/ui/theme.rs`

Rendering: the UI camera draws at WINDOW resolution above the low-res world (`ui/camera.rs:22-31` builds a `Camera2d` with `order: 1`; the world is a `LowRes` image of 360 rows, `render/tune.rs:8` `REFERENCE_ROWS`). So text is sharp at any window size; but nothing scales it: there is no `UiScale` anywhere (grep empty), so 14 px text is 2.6 % of a 540 px window and only 1.3 % of a 1080 px one. Decision: set `UiScale = (window height / 720).clamp(0.75, 3.0)` in `ui/camera.rs::frame_view` (the existing `window_changed` system), and define every token below at the 1280 by 720 reference. The smallest supported window, 960 by 540, becomes scale 0.75 and lays out exactly like 1280 by 720, so layouts are designed once. 1080p gets 1.5, 4K gets 3.

Type scale (logical px at the 720 reference; Undefined Medium only, never bold):

| Token | Size | Line height | Use |
| --- | --- | --- | --- |
| `FONT_SMALL` | 16 | 24 (1.5) | All HUD text, list rows, facts, hints, key caps. Minimum text size anywhere. |
| `FONT_BODY` | 20 | 28 | Card title, primary button, prompt warnings, menu items. |
| `FONT_HEADING` | 32 | 40 | Banner, section headings of full-screen menus. |
| `FONT_TITLE` | 72 | 80 | Title screen only. |
| `ICON` | 24 | 24 | Every icon; `ICON_SMALL` 16 only inline in a 16 px text line; never scaled by a fraction. |

At most three sizes on one screen (today: main menu uses TITLE, BODY, SMALL = 3, worlds list uses BODY, HEADING, SMALL = 3; fine, keep; the map and HUD use 14, 18, 28 = 3). Section labels in capitals (`ui/widgets.rs:caps`, letter spacing 3 px) use SMALL, not a fourth size. A test counts distinct `TextFont` sizes per spawned screen and asserts at most three.

Colour tokens (sRGB, alpha last). Colour is allowed on chrome now, but only these:

| Token | Value | Means |
| --- | --- | --- |
| `TEXT` | 1.00 1.00 1.00 1.00 | Primary text. |
| `TEXT_2` | 1.00 1.00 1.00 0.72 | Secondary text and labels. Replaces `MUTED` 0.65, which fails 4.5:1 over a bright sky even on a panel. |
| `TEXT_OFF` | 1.00 1.00 1.00 0.45 | Disabled only. Never carries information alone. |
| `INK` | 0.02 0.03 0.04 1.00 | Text on an accent fill. |
| `PANEL` | 0.04 0.05 0.07 0.72 | Fill of every HUD panel and strip over the live world. |
| `PANEL_BLUR` | 0.04 0.05 0.07 0.55 | Fill of blurred big panels (map side panel, menus), on top of `Blurred`, whose `TINT` 0.82 does the rest. |
| `HOVER` | 1.00 1.00 1.00 0.10 | Row under the pointer or keyboard focus. |
| `SELECT` | accent at 0.20 | Chosen row. A 4 px full-height bar of `ACCENT` at its left edge marks it (not a border: it is a filled node inside the row). |
| `ACCENT` | 0.40 0.80 1.00 1.00 | The primary action ("Go"), selection, the course line and its compass marker. One accent only. |
| `PIN` | 1.00 0.86 0.35 1.00 | Waypoints and favourites (one kind, section 5.3), pin icons in lists. |
| `WARN` | 1.00 0.60 0.25 1.00 | A blocked course, weather and heat warnings, "Frame" under 50 %. |
| `DANGER` | 1.00 0.40 0.38 1.00 | Frame under 25 %, a part broken. |
| friend | `map/palette.rs:player_color` | One colour per player, as today. Palette must avoid the hues of ACCENT, PIN, WARN (test it: no friend colour within 30 degrees of those). |
| home, ship | `TEXT` | Distinguished by icon shape, not colour. |

Contrast, computed (sRGB relative luminance, black-tinted panel over a pure white sky, the worst case, and over black):
- Over a white sky: `PANEL` (0.04 0.05 0.07 at 0.72) gives a backdrop of about 0.31 sRGB, luminance 0.078. `TEXT` is 8.5:1, `TEXT_2` is 4.7:1, `ACCENT` text is 8:1. All at least 4.5:1.
- The old `STRIP` (black at 0.50) gives 4.0:1 for `TEXT` over white: fails. Raise to `PANEL`.
- Over black sky: everything above 15:1.
- `PANEL_BLUR` over blur: the `TINT` 0.82 rule of `theme.rs` already holds 4.5:1; keep `theme_tests` and add the same assertion for `PANEL` over white using `ui::contrast`.
- Text without a panel (compass labels, coordinates) gets `TextShadow` and a `PANEL` backing strip; MUTED/`TEXT_2` is not allowed on a bare world.
- `WARN` and `DANGER` text sit on a `PANEL` too: 7:1 and 6:1 on that panel over white (assert in the same test).

### 6.3 Margins, grid, legibility ("generous, legible, clean, modern")

Spacing scale (px at the 720 reference; multiples of 4, base 8). Replace `GAP_XS..GAP_XL` (4, 8, 16, 24, 40, `theme.rs:28-39`) by:

| Token | px | Rule: where it goes |
| --- | --- | --- |
| `S1` | 4 | Between an icon and its text, between lines of one fact. |
| `S2` | 8 | Between small siblings: key cap and action, chips. |
| `S3` | 16 | Padding inside a panel strip (horizontal) and between rows of a list (row height 32 already includes 8 of air). |
| `S4` | 24 | Panel padding on all sides, gap between groups inside a panel. |
| `S5` | 32 | Screen edge margin, every corner, every edge. Gap between two panels. |
| `S6` | 48 | Between sections of a full-screen menu; top margin under a title. |

Rules: screen edge = `S5` (replaces `EDGE` 32, `GAP_M` 16 and the 22 % and 14 % offsets). Panel padding = `S4` all around (a one-line strip may use `S3` horizontal and `S2` vertical, never less). Gap between groups = `S4`. Gap between rows = 0 with row height 32 (hit target at least 32 by 32). Row text is vertically centered. Text never touches a panel edge or the window edge: minimum 16 px between a glyph and any edge of its panel or the screen.

One grid: a 8 px grid; every `left`, `top`, `width`, `height` and gap on a screen is a multiple of 4, and positions of panels a multiple of 8. A headless layout test collects the `ComputedNode` rects of a spawned HUD and map and asserts positions are multiples of 4 (after dividing by `UiScale`) and that no two sibling panels overlap or sit closer than `S5`.

Body text: at most 45 characters per line in cards and 60 in hints (a card is 320 px wide at the reference; at 16 px glyph 0.6 aspect = 9.6 px, so 320 minus 48 padding is 28 characters: widen the side panel to 384 so a line holds 35 characters, and wrap sentences at 2 lines max). Facts are label left, value right, one line. Line height 1.5 for sentences, 1.25 for single-line rows.

Limits per screen (things shown at once, as count of distinct panels and rows): HUD on foot at most 4 elements (compass, minimap, place line, prompt); in the ship at most 6 (add instruments and frame); map screen at most 3 panels (map, side panel with card, footer legend) and a list of at most 8 rows; card at most 4 groups of 2 facts (8 facts) with the rest behind a "More" row (see 6.7); at most 1 primary button per screen. A prompt line shows one thing.

Current violations (file:line at HEAD):
- Cramped strips: prompt and course rows pad 8 by 4 (`hud/prompt.rs:58`, `hud/prompt/course.rs:37`); instrument block pads 8 all around (`hud/panel.rs:84`); minimap caption pads 4 by 0, text touching the strip top and bottom (`hud/minimap/widget.rs:87`); fps counter at 8 px from the edge (`ui/fps.rs:40`); favourites rows pad 4 (`map/side.rs:104`, `computer/section.rs:45`); deed buttons gap 4 (`map/deeds.rs:86-88`).
- Uneven edge margins: 16 px (`hud/coords.rs:27-28`, `hud/frame.rs:19-20`, `hud/compass/mod.rs:116`, `hud/minimap/widget.rs:114-115`, `computer/view.rs:73` 16), 32 px (`hud/panel.rs:81-82`, `menus/pause_hud.rs:38-39`), 22 % and 14 % (`hud/prompt.rs:45`, `hud/banner.rs:50`).
- Uneven gaps: map panel uses a 24 px gap between sections (`computer/layout.rs:SECTION_GAP`) and 4 to 8 px inside, while the HUD instrument rows have 8 px column gaps with no row gap (`hud/panel.rs:37`).
- Text sizes: 4 sizes exist (14, 18, 28, 64) and the icon glyph at 14 (`hud/compass/mod.rs:40` `MARK`) is a fifth visual size next to 24 px icons elsewhere. Map card shows title 18, rows 14, headings caps 14, icons 24: 3, fine. The title screen shows 64, 18, 14.
- Small text: 14 px everywhere and no `UiScale`: 66 uses of `FONT_SMALL`; unreadable on 1440p and 4K.
- Line length: `computer/layout.rs:LINE_CHARS` is 33 characters at 14 px in a 320 panel with 16 px margins (text only 6 px from nothing but the 16 px margin; wrap is cut off by `clip`, `readout.rs:fact`), so long values are clipped with "..." (`format::clip`), a legibility loss. Reflow to two lines instead.

### 6.4 Icons: a real set, no black boxes

Today: one font only, `assets/fonts/pixelart-icons-font.ttf` (pixelarticons 2.4.2, MIT, `LICENSES/pixelarticons-MIT.txt`), loaded by `ui/icon.rs:13-20` (`include_bytes!`, handle `ICON_FONT`), 35 glyphs in `CODEPOINTS` (`icon.rs:58`). It has no ship, planet-stat or galaxy icons; `Icon::Send` stands for "ship" and "course" and `Icon::Earth` for planet; the doc says an action with no fitting icon gets no icon (`icon.rs:5-7`): that rule is dropped.

Proposal (no new crate; same font mechanism as today):
1. Generic and stat icons: Phosphor Icons, Regular weight, MIT licence (copyright Phosphor Icons), https://github.com/phosphor-icons/web (webfont with Private Use Area codepoints, six weights) and SVG sources https://github.com/phosphor-icons/core . Opened: the `web` README (licence MIT, font weights) and, from `core`, `raw/regular/planet.svg`, `raw/regular/rocket-launch.svg` and the MIT LICENSE, so `planet` and `rocket-launch` exist. Not verified one by one: moon, star, house, map-pin, user, thermometer, wind, drop, timer, gauge, paw-print, leaf, spiral, ruler, scales; the implementer checks each name in the package CSS and, where a name is missing, draws it (next point). Caveat for the owner: Phosphor strokes have round caps and joins (visible in the SVG: `stroke-linecap="round"`). If that clashes with "no rounded corners", pick Material Symbols Sharp (Apache-2.0, https://github.com/google/material-design-icons; not opened, licence not verified here) or keep pixelarticons for UI chrome and use the new set only for the map and stats.
2. Ship icons (ship parked, ship flying, ship in warp, ship damaged) and any missing glyph: our own art, white mask PNGs, 48 by 48 at 2x, in `assets/icons/`, drawn with `ImageNode` and `ImageNode.color` for the tone (same tinting path as font text). Reason: no pack matches the game's ship. Drawn in code is not worth it: Bevy UI has no vector path node.
3. Loading: add `ICON_FONT_2` (second `uuid_handle!`, `include_bytes!` of the Phosphor TTF, subset to the used glyphs to stay under 100 KB with `pyftsubset`, a one-off tool step recorded in `packaging/`), extend the `Icon` enum and `CODEPOINTS` (array length constant `35` at `icon.rs:58` becomes the new count), keep `icon_tests.rs` checking every codepoint exists in its font. Add `LICENSES/phosphor-MIT.txt`.

Icon map (name is what hover/selection shows; every icon has a text name somewhere on screen, see 6.6):

| Meaning | Icon (Phosphor regular, or own) | Used at |
| --- | --- | --- |
| Ship, parked | own `ship_parked` | map ship pin, prompt `[F] Board`, `markers.rs` Ship |
| Ship, flying | own `ship_flying` | map ship pin in flight, instrument block header |
| Ship, in warp | own `ship_warp` | warp prompt, compass course marker while warping |
| Ship, damaged | own `ship_damaged` | frame row |
| Planet | `planet` | list rows, card, compass remote bodies |
| Moon | `moon` | list rows |
| Star | `star` is taken by favourite: use `sun` | list rows |
| Galaxy | `spiral` | list rows |
| Home | `house` | home pin, Home button |
| Pin (waypoint and favourite) | `map-pin` | pins everywhere, PIN colour |
| Friend | `user` in the friend's colour | compass, map, list |
| Herd | `paw-print` | compass, minimap |
| Go | `rocket-launch` | Go button |
| Stop | `x-circle` | Stop button |
| Radius | `ruler` | stats |
| Gravity | `scales` | stats |
| Temperature | `thermometer` | stats |
| Air pressure | `wind` | stats |
| Day length | `sun-horizon` | stats |
| Year length | `timer` | stats |
| Water | `drop` | stats |
| Life | `leaf` | stats |

Black boxes behind icons (remove all; icons sit on the panel or the world, and the panel supplies the contrast):
- `hud/minimap/widget.rs:77-78`: `mark()` draws every minimap rim icon in a 14 px `BackgroundColor(theme::INK)` box. Remove the fill; give the glyph a `TextShadow` and size 16.
- `hud/minimap/widget.rs:92` caption strip `STRIP` behind text (keep as `PANEL`, it is text, not an icon).
- `map/pins_draw.rs:137` pin dot `BackgroundColor(dress.fill)`: an 8 px filled square behind each marker. Replace the square by the icon glyph in the marker colour (no fill node); selected pin 24 px, others 16 px.
- `menus/hud/prompt.rs:61` and `prompt/course.rs:39` `STRIP` behind the key cap and action: keep a panel behind text, but the icon is inside the panel, not a separate box.
- `map/fade.rs:49` black veil over the map: becomes a `PANEL`-tinted veil.
- `computer/view.rs:87` and `:207`: side panel `STRIP` and root `SCRIM`: root keeps the blur, the side panel changes to `PANEL_BLUR`; neither is behind an icon.
- `ui/item.rs:step_button` icon buttons draw a 24 by 24 hit box with `Look` background on hover (inversion to white). Replace the inversion by `HOVER` tint; no white block behind an icon.

### 6.5 Remove the monument dots

They are the structure stamps on the MINIMAP ground image: `menus/hud/minimap/paint.rs:92-96` (`render` loops `sites(...)` and calls `stamp`), `paint.rs:43-58` (`stamp`), `paint.rs:60-87` (`on_face`, `sites`, `SITES_RANGE` at `:26`), the import `paint.rs:17`, and the test `menus/hud/minimap/tests/sites.rs`. The big map has none (no structure code under `src/map`, checked by grep). Delete `stamp`, `on_face`, `sites`, `SITES_RANGE`, the import, the `render` branch and `tests/sites.rs`; `render` becomes `paint(globe, patch)`. (`structure_in_cell` itself stays: the world uses it.) WP-C owns this.

### 6.6 No guessing: every place the player must guess

| Where (file:line) | The guess | Fix |
| --- | --- | --- |
| `map/side.rs:80-84` favourite row icons Send, Pencil, Trash | Three icon-only buttons | Text labels "Go", "Rename", "Remove" on the selected row only (the card, 5.3); rows show only name and distance. |
| `map/deeds.rs:67-76` "Engage", "Clear" buttons | No key shown | Primary button "Go [G]" / "Stop [G]" (5.1). |
| `computer/keys.rs:145-181` Enter, Delete, Home, F2, `/` | Hidden keys | Footer legend (5.4); delete Delete-as-clear and F2. |
| `computer/keys.rs:63-68` Left/Right tabs | Tabs without buttons | Breadcrumb is the only switch (5.1 step 5). |
| `map/glance/mod.rs:58`, `computer/pointer/click.rs:48` | Single, slow second, double click | One rule: click selects, double click enters, click empty ground deselects. Footer says it. |
| `map/menu/mod.rs:95` context menu | Nine choices vary by target | Delete (5.3). |
| `hud/minimap/mod.rs:cycle` N key | Minimap range cycle with no cue | Minimap caption shows its range ("250 m") and the key on first use (hint table, 6.8). |
| `hud/flight.rs:21-30` ASSIST, APO, PERI, THR | Jargon | Rename "Assist" to "Stabiliser", APO/PERI to "Highest point" and "Lowest point" (only in orbit); THR to "Throttle"; add the key beside Assist and Throttle once (hint). |
| `computer/route.rs:84` labels "blocked", "dropping out", "spooling" | State not explained | Sentences of 5.2. |
| `hud/compass/show.rs:label` marker with empty label shows only a distance | Which marker is it? | Unnamed markers show their kind name: "Pin", "Home", "Ship", "Herd" (`pins_draw.rs:shape` already has the names; reuse). |
| `map/pins_draw.rs:76-85` labels | Names appear only when `named` has room | Selected and hovered pin always shows its name and kind. |
| `computer/detail.rs:advice` "Visit this system to chart its planets" | Unclear how | "Not visited. Fly there to learn its air, water and climate." |
| `hud/banner.rs:watch` planet name only | Where am I | Banner second line: "Mars, Sol system" kept; add the key to the map the first time (hint). |
| `settings` Home key (`computer/keys.rs:HOME_KEY`) | Missing on laptops | H key and a button. |
| `map/waypoint.rs:88` B places at the feet while the doc says "where the player looks" (`settings/model.rs:179`) | Wrong doc | Fix the doc: "at your feet". |
| Players list hidden until a friend joins (`map/side_show.rs:61`) | Why is there no list | Empty text in a session (6.7). |
| Planet card facts: "Light 1.2 x home", "Tilt 23 deg", "Zone" (`computer/facts.rs:30-37`) | What does it mean to me | Stat redesign, 6.9. |
| Icon with no name on hover | Any icon-only control | Every icon button shows its text name on hover/focus after 400 ms as a `PANEL` tooltip below it (`ui/` new `tooltip.rs`, 120 ms fade). |

### 6.7 Empty and loading states (exact words)

Each is a 16 px `TEXT_2` sentence, centered in the empty area, over `PANEL`, optionally with a 24 px icon above it. No black rectangle without words.

| Where (file:line) | When | Text |
| --- | --- | --- |
| `map/planet.rs:13,41` (`BLANK` tiles) | Picture not ready | "Drawing the map..." |
| `map/planet.rs` plus `map/raster`/`explored` | Planet level, nothing explored (explored share 0) | "Nothing explored here yet. Walk or fly over the ground to chart it." |
| same | Planet level, explored but the pointer is over black ground | Tooltip: "Not explored yet." |
| `computer/sift.rs:108` | Empty list (nothing charted) | "Nothing charted here yet. Visit a place to chart it." |
| `computer/sift.rs:109` | Search finds nothing | "No place is called \"{filter}\". Press Esc to clear the search." |
| `computer/readout.rs:68,136` (`Card::default`) | No selection | "Pick a place on the map or in the list to see it here." |
| `map/side_show.rs:write` | Zero pins | "No pins yet. Press B or right-click the ground to drop one." |
| `map/side_show.rs:61`, `map/side.rs:players` | In a session, alone | "No one else is here yet." (hidden in single player) |
| `menus/hud/minimap/space.rs:40,142` (`QUIET` "no chart") | In space, no chart | "Not charted yet." |
| `menus/hud/minimap/widget.rs` (patch hidden until painted) | Ground patch loading | "Mapping the ground..." |
| `computer/facts.rs:63-69` uncharted planet card | Not visited | Note under the size facts: "Not visited. Fly there to learn its air, water and climate." |
| `computer/map.rs` system chart | Selected star not charted, no planets known | Over the chart: "This system has not been charted. Fly there to see its planets." |
| Galaxy and Universe charts | No visited places | "Only places you have visited show here." |
| `menus/worlds/view.rs:91` | No worlds | Keep "No worlds yet" and add a line "Press N to start one." (match the real key; check `worlds/keys.rs`). |
| `menus/multiplayer/view.rs:106` | No servers | Keep; its key hint is already there. |
| `hud/compass/mod.rs` | Nothing to show | Nothing: a compass is not a map; the bar's own hairline stays. |

### 6.8 Hints and the prompt line (adds to 5.7)

The hint table of 5.7 gains: `[N] Minimap range` first time the minimap shows and N never used; `[V] View` no. Keep at most four hints total in the game. Each uses the prompt line (one row, `PANEL`, 16 px, `S3` by `S2` padding, 120 ms fade).

### 6.9 Planet card redesign

Layout (card inside the 384 px side panel, `S4` padding, groups separated by `S4`, no rules or borders):

```
[planet]  Mars                                  title 20, TEXT
          Rocky planet of Sol                   16, TEXT_2
[ Go  [G] ]   about 4 min                       primary button, accent fill
Not visited. Fly there to learn its air...      only when uncharted

SIZE AND GRAVITY                                caps 16, TEXT_2
[ruler]   Radius        3,390 km     0.5 x home
[scales]  Gravity       0.38 g       0.4 x home     Light: long jumps

AIR AND HEAT
[wind]        Air         thin, 0.6 bar          Wings give weak lift
[thermometer] Temperature  -63 C                 Cold
 
DAY AND YEAR
[sun-horizon] Day         24 h 40 min            about as long as home
[timer]       Year        687 days               1.9 x home

WATER AND LIFE
[drop]        Water       ice at the poles       (existing `words::water`)
[leaf]        Life        low odds               (existing `odds.rs` row)
```

Rules: each row is icon (24), label (`TEXT_2`), value (`TEXT`, right-aligned to the same column, unit always written: km, g, bar, C, h), and an optional third column of one to three words of meaning (`TEXT_2`). Comparison against home is always "x home" with one decimal, computed against `universe.planet(universe.home())` (so "1.2 x home gravity", never an unexplained number). Four groups of two rows; the rest of today's facts (Light, Zone, Tilt, Moons, Rings, Orbit) go behind a "More" row that expands in place (`Row::Heading` + `Row::Fact` in `computer/detail.rs`, `facts.rs`; expansion state in `Board`). Uncharted bodies show Size and gravity only (radius and gravity are known, `facts.rs:uncharted`).

"What it means for me", plain, three questions, derived from values already in `PlanetSpec` (thresholds are proposals for the owner; they only choose a word):
- Can I land: "Yes" or "No ground: gas giant" from `PlanetClass::GasGiant` (`giant.rs:surface`).
- Can I walk: gravity words: under 0.3 g "Very light: huge jumps"; 0.3 to 0.8 "Light"; 0.8 to 1.25 "Like home"; 1.25 to 2 "Heavy: short jumps"; over 2 "Crushing". (`words.rs` new `gravity_feel`.)
- Can I fly: pressure words: `words.rs:AIR` already has none, thin, like home, thick: "No lift: boots only", "Weak lift", "Wings work", "Strong lift, hard on the wings" (the last wording needs an owner check against the glider model).
- Temperature words: below -100 C "Freezing", -100 to 0 "Cold", 0 to 35 "Mild", 35 to 100 "Hot", above 100 "Scorching" (`words.rs:temperature` returns the number; add `feel`).

Files: `computer/facts.rs:planet`, `body`, `uncharted` (lines 14 to 60), `computer/giant.rs:surface`, `company`, `computer/detail.rs:Row` (add `Row::Stat { icon, label, value, note }`), `computer/readout.rs:fact` and `row` (render the four columns), `computer/words.rs` (feel words, "x home" helpers using `format::short`), `computer/odds.rs` (life row becomes a Stat with the leaf icon). Tests: for Earth-like home the rows read "1.0 x home" for gravity, radius, year; a gas giant shows "No ground: gas giant" and no Water row; every Stat value has a unit; no row exceeds the line width (clip test `tests_width.rs`).

### 6.10 Changes to the work packages (section 5.9)

- WP-A (Go flow, `computer/` route files, ship route): unchanged, plus `computer/words.rs`, `computer/facts.rs`, `computer/giant.rs`, `computer/detail.rs`, `computer/odds.rs`, `computer/readout.rs` (planet card, 6.9) and the empty/card sentences of 6.7 that live in `computer/` (`sift.rs`, `readout.rs`, `map.rs`).
- WP-B (pins and map): adds empty states of `map/` (6.7: planet map text layer in `map/planet.rs`, pins and players text in `map/side*.rs`), pin glyph drawing (`pins_draw.rs:137`), tooltips on map controls, map fade veil.
- WP-C (HUD, theme, icons): owns `ui/theme.rs` tokens, `ui/motion.rs`, `ui/icon.rs` and the icon assets, `UiScale` in `ui/camera.rs`, `ui/item.rs` hover tint, `ui/tooltip.rs`, `ui/chrome.rs`, HUD files, minimap mark boxes and monument deletion (6.4, 6.5), `AGENTS.md` design rules edit (the lead approves). WP-C merges FIRST for tokens (A and B depend on `TEXT_2`, `ACCENT`, `PANEL`, spacing names and the `Icon` variants): it lands a thin commit "theme tokens and icon enum" before the rest; A and B code against the names in 6.2 and 6.4 in the meantime.
- Extra acceptance (screenshots): map side panel with the planet card of 6.9 (four groups, icons, units, comparisons), over bright snow and over black space, text readable; ground map with no exploration showing the sentence; HUD on foot over a white sky with all text on panels; no black square behind any icon on the minimap rim.
- Extra tests: contrast (6.2), no `BorderRadius` non-zero, at most three font sizes per screen, positions multiple of 4, every `Icon` codepoint exists in its font, every empty state of 6.7 has a test that spawns the state and finds the exact sentence, `UiScale` at 540, 720, 1080, 2160 rows is 0.75, 1.0, 1.5, 3.0.

### 6.11 Not verified (read-only, no build)

The Phosphor names other than planet and rocket-launch; Material Symbols Sharp; the gliding feel words against the glider model; whether `Blurred` can be applied to a nested node inside another `Blurred` node without double sampling; the actual rendered contrast (values above are computed, not measured from a screenshot, so the WP-C screenshot check must confirm them); `players` empty-state key for the worlds menu.
