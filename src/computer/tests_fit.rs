//! Width tests of the card of the side panel: every row of every card fits one line at the
//! smallest window, and its buttons fit the rows kept for them.

use super::detail::{Card, ROWS, Row, card};
use super::layout::{DEED_ROWS, GLYPH, LINE_CHARS, NAME_CHARS, ROOM};
use super::tests_detail::{records, world};
use super::{Atlas, Block, RouteStatus, Target, View};
use crate::map::{named, offer};
use crate::ui::theme;
use crate::universe::{BodyId, PlanetClass, PlanetSpec, Rings, Universe};

/// Returns the card of `planet` on `line`, led by its line on life as the panel shows it.
fn alive(line: &super::Line, planet: &PlanetSpec, universe: &Universe) -> Card {
    let mut shown = card(line, Some(planet), universe);
    let life = super::odds::row(universe, planet, line.visited);
    shown.rows.insert(0, life);
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

/// Returns true when `row` of a card fits one line of the panel: a fact with its label, a gap
/// and its value, a heading in its spaced capitals, a note in two lines.
fn fits(row: &Row) -> bool {
    match row {
        Row::Heading(name) => name.chars().count() as f32 * (GLYPH + theme::CAPS_SPACING) <= ROOM,
        Row::Fact(name, value) => name.chars().count() + 1 + value.chars().count() <= LINE_CHARS,
        Row::Note(words) => words.chars().count() <= 2 * LINE_CHARS,
    }
}

/// Every row of every card fits one line of the panel, the name fits in the body font, and
/// no card has more rows than the panel keeps room for.
#[test]
fn every_card_fits_the_panel() {
    assert!(NAME_CHARS as f32 * theme::GLYPH_ASPECT * theme::FONT_BODY <= ROOM);
    let universe = Universe::new(7);
    for shown in planets(&universe).into_iter().chain(far(&universe)) {
        assert!(shown.rows.len() <= ROWS, "{shown:?}");
        assert!(
            shown.subtitle.chars().count() <= LINE_CHARS,
            "{}",
            shown.subtitle
        );
        let wide = shown.rows.iter().find(|row| !fits(row));
        assert!(wide.is_none(), "{wide:?}");
    }
}

/// Returns how many rows the buttons of the card of `place` take once wrapped, each button its
/// icon, the gap to its words, its words and its padding, and checks each fits a row alone.
fn rows_taken(place: Target, course: Option<Target>, status: RouteStatus) -> usize {
    let width = |words: &str| {
        theme::ICON + theme::GAP_S + words.chars().count() as f32 * GLYPH + 2.0 * theme::GAP_XS
    };
    let mut rows = (0, ROOM);
    for deed in offer(place, course, status) {
        let wide = width(named(deed, place).1);
        assert!(wide <= ROOM, "{deed:?}");
        let next = rows.1 + theme::GAP_XS + wide;
        rows = match next <= ROOM {
            true => (rows.0, next),
            false => (rows.0 + 1, wide),
        };
    }
    rows.0
}

/// The buttons of every card wrap into the two rows kept for them, whatever the course and the
/// route, and the reason the drive waits, which has its own line under them, fits two lines.
#[test]
fn the_buttons_of_the_card_fit_their_rows() {
    let universe = Universe::new(7);
    let home = universe.home();
    let places = [
        Target::Body(home),
        Target::Star(home.star),
        Target::Galaxy(home.star.galaxy),
    ];
    let (idle, landed) = (RouteStatus::Idle, RouteStatus::Blocked(Block::Landed));
    for place in places {
        assert!(rows_taken(place, None, idle) <= DEED_ROWS);
        assert!(rows_taken(place, Some(place), idle) <= DEED_ROWS);
        assert!(rows_taken(place, Some(place), landed) <= DEED_ROWS);
    }
    let inside = Block::Inside {
        star: true,
        lock: 99,
        tenths: 999,
    };
    for block in [Block::Landed, Block::Surface, inside] {
        assert!(
            block.words().chars().count() <= 2 * LINE_CHARS,
            "{}",
            block.words()
        );
    }
}
