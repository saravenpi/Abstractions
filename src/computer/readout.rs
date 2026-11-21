//! The card of the side panel. Its head never scrolls: the icon and the name of the selected
//! search result, what it is, and the Go button. Its facts scroll under it, in groups: every
//! fact has an icon, a label and a value with its unit, a planet adds a comparison with home and
//! ends its last group with its line on life, and its rarer facts wait behind a More row.

use bevy::prelude::*;

use super::board::Board;
use super::detail::{Card, Row, card as of, visible};
use super::face::{group, meaning, more, stat};
use super::format::clip;
use super::layout::{LINE_CHARS, NAME_CHARS};
use super::odds::Odds;
use super::section::kind;
use crate::ui::{Icon, Tone, icon, text, theme};
use crate::universe::Universe;

/// What the card of a place says while the table has no rows.
pub const NO_CARD: &str = "Pick a place on the map or in the list to see it here.";

/// Whether the rarer facts of the card show.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) struct More(pub bool);

/// Marks the icon that says what the place of the card is.
#[derive(Component)]
pub(super) struct Sign;

/// Marks the name of the place on the card.
#[derive(Component)]
pub(super) struct Title;

/// Marks the line under the name that says what the place is.
#[derive(Component)]
pub(super) struct Subtitle;

/// Marks the column the rows of the card are written in.
#[derive(Component)]
pub(super) struct Facts;

/// The first row of the card: the icon of what the place is, then its name.
fn name() -> impl Bundle {
    let node = Node {
        height: px(theme::ICON),
        align_items: AlignItems::Center,
        column_gap: px(theme::S2),
        overflow: Overflow::clip(),
        ..default()
    };
    let title = (
        Title,
        text("", theme::FONT_BODY, theme::TEXT),
        TextLayout::no_wrap(),
    );
    (node, children![(Sign, icon(Icon::Map, Tone::Plain)), title])
}

/// The head of the card, which is never cut and never scrolls: the icon and the name of the
/// place, what it is, and the Go button with its sentence. `write` fills it.
pub(super) fn card() -> impl Bundle {
    let node = Node {
        width: percent(100),
        flex_direction: FlexDirection::Column,
        flex_shrink: 0.0,
        ..default()
    };
    let said = (
        Subtitle,
        text("", theme::FONT_SMALL, theme::TEXT_2),
        TextLayout::no_wrap(),
    );
    let below = Node {
        width: percent(100),
        flex_direction: FlexDirection::Column,
        margin: UiRect::top(px(theme::GAP_S)),
        ..default()
    };
    (
        node,
        children![name(), said, (below, children![crate::map::deeds()]),],
    )
}

/// The empty column of the facts of the card, the first thing in the part of the panel that
/// scrolls. `write` fills it.
pub(super) fn facts() -> impl Bundle {
    let node = Node {
        width: percent(100),
        flex_direction: FlexDirection::Column,
        ..default()
    };
    (Facts, node)
}

/// Returns the card of the row under the cursor, or an empty card for an empty table. A planet
/// or a moon whose record is known ends its last group with its line on life, which `odds`
/// keeps while the same body stays selected.
pub(super) fn shown(board: &Board, universe: &Universe, odds: &mut Odds) -> Card {
    let world = board.worlds.get(board.cursor).and_then(Option::as_ref);
    let Some(line) = board.lines.get(board.cursor) else {
        return Card::default();
    };
    let mut card = of(line, world, universe);
    if let Some(body) = world {
        let at = card.rows.iter().position(|row| matches!(row, Row::More));
        let at = at.unwrap_or(card.rows.len());
        card.rows
            .splice(at..at, odds.rows(universe, body, line.visited));
    }
    card
}

/// Spawns one row of the card under `parent`. A heading that is not the first row keeps room
/// over it, which sets its group apart from the one above. `open` tells the More row what it
/// reads.
fn row(parent: &mut ChildSpawnerCommands, row: &Row, (first, open): (bool, bool)) {
    match row {
        Row::Heading(symbol, name) => {
            let gap = [theme::S3, 0.0][usize::from(first)];
            group(parent, (*symbol, *name), gap);
        }
        Row::Note(words) => {
            parent.spawn(text(*words, theme::FONT_SMALL, theme::TEXT_2));
        }
        Row::Stat {
            icon,
            label,
            value,
            note,
        } => stat(parent, (*icon, *label), (value.as_str(), note.as_str())),
        Row::Meaning(words) => meaning(parent, words),
        Row::More => more(parent, open),
    }
}

/// The icon, the name and the subtitle of the card, and the column of its rows.
type Parts<'w, 's> = (
    Query<'w, 's, &'static mut Text, With<Title>>,
    Query<'w, 's, &'static mut Text, (With<Subtitle>, Without<Title>)>,
    Query<'w, 's, &'static mut Text, (With<Sign>, Without<Title>, Without<Subtitle>)>,
    Query<'w, 's, Entity, With<Facts>>,
);

/// Writes the head of the card: the icon of the place, its name and what it is.
fn head(board: &Board, shown: &Card, (titles, subtitles, signs, _): &mut Parts) {
    let target = board.lines.get(board.cursor).map(|line| line.target);
    let sign = target.map_or(Icon::Map, kind).glyph();
    for mut text in titles.iter_mut() {
        text.set_if_neq(Text(clip(&shown.title, NAME_CHARS)));
    }
    for mut text in subtitles.iter_mut() {
        text.set_if_neq(Text(clip(&shown.subtitle, LINE_CHARS)));
    }
    for mut text in signs.iter_mut() {
        text.set_if_neq(Text(sign.clone()));
    }
}

/// Writes the card when the table, the cursor or the More row changed, and when the screen was
/// just opened. A table with no rows says how to pick a place.
pub(super) fn write(
    (board, universe, open): (Res<Board>, Option<Res<Universe>>, Option<Res<More>>),
    fresh: Query<(), Added<Facts>>,
    mut parts: Parts,
    mut commands: Commands,
    mut odds: Local<Odds>,
) {
    let flipped = open.as_ref().is_some_and(|open| open.is_changed());
    if !(board.is_changed() || flipped || !fresh.is_empty()) {
        return;
    }
    let open = open.is_some_and(|open| open.0);
    let mut shown = universe.map_or_else(Card::default, |universe| {
        shown(&board, &universe, &mut odds)
    });
    if shown.title.is_empty() {
        shown.rows = vec![Row::Note(NO_CARD)];
    }
    head(&board, &shown, &mut parts);
    for column in &parts.3 {
        commands
            .entity(column)
            .despawn_children()
            .with_children(|parent| {
                for (at, fact) in visible(&shown.rows, open).iter().enumerate() {
                    row(parent, fact, (at == 0, open));
                }
            });
    }
}
