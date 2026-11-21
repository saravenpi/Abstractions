//! Width tests of the card of the side panel: every row of every card fits one line at the
//! smallest window, and every sentence of Go fits two lines.

use super::course::hud_line;
use super::detail::{Card, Row, card, visible};
use super::layout::{GLYPH, LINE_CHARS, NAME_CHARS, ROOM};
use super::route::Keys;
use super::tests_detail::{ROWS, records, world};
use super::{Atlas, Block, RouteStatus, View};
use crate::ui::theme;
use crate::universe::{BodyId, PlanetClass, PlanetSpec, Rings, Universe};

/// Characters the icon of a stat takes, with the gap after it.
const ICON_CHARS: usize = 3;

/// Characters the column of notes of a stat takes at the right end of its row.
const NOTE_CHARS: usize = 12;

/// Returns the card of `planet` on `line`, with its line on life at the end of its last group
/// as the panel shows it.
fn alive(line: &super::Line, planet: &PlanetSpec, universe: &Universe) -> Card {
    let mut shown = card(line, Some(planet), universe);
    let life = [
        super::odds::row(universe, planet, line.visited),
        super::odds::meaning(universe, planet, line.visited),
    ];
    let more = shown.rows.iter().position(|row| matches!(row, Row::More));
    let at = more.unwrap_or(shown.rows.len());
    shown.rows.splice(at..at, life);
    shown
}

/// Returns the cards of every class, of a gas giant with the most moons and wide rings, of a
/// moon and of a planet not charted, each charted one led by its line on life.
fn planets(universe: &Universe) -> Vec<Card> {
    let classes = records().map(|record| {
        let (line, planet) = world(record);
        alive(&line, &planet, universe)
    });
    let (line, planet) = world(records()[0]);
    let mut giant = PlanetSpec {
        moons: 6,
        rings: Some(Rings {
            inner: 9.9e8,
            outer: 9.9e9,
            density: 0.5,
            color: Default::default(),
        }),
        ..planet.clone()
    };
    giant.record.class = PlanetClass::GasGiant;
    let moon = PlanetSpec {
        id: BodyId {
            moon: 1,
            ..planet.id
        },
        ..planet
    };
    let others = [&giant, &moon].map(|body| alive(&line, body, universe));
    let uncharted = card(&line, None, universe);
    classes
        .into_iter()
        .chain(others)
        .chain([uncharted])
        .collect()
}

/// Returns the cards of the home star and of every known galaxy.
fn far(universe: &Universe) -> Vec<Card> {
    let atlas = Atlas::new(universe);
    let view = View {
        universe,
        atlas: &atlas,
        ship: Default::default(),
        time: 0.0,
    };
    let star = view.system(universe.home().star).into_iter().take(1);
    let lines = star.chain(view.universe());
    lines.map(|line| card(&line, None, universe)).collect()
}

/// Returns true when `row` of a card fits one line of the panel: a stat with its icon, its
/// label, its value and its note column, a heading after its icon in its spaced capitals, a
/// note or a meaning in two lines.
fn fits(row: &Row) -> bool {
    match row {
        Row::Heading(_, name) => {
            let caps = name.chars().count() as f32 * (GLYPH + theme::CAPS_SPACING);
            theme::ICON + theme::S2 + caps <= ROOM
        }
        Row::Note(words) => words.chars().count() <= 2 * LINE_CHARS,
        Row::Meaning(words) => words.chars().count() <= 2 * LINE_CHARS,
        Row::More => true,
        Row::Stat {
            label, value, note, ..
        } => {
            let room = LINE_CHARS - ICON_CHARS - [NOTE_CHARS, 0][usize::from(note.is_empty())];
            let words = label.chars().count() + 1 + value.chars().count();
            words <= room && note.chars().count() <= NOTE_CHARS
        }
    }
}

/// Every row of every card fits one line of the panel, the name fits in the body font, and
/// no card has more rows than the panel keeps room for. The subtitle fits with a star name of
/// six letters: the home star's generated name may be longer, and the panel clips it like any name.
#[test]
fn every_card_fits_the_panel() {
    let name = NAME_CHARS as f32 * theme::GLYPH_ASPECT * theme::FONT_BODY;
    assert!(theme::ICON + theme::S2 + name <= ROOM + 0.01);
    let universe = Universe::new(7);
    let star = universe.system(universe.home().star).star.name;
    for shown in planets(&universe).into_iter().chain(far(&universe)) {
        assert!(visible(&shown.rows, false).len() <= ROWS, "{shown:?}");
        let words = shown.subtitle.replace(&star, "Varena");
        assert!(words.chars().count() <= LINE_CHARS, "{}", shown.subtitle);
        let wide = shown.rows.iter().find(|row| !fits(row));
        assert!(wide.is_none(), "{wide:?}");
    }
}

/// Every measure of every planet card writes its unit: kilometres, multiples of home, degrees Celsius, and
/// minutes, hours or days for time.
#[test]
fn every_measure_writes_its_unit() {
    let universe = Universe::new(7);
    let units = [
        ("Radius", " km"),
        ("Gravity", " x home"),
        ("Air", " x home"),
        ("Deck air", " x home"),
        ("Temperature", " C"),
    ];
    for shown in planets(&universe) {
        for row in &shown.rows {
            let Row::Stat { label, value, .. } = row else {
                continue;
            };
            if let Some((_, unit)) = units.iter().find(|(name, _)| name == label) {
                assert!(value.ends_with(unit), "{label}: {value}");
            }
            if matches!(*label, "Day" | "Year" | "Orbit") {
                assert!(
                    value.ends_with("min") || value.ends_with("days"),
                    "{label}: {value}"
                );
            }
        }
    }
}

/// Every sentence Go can say fits the two lines the card keeps for it, with the longest name
/// of a place and the longest key names.
#[test]
fn the_go_sentences_fit_two_lines() {
    let keys = Keys {
        board: "Num Enter".to_string(),
        rise: "Space".to_string(),
        go: "Num Enter".to_string(),
    };
    let star = Block::Inside {
        star: true,
        lock: 50,
        tenths: 30,
    };
    let name = "x".repeat(NAME_CHARS);
    let route = [
        RouteStatus::Blocked(Block::OnFoot),
        RouteStatus::Blocked(Block::Landed),
        RouteStatus::Blocked(star),
        RouteStatus::Climbing,
        RouteStatus::Dropping,
        RouteStatus::Aligning(50),
        RouteStatus::Spooling,
        RouteStatus::InWarp,
        RouteStatus::Arrived,
    ];
    for status in route {
        let words = hud_line(&name, status, &keys);
        assert!(words.chars().count() <= 2 * LINE_CHARS, "{words}");
    }
}

/// Every row of every card says what it is with a picture: a heading and a stat carry an icon
/// that has a glyph, and only sentences, which explain the row above them, and the More row,
/// which draws its own, go without. The cards of a star and of a galaxy have no bare row.
#[test]
fn every_row_of_every_card_has_an_icon() {
    let universe = Universe::new(7);
    let cards: Vec<Card> = planets(&universe)
        .into_iter()
        .chain(far(&universe))
        .collect();
    assert!(cards.len() > 8);
    let mut seen = 0;
    for shown in &cards {
        for row in &shown.rows {
            let icon = match row {
                Row::Heading(icon, _) | Row::Stat { icon, .. } => *icon,
                Row::Note(_) | Row::Meaning(_) | Row::More => continue,
            };
            assert!(crate::ui::Icon::ALL.contains(&icon), "{row:?}");
            seen += 1;
        }
    }
    assert!(seen > 60, "{seen} rows with an icon");
    for shown in far(&universe) {
        let bare = shown
            .rows
            .iter()
            .filter(|row| !matches!(row, Row::Heading(..) | Row::Stat { .. } | Row::Note(_)));
        assert_eq!(bare.count(), 0, "{shown:?}");
        assert!(shown.rows.iter().any(|row| matches!(row, Row::Stat { .. })));
    }
}
