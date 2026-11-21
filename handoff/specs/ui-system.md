# Interface system: audit, rules and components

Owner request for 0.11.0: harmonize margins and padding, the same rules on every page, real components, clean layouts. Tone set by the owner: Swiss design, clean gaps between section titles and their content, icons that make meaning faster to read.

This document has three parts: the audit of what the code does today, the system every screen follows, and the state of the conversion. Values are logical pixels at the 1280 by 720 reference. They were read from the code, not measured on a picture.

## 1. The system

### 1.1 Principles

- One grid. Every block is a whole number of grid columns wide. Every text of a page starts on one of two vertical lines: the icon line or the label line.
- Flush left, ragged right. Nothing is centred on a page. Values start on a shared left line, they are not centred between their buttons and not pushed to the right edge.
- Hierarchy comes from size, tone and space. Three sizes at most on a screen. `TEXT` for what matters, `TEXT_2` for what supports it. Spaced capitals for section titles. No rule, no box, no ornament.
- Space says what belongs together. A title is nearer to its content than to the block above it. This is the one rule for every title in the game.
- An icon is a word, not an ornament. One concept, one icon, on every screen, and its name in words beside it.

### 1.2 Tokens (`src/ui/theme.rs`)

| Token | Value | Use |
| --- | --- | --- |
| `S1` to `S6` | 4, 8, 16, 24, 32, 48 | Every padding, margin and gap |
| `EDGE` | `S5` = 32 | Screen edge, on every side of every full-screen page |
| `INSET` | `S2` = 8 | From the edge of a row's fill to its text. Text that is not in a row is inset by the same 8, so all text shares one line |
| `ICON` | 24 | The icon slot. One size, whatever the text size beside it |
| `ICON_GAP` | `S2` = 8 | Between an icon and its label |
| `COLUMN`, `GUTTER` | 48, `S5` = 32 | The alignment grid |
| `span(n)` | `n * 80 - 32` | Width of `n` columns: 48, 128, 208, 288, 368, 448, 528, 608, 688, 768, 848 |
| `SECTION_ABOVE` | `S5` = 32 | Above a section title on a page |
| `GROUP_ABOVE` | `S4` = 24 | Above a group title inside a card or a panel |
| `TITLE_BELOW` | `S2` = 8 | Between a title and its content, everywhere |
| `ROW_HEIGHT` | 32 | A row of one line, an action, a menu entry. The smallest hit target |
| `ROW_TALL` | `S6` = 48 | A row of two lines: a name and a fact under it |
| `VALUE_WIDTH` | 168 | The value column: 14 glyphs of body text |
| `VALUE_CLUSTER` | 224 = `span(3) + S3` | The value zone of a row: the value column and the room of two step buttons |
| `FONT_SMALL`, `FONT_BODY`, `FONT_HEADING`, `FONT_TITLE` | 16, 20, 32, 72 | The type scale |

### 1.3 The grid

The page is 11 columns wide at the smallest logical width (960: 896 inside the edges, `span(11)` = 848) and 15 at 1280.

```text
x = 32        40        72                                              
    |edge     |icon line |label line                                     
    [ fill of a row ............................................ ]       
              [i]        Label                        Value      < >     
```

- A row's fill starts at the screen edge, 32.
- The icon line is at 40: `EDGE + INSET`. The page title, section icons, row icons and the back action's icon start there.
- The label line is at 72: `40 + ICON + ICON_GAP`. Section titles, row labels and the reason of an empty state start there. A row without an icon keeps the slot, so its label does not move.
- The value line of a settings page is the left edge of the value zone, the same x for every row of every section of the page.

### 1.4 The page skeleton (`ui::page`)

Every full-screen page has three zones, built by `page(head, body, foot)`:

```text
y = 0
    EDGE 32
    heading zone   40 high (LINE_HEADING). Title, 32, TEXT, on the icon line. Status, 16, TEXT_2, at the right
    S5 32
    content zone   takes the room left. Starts at y = 104 on every page
    S4 24
    footer zone    32 high (ROW_HEIGHT). Back action first, then the key legend
    EDGE 32
```

- `PAGE_HEAD` = 104 and `PAGE_FOOT` = 88 are the heights above and below the content zone. A fit test adds them, it does not guess them.
- The back action lives in the footer zone, first, on every page: icon, a word for what Escape does there ("Back", "Resume", "Cancel"), and "Esc". A page has no other Back button.
- The status of a page (a count, "Saved") lives at the right end of the heading zone.
- The legend is `ui::legend` from `map-ui`: label first in `TEXT`, key after it in `TEXT_2`, `KEY_GAP` = S3 between them and `HINT_GAP` = S5 between hints.
- Fill by context: a page over the blurred game is `SCRIM` on `Blurred` and its content has no panel. A panel over the live world is `PANEL`, a row that can hold `WARN` or `DANGER` text is `PANEL_DEEP`, a big panel over the blur is `PANEL_BLUR`, a panel over a slide of the title screen is `PANEL`.

### 1.5 Section titles (`ui::section`)

One rule everywhere: more space above a title than below it.

| Where | Above the title | Below the title |
| --- | --- | --- |
| A section of a page (`sections()`) | `SECTION_ABOVE` = S5 = 32 | `TITLE_BELOW` = S2 = 8 |
| A group inside a card or a panel (`groups()`) | `GROUP_ABOVE` = S4 = 24 | `TITLE_BELOW` = S2 = 8 |

Annotated example, the Graphics page of the settings:

```text
x=32      40  72                                   424
          Settings                                          y = 32   page title, 32 px, 40 high
                                                            32 (S5)  heading zone to content
          [o] VIEW                                          y = 104  title: 16 px spaced capitals, TEXT_2, 24 high
                                                            8  (S2)  title to its rows
    [     [>] Render distance                      Medium      < > ]   row, 32 high, body 20
    [         Field of view                        90 deg      < > ]
                                                            32 (S5)  above the next title
          [*] LIGHT
                                                            8  (S2)
    [     [*] Lighting quality                     High        < > ]
    [         Shadows                              On              ]
```

- The title icon sits on the icon line and the title on the label line, so the rows under a title share its left edge.
- The first title of a page has the 32 of the heading zone above it. The rule holds there too: 32 above, 8 below.
- A section is `section(icon, title, body)`. Put sections in `sections()` on a page and in `groups()` in a card. Never set a margin on a title by hand.

For `map-ui`: the map card, the guide card and the side panel should build their titled groups with `ui::groups()` and `ui::section(icon, title, body)`. Rows inside use `ui::rows()`. That gives 24 above and 8 below with no number in the screen file.

### 1.6 Components

All in `src/ui`. A screen file names content. It sets no pixel.

| Component | Function | Layout | Type and colour |
| --- | --- | --- | --- |
| Page | `page`, `page_head`, `page_title`, `page_status`, `page_body`, `page_foot` | 1.4 | Title 32 `TEXT`, status 16 `TEXT_2` |
| Back action | `back(words)` | First in the footer zone | 16, icon `Back`, key "Esc" |
| Section | `sections`, `groups`, `section`, `section_title`, `rows` | 1.5 | Title 16 capitals `TEXT_2`, icon `TEXT_2` |
| Menu entry | `item(icon, label, tone)` | Full width of its column, 32 high, padding `INSET` by `S1`, icon slot, chevron when current | 20, `Tone` |
| Settings row | `row_with(group, name, value)` with `entry(icon, label, ..)` and a `VALUE_CLUSTER` zone | 32 high, padding `INSET`, name left, value zone right, `S3` between | Name 20 `TEXT_2`, value 20 `TEXT`, "Off" `TEXT_2` |
| Row states | `Look`, `Selected` | Rest: no fill. Current (focus or pointer): `HOVER`. Chosen: `SELECT` | Dimmed texts go to `TEXT` on the current and the chosen row |
| Action | `action(icon, label, key, tone)` | 32 high, padding `INSET`, sizes to content, pointer only | 16. `Tone::Accent` primary, `Plain`, `Danger` for a destructive one. Key in `TEXT_2` |
| Action bar | `strip()` | A row of actions, `S3` apart, `S4` above it | |
| Table | `table_head`, `table_row`, `head`, `cell` | Cells are `span(n)` wide, a gutter apart. Head 28 high, row 32 or `ROW_TALL` | Head 16 capitals `TEXT_2`, words only |
| List | `scroll_area(children)` from `map-ui` | Takes the height left, bar in the gutter | Bar `TEXT_2` |
| Empty state | `empty_state(icon, title, why)` | `span(6)` wide, icon line and label line, the action that fills the list follows | Title 20 `TEXT`, why 16 `TEXT_2` |
| Notice | `notice()`, `notice_row(icon, words)` | One line, keeps its height while empty | 16 `TEXT`, `WARN` or `DANGER` by meaning |
| Key hint, legend | `key_hint`, `legend` from `map-ui` | 1.4 | |
| Tooltip | `Tip` | Padding `S3` by `S2`, `PANEL` | 16 |
| Meter | `meter(width, share)` | `METER` = `S1` high, the share filled | Fill `TEXT` over a `HAIRLINE` track |
| Stat row | `stat(label, share, value)` | One line of `LINE_SMALL`: label in `STAT_LABEL` = 64, meter of `STAT_METER` = 64, value, `S2` apart | 16, label `TEXT_2`, value `TEXT` |
| Card | `card(chosen)` | Pointer button, padding `S3`, blocks `S3` apart side by side, the last ones wrap under the others on a narrow card | Rest none, pointer `HOVER`, chosen `SELECT` |
| Tall row | `row_tall(icon, chosen, top, under)` | Pointer button, `ROW_TALL` high, padding `INSET`, icon slot, then two `line()`s | Rest none, pointer `HOVER`, chosen `SELECT` |
| Line | `line()` | One line inside a row or a card, children `S2` apart, one child with `flex_grow` pushes the rest to the right end | |
| Reveal | `Reveal` | A node under a screen's own cursor: its scroll area brings it into view once, then the wheel scrolls freely | |

To build in round 2, with the screens that use them:

| Component | Layout | Type and colour |
| --- | --- | --- |
| Form field | Label as a section title (icon, capitals) with `TITLE_BELOW` under it, box `ROW_TALL` high and `span(6)` wide, padding `INSET`, fill `HOVER`, `GROUP_ABOVE` between fields | Value 20 `TEXT`, placeholder `TEXT_2` |
| Card | Padding `S4`, fill by context, groups inside `GROUP_ABOVE` apart | |
| Modal confirm | The question replaces the row it is about, the two answers are an action bar, the safe one chosen first | Question 20, `Tone::Danger` on the destructive answer |
| HUD strip | One line, padding `S3` by `S2`, gap `S2`, icon 16 in a 16 line, fill `PANEL` or `PANEL_DEEP`, `S2` between two strips | 16 |
| HUD panel | Padding `S4`, fill `PANEL`, rows 24 high with icon, label column, value | 16 |
| Toast | A HUD strip that fades in `FADE` and leaves by itself | 16 |

### 1.7 Icons

- One slot, 24 by 24, one gap of `ICON_GAP` to the label. The slot stays when a row has no icon.
- A section title, a row, an action, a tab, a menu entry, a notice and an empty state take an optional leading icon. A table head takes none: its columns are named in words, and a row shows its kind or state with an icon in its first cell.
- A row gets an icon when a glyph tells it apart from its neighbours. A row whose concept is the concept of its section leaves the slot empty: the title carries the icon.
- A variant of `ui::Icon` is named after its concept, and that is the vocabulary. A concept that has no glyph yet borrows one through a constant of `ui::sign`, which names the glyph it wants.

Vocabulary used by the converted screens:

| Concept | Icon | Glyph |
| --- | --- | --- |
| Back, cancel, resume | `Back` | `arrow_back` |
| Play, continue | `Play` | `play_arrow` |
| Create, new | `Plus` | `add` |
| Rename, edit | `Pencil` | `edit` |
| Copy a world | `Copy` | `content_copy` |
| Delete | `Trash` | `delete` |
| Sort, invert an axis | `Sort` | `swap_vert` |
| Confirm | `Check` | `check` |
| Keep, close | `Close` | `close` |
| A world, the game itself | `Planet` | `planet` |
| Settings, the General tab | `Settings` | `tune` |
| The display, graphics, the interface | `Monitor` | `monitor` |
| Keys | `Keyboard` | `keyboard` |
| Mouse | `Mouse` | `mouse` |
| The view, the camera | `Eye` | `visibility` |
| Light | `Light` | `light_mode` |
| Clouds, the sky | `Cloudy` | `cloud` |
| The sea, water | `Water` | `water_drop` |
| A distance | `Distance` | `distance` |
| Sound | `Volume` | `volume_up` |
| The ship | `ShipParked` | `rocket` |
| On foot, a player | `User` | `person` |
| Speed, frame rate | `Speed` | `speed` |
| Markers, pins | `MapPin` | `location_on` |
| Updates, refresh | `Reload` | `refresh` |
| The robot lamp | `HeadlightOn` | `flashlight_on` |
| Bloom (borrowed) | `Radial` | `flare` |
| Hitstop (borrowed, wants `front_hand`) | `Pause` | `pause` |
| Interface size (borrowed, wants `format_size`) | `Zoom` | `zoom_in` |
| The pack | `Pack` | `backpack` |
| The workshop, a made tool | `Workshop` | `construction` |
| Cut, pierce, crush, throw | `Cut`, `Pierce`, `Crush`, `Throw` | `content_cut`, `push_pin`, `hardware`, `sports_handball` |
| Stone (borrowed, wants `landslide`) | `Terrain` | `landscape` |
| Bone | `Skeleton` | `skeleton` |
| Horn and shell | `Shield` | `shield` |
| Hide (borrowed) | `Animal` | `pets` |
| Wood | `Tree` | `park` |
| Flint, obsidian, glass (borrowed) | `Crystal` | `diamond` |
| Ice (borrowed, wants `ac_unit`) | `Snow` | `weather_snowy` |
| Resin (borrowed) | `Water` | `water_drop` |
| A made tool, by intent when it has no parts | `Cut`, `Pierce`, `Crush`, `Throw` | see above |

Made tools have no glyph: each has a picture drawn from its parts (`craft::ui::glyph`). The head's figure tells the kind (knife, blade, axe, spear, javelin, club, hammer), the shaft's length and each part's grey tell the rest: stone white, bone, horn and shell, then wood dimmest, no colour. The tool lies on the diagonal, head at the top right, shaft one twelfth of the picture wide, as the font icons draw their strokes, flat with no shadow and no outline. Each pixel is on or off, tested at its centre, and the picture is drawn at the physical pixel size its node shows at, its node sized to cover exactly those pixels, sampled to the nearest: crisp at every interface size. It is drawn once per look and size and kept as an image. Sizes: 24 in a pack row and a slot of the pack screen, 48 on a workshop card, 16 under the number of a slot of the game's hotbar. A tool with no parts shows the glyph of its intent.
| Make a tool | `Plus` | `add` |
| Swap a part | `Shuffle` | `shuffle` |
| A strength, a flaw | `Check`, `Danger` | `check`, `warning` |
| A note after an action | `Info` | `info` |

Glyphs the subset lacks, by Material Symbols name. Rows that need them keep an empty slot today:

| Wanted for | Glyph |
| --- | --- |
| Shadows, shadow detail, cloud shadows | `ev_shadow` |
| Ambient occlusion | `blur_circular` |
| Auto exposure | `exposure` |
| White balance | `wb_auto` |
| Vsync | `sync` |
| Fullscreen | `fullscreen` |
| Field of view | `panorama_horizontal` |
| Head bob | `vibration` |
| Camera auto-follow | `videocam` |
| Music | `music_note` |
| Effects volume | `graphic_eq` |
| Blood | `bloodtype` |
| Reduce motion | `animation` |
| Hitstop | `front_hand` |
| Interface size | `format_size` |
| The seed of a world | `tag` |
| A password | `key` |
| A server address | `lan` |

### 1.8 Tests that hold the rules

| Test | File | Holds |
| --- | --- | --- |
| `every_node_is_on_the_scale_and_plain` | `src/menus/tests/rules.rs` | Paddings, margins and gaps on the scale, positions and sizes on the 4 grid, no border, radius, outline or shadow |
| `every_text_is_on_the_type_scale` | same | Sizes 16, 20, 32, 72, three at most per page, icons at 24 |
| `every_colour_is_a_token` | `src/menus/tests/rules_colour.rs` | Fills and text colours are theme tokens |
| `every_page_has_the_same_skeleton` | `src/menus/tests/rules_page.rs` | Edge `S5` on all sides, the three zones in order with the same measures, so headings and footers sit at the same place |
| `a_section_title_is_nearer_to_its_content` | same | For every section: gap below is `TITLE_BELOW`, gap above is one of the two allowed and larger |
| `every_icon_is_named_in_words` | same | An icon has a text in its control or row, or a `Tip` |
| `spans_sit_on_the_grid`, `the_space_above_a_title_is_larger_than_the_space_below`, `a_concept_maps_to_one_icon` | `src/ui/rules_tests.rs` | The grid, the title gaps, one glyph per icon and one entry per borrowed concept |

The walking tests run on the pages listed in `PAGES` of `rules.rs`. A page joins the list when it is converted.

## 2. Audit of the code before the conversion

Verdicts: **break** means it breaks a rule of AGENTS.md, **drift** means it differs from its siblings for no reason, **ok** means it holds.

### 2.1 Shared by every menu

| Screen | Element | Value today | Rule | Verdict |
| --- | --- | --- | --- | --- |
| All menus | Root side padding (`root_on`) | 5 percent: 64 at 1280, 48 at 960 | Screen edge `S5` = 32 | break |
| All menus | Root top and bottom padding | 32 | `S5` | ok |
| All menus | Page heading (`heading`) | 16 capitals, `TEXT_2`, 24 below | A heading is the largest text of its page | drift: the weakest text on the page is its title |
| All menus | Footer hints (`footer`) | Absolute, left and right 5 percent, bottom 32 | `S5`, in the flow | break |
| All menus | Text edge | Heading at x = 64, row text at x = 72 (rows pad 8) | One left line | drift |
| All menus | Menu entry (`item`) | Padding 8 by 4, 32 high by its content, not by a set height | `ROW_HEIGHT` | ok, implicit |
| All menus | Action (`action`) | Padding 16 by 8, 40 high | Row height 32, padding as a row | drift: an action is taller than a row and its icon is 8 right of a row's |
| All menus | Action bar (`strip`) | Margin 16 above and below | | drift: also 16 under the last element of a page |
| All menus | Back | A row at the bottom of the sidebar (settings), the last action of the bar (worlds, multiplayer), an entry under the list (players), absent (pause) | One place | drift |

### 2.2 Menus

| Screen | Element | Value today | Rule | Verdict |
| --- | --- | --- | --- | --- |
| Main | Panel | Padding 24, margin -24, `PANEL` | Panels pad `S4`. A negative margin puts the fill 24 outside the screen edge zone | drift: the panel edge is at 40, nothing else is |
| Main | Title | 72, with a 48 by 2 rule under it, margins 8 and 24 | No ornament | drift: the rule is the only ornament in the game |
| Main | Entry column | 300 wide | 4 grid, shared widths | drift: 300 is on no grid |
| Worlds | Column widths | 270, 165, 110, 160, gaps 16 | Multiples of 4 | break: 270, 165 and 110 |
| Worlds | Row height | 44 | Spacing or row scale | drift: 44 is on no scale |
| Worlds | Empty state title | 32 under a 16 page heading | Hierarchy | drift: the empty state outweighs the page title |
| Worlds | Summary and notice | Two reserved lines of 24 under the table | | ok, but costs 48 of height on a small window |
| New world | Field | 450 by 48, padding 8, 24 below, label 4 above its box | 4 grid | break: 450. Label gap 4 against 24 below: the right way round, by other numbers than elsewhere |
| New world | Actions column | 300 | | drift |
| Multiplayer | Column widths | 420, 150, row 44, list 300 high | | drift: 150 and 44 on no scale |
| Multiplayer | Caption (section title) | 16 above, 4 below | One title rule | drift: third pair of numbers for a title |
| Multiplayer | Empty line | Margin 8 on all sides | Says why | ok on words, no icon |
| Multiplayer | Form | Field 48 high, 16 below, label gap 4, title 20 | | drift from the new world field (24 below) |
| Joining | Line | 32, waiting line with 16 above and below | | ok |
| Settings | Sidebar | 180 wide, 48 to the page | 4 grid | drift: gutter 48 here, 24 in the pack |
| Settings | Page | 600 wide, rows 32, no sections: 18 rows in one block on Graphics | | drift: no grouping at all |
| Settings | Values | Centred between the step buttons on a stepper, pushed right on a toggle | One value line | break of alignment: two value positions on one page |
| Settings | Hint line | 8 above, 24 high | | ok |
| Settings | Height fit | `CHROME` = 240, tuned by hand | | drift: a number nobody can derive |
| Settings | Reference | Two halves of 288, key column 136, key before action | | drift: the opposite order of the rows of the other tabs |
| Settings | General tab icon | `Home` | One concept, one icon | break: `Home` is the home base everywhere else |
| Pause | Entry column | 300: "Save and quit the game" is cut | Fits | break: clipped text |
| Pause | Corner texts | Absolute, 32 from the right, top and bottom | `S5` | ok |
| Players | Column | 300, gap 8, Back 24 under it | | drift: the only entry list with a gap |
| Update | Page | 600 wide, notes 160 high, bar 4, gaps 16 and 8 | | ok on the scale, 600 on no grid |

### 2.3 HUD

| Screen | Element | Value today | Rule | Verdict |
| --- | --- | --- | --- | --- |
| HUD | Bottom-left stack | Left 32, bottom 32, gap 8 | `S5` | ok |
| HUD | Instrument panel | Padding 24 by 16, `PANEL` | Panels pad `S4` | break: 16 vertical |
| HUD | Instrument row | Gap 8, icon 16, label column 112, value | | ok |
| HUD | Prompt | Bottom 96 (32 + 56 + 8), padding 16 by 8, `PANEL` or `PANEL_DEEP` | | ok |
| HUD | Banner | Top 88, padding 24 by 16, name 32 | | ok |
| HUD | Top-left corner | Top 32, left 32, coordinates strip and climate strip touch | Separate by space | drift: gap 0 between two strips, 4 between frame rows, 8 in the stack |
| HUD | Strips | Padding 16 by 8, `PANEL` | | ok |
| HUD | Frame rows | Gap 4, padding 16 by 8, `PANEL_DEEP` | | ok |
| HUD | Compass | 360 by 32 at top 32, base rule at top 31, tick at 50 percent | Multiples of 4 | break: 31 |
| HUD | Compass marks | Glyph 16 in `Color::WHITE`, herd colour a raw `srgba` | Tokens | break: raw colours |
| HUD | Minimap | Top 32, right 32, side in `Vh` clamped to 128 to 240 | | ok on the edge, size not in pixels |
| HUD | Minimap needle | Bars 2, 6, 10 by 3 | 4 grid | break, small |
| HUD | Minimap caption | Padding 8 by 4, `PANEL` | Strip padding 16 by 8 | drift: a second strip padding |
| HUD | Markers over the world | Glyph and arrow in `Color::WHITE`, label padding 4, reach 22 | Tokens | break: raw colours |
| HUD | Snow | Raw `srgba` greys | The picture is free | ok: an effect, not chrome |
| Hotbar | Row | Bottom 32, gap 8, slots 56 by 56, padding 4 by 8 | | ok, 56 is a size |
| Hotbar | Empty slot | Raw `srgba(0.02, 0.025, 0.035, 0.32)` | Tokens | break: a fill that is no token |
| Pause HUD | Corners | 32 from the edges | | ok |

### 2.4 Pack, workshop, guide, map

| Screen | Element | Value today | Rule | Verdict |
| --- | --- | --- | --- | --- |
| Pack | Screen edge | 24 (`guide::layout::EDGE`) | `S5` | break |
| Pack | Header | 73.6 high (32 by 1.2, 19.2, 16) | 4 grid | break |
| Pack | Tabs | 32, gap 24 | | drift: tabs as headings here, as a sidebar in the settings |
| Pack | List and card | 440 and 600, 24 apart | Grid, gutter | drift: gutter 24 here, 48 in the settings |
| Pack | List row | 28 high, padding 8, gap 8 | `ROW_HEIGHT` | drift: 28 here, 32 in the settings |
| Pack | Card | Title 20, subtitle 16, group title with 8 above and 0 below | Title rule | break: a group title is nearer to the block above than to its content |
| Pack | Footer | 27.2 high | 4 grid | break |
| Guide | Same layout constants | `EDGE` 24, `HEAD` 73.6, `FOOT` 27.2 | | break, owned by `map-ui` |
| Map | Side panel | 384 wide, padding 24, gap 16, `PANEL_BLUR`, flush to the top, right and bottom of the window | Screen edge | break, owned by `map-ui` |
| Map | Words over the picture | Padding 32, lines 19.2 high | | 19.2 off the grid, owned by `map-ui` |

### 2.5 Screens

| Screen | Element | Value today | Rule | Verdict |
| --- | --- | --- | --- | --- |
| Title | Root | 5 percent side padding | `S5` | break |
| Title | Version | Right 32, bottom 32, padding 8 by 4, `PANEL` | Strip padding 16 by 8 | drift |
| Loading | Root | 5 percent side padding, content from the bottom | `S5` | break |
| Loading | Lines | Heading 32, detail 16 with 8 above and below, track 300 by 2 | | drift: 300 |
| Boot | Root | 5 percent side padding, centred | `S5`, nothing centred | break and drift |
| Boot | Lines | Gap 8, label column 250, letter spacing 2, body 20 | 4 grid, `CAPS_SPACING` 3 | break: 250. Drift: a second letter spacing |
| Boot | Skip hint | Left 5 percent, bottom 32 | `S5` | break |
| Stall banner, damage panel | | No interface node: the damage panel is a 3D plate | | not in scope |

### 2.6 What the audit says, in short

- The screen edge is 32 in the HUD, 5 percent in every menu and screen, and 24 in the pack and the guide. Three edges.
- A title has four different gap pairs: 24 below (page heading), 16 above and 4 below (multiplayer), 8 above and 0 below (cards), 4 below (fields).
- Row heights: 28, 32, 44. Gutters: 16, 24, 48. Column widths: 180, 250, 270, 288, 300, 320, 384, 420, 440, 450, 600, on no grid.
- Back is in four different places.
- Values of a settings page sit on two lines.

## 3. Conversion

### 3.1 Round 1, done

- Components: `ui::page`, `ui::section`, `ui::state`, `ui::sign`, the grid and the title gaps in `ui::theme`, `back` in `ui::chrome`. `action` is now a row: 32 high, padding `INSET`. Table rows put their cells a gutter apart.
- Settings, every tab: page skeleton, tabs on `span(3)`, page on `span(8)`, sections with icons, rows with icons, values on one line, the page scrolls in `scroll_area` and takes the height the window has. `fit.rs` and its hand-tuned `CHROME` are gone. The reference is laid out as rows. The General tab wears `Settings`.
- Worlds: page skeleton, table on `span(4, 3, 2, 2)`, rows `ROW_TALL`, the count in the heading zone, the empty state component, Back in the footer. Five rows show at once, so the page fits a 540 high logical window.
- Changed words: "Duplicate" is "Copy", "New world" in the bar is "New", "Last played" is "Last seen", "Check for updates at launch" is "Check for updates" with its description under the rows.

### 3.2 Round 2, to do

Main menu (title on the icon line, no rule, entries on `span(4)`), new world and multiplayer (form field component), pause and players, update notes, joining, the HUD hotbar, HUD strips and panels, loading and boot, the title version strip.

Pack and workshop, done (`src/craft/ui`): one page over the blur. Heading zone: "Pack and workshop", the load as the status. Content zone: the pack pane on `span(4)` (title, groups by kind in a scroll area, one `row_tall` per entry, Hold and Drop as actions), a gutter, the workshop pane taking the rest (title, four intent tiles `ROW_TALL` high, the line on the intent, the three tool cards as `card`s in a scroll area, Make in `ACCENT` first in the action bar with its note), and the hotbar under both, six slots `ROW_TALL` high, `S4` above. Footer: Close (Esc), then the legend of the pane the keys drive. It fits 960 by 540 logical: the cards and the pack scroll. `fauna::guide::layout` no longer serves it.

Once the rest of round 2 is done, `root`, `root_on`, `panel`, `heading`, `rule`, `keys::footer`, `MENU_WIDTH` and the `GAP_*` aliases can go.

### 3.3 Open questions for the owner

1. Values flush left on one line, with the two step buttons after them at the right of the row, in place of the value centred between its buttons. This is the Swiss reading of "a value column aligns". Say if the old `< value >` is wanted back: it can also sit on one line, centred.
2. Settings rows without a glyph keep an empty icon slot until the font has the glyphs of 1.7. The other choice is no row icons at all, sections only.
3. The page title is now 32, in place of 16 capitals. The capitals remain for section titles.
4. Five worlds show at once, in place of six, so that the page fits the smallest window at the Large interface size.
