//! The card of the side panel: the name of the selected search result, what it is, the Go
//! button, and its facts in groups. A planet shows stats with an icon, a unit and a comparison
//! with home, and ends its last group with its line on life; its rarer facts wait behind a More
//! row.

use bevy::prelude::*;

use super::board::Board;
use super::detail::{Card, Row, card as of, visible};
use super::face::{meaning, more, stat};
use super::format::clip;
use super::layout::{LINE_CHARS, NAME_CHARS};
use super::odds::Odds;
use super::section::{heading, heading_row};
use crate::ui::{caps, text, theme};
use crate::universe::Universe;

/// What the card of a place says while the table has no rows.
pub const NO_CARD: &str = "Pick a place on the map or in the list to see it here.";

/// Whether the rarer facts of the card show.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) struct More(pub bool);

/// Marks the name of the place on the card.
#[derive(Component)]
pub(super) struct Title;

/// Marks the line under the name that says what the place is.
#[derive(Component)]
pub(super) struct Subtitle;

/// Marks the column the rows of the card are written in.
#[derive(Component)]
pub(super) struct Facts;

/// The empty card, which is cut at its bottom when the panel is too short for all of it.
/// `write` fills it.
pub(super) fn card() -> impl Bundle {
    let node = Node {
        width: percent(100),
        flex_direction: FlexDirection::Column,
        min_height: px(0),
        overflow: Overflow::clip(),
        ..default()
    };
    let small = |color: Color| (text("", theme::FONT_SMALL, color), TextLayout::no_wrap());
    let below = |gap: f32| Node {
        width: percent(100),
        flex_direction: FlexDirection::Column,
        margin: UiRect::top(px(gap)),
        ..default()
    };
    (
        node,
        children![
            (heading_row(), children![heading("Place")]),
            (
                Title,
                text("", theme::FONT_BODY, theme::TEXT),
                TextLayout::no_wrap()
            ),
            (Subtitle, small(theme::TEXT_2)),
            (below(theme::GAP_S), children![crate::map::deeds()]),
            (Facts, below(theme::GAP_S)),
        ],
    )
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

/// Spawns one fact of the card under `parent`: its muted label on the left and its value on
/// the right, cut to fit the line.
fn fact(parent: &mut ChildSpawnerCommands, name: &str, value: &str) {
    let small = |words: String, color: Color| {
        (text(words, theme::FONT_SMALL, color), TextLayout::no_wrap())
    };
    let line = Node {
        width: percent(100),
        justify_content: JustifyContent::SpaceBetween,
        column_gap: px(theme::GAP_S),
        ..default()
    };
    let value = clip(value, LINE_CHARS.saturating_sub(name.chars().count() + 1));
    parent.spawn((
        line,
        children![
            small(name.to_string(), theme::TEXT_2),
            small(value, theme::TEXT)
        ],
    ));
}

/// Spawns one row of the card under `parent`. A heading that is not the first row keeps room
/// over it, which sets its group apart from the one above. `open` tells the More row what it
/// reads.
fn row(parent: &mut ChildSpawnerCommands, row: &Row, (first, open): (bool, bool)) {
    match row {
        Row::Heading(name) => {
            let gap = [theme::S3, 0.0][usize::from(first)];
            let node = Node {
                margin: UiRect::top(px(gap)),
                ..default()
            };
            parent.spawn((caps(name), node));
        }
        Row::Fact(name, value) => fact(parent, name, value),
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

/// The name, the subtitle and the rows of the card.
type Parts<'w, 's> = (
    Query<'w, 's, &'static mut Text, With<Title>>,
    Query<'w, 's, &'static mut Text, (With<Subtitle>, Without<Title>)>,
    Query<'w, 's, Entity, With<Facts>>,
);

/// Writes the card when the table, the cursor or the More row changed, and when the screen was
/// just opened. A table with no rows says how to pick a place.
pub(super) fn write(
    (board, universe, open): (Res<Board>, Option<Res<Universe>>, Option<Res<More>>),
    fresh: Query<(), Added<Facts>>,
    (mut titles, mut subtitles, facts): Parts,
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
    for mut text in &mut titles {
        text.set_if_neq(Text(clip(&shown.title, NAME_CHARS)));
    }
    for mut text in &mut subtitles {
        text.set_if_neq(Text(clip(&shown.subtitle, LINE_CHARS)));
    }
    for column in &facts {
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
