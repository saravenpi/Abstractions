//! The card of the side panel: the name of the selected search result, what it is, the
//! buttons of what can be done with it, and its facts in groups, a muted label on the left of
//! each fact and its value on the right.

use bevy::prelude::*;

use super::board::Board;
use super::detail::{Card, Row, card as of};
use super::format::clip;
use super::layout::{LINE_CHARS, NAME_CHARS};
use crate::ui::{caps, text, theme};
use crate::universe::Universe;

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
        row_gap: px(theme::GAP_XS),
        min_height: px(0),
        overflow: Overflow::clip(),
        ..default()
    };
    let small = |color: Color| (text("", theme::FONT_SMALL, color), TextLayout::no_wrap());
    let facts = Node {
        width: percent(100),
        flex_direction: FlexDirection::Column,
        ..default()
    };
    (
        node,
        children![
            (
                Title,
                text("", theme::FONT_BODY, theme::TEXT),
                TextLayout::no_wrap()
            ),
            (Subtitle, small(theme::MUTED)),
            crate::map::deeds(),
            (Facts, facts),
        ],
    )
}

/// Returns the card of the row under the cursor, or an empty card for an empty table.
pub(super) fn shown(board: &Board, universe: &Universe) -> Card {
    let world = board.worlds.get(board.cursor).and_then(Option::as_ref);
    let line = board.lines.get(board.cursor);
    line.map_or_else(Card::default, |line| of(line, world, universe))
}

/// Spawns one row of the card under `parent`.
fn row(parent: &mut ChildSpawnerCommands, row: &Row) {
    let small = |words: String, color: Color| {
        (text(words, theme::FONT_SMALL, color), TextLayout::no_wrap())
    };
    match row {
        Row::Heading(name) => {
            parent.spawn((caps(name), Node::default()));
        }
        Row::Fact(name, value) => {
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
                    small(name.to_string(), theme::MUTED),
                    small(value, theme::TEXT)
                ],
            ));
        }
        Row::Note(words) => {
            parent.spawn(text(*words, theme::FONT_SMALL, theme::MUTED));
        }
    }
}

/// The name, the subtitle and the rows of the card.
type Parts<'w, 's> = (
    Query<'w, 's, &'static mut Text, With<Title>>,
    Query<'w, 's, &'static mut Text, (With<Subtitle>, Without<Title>)>,
    Query<'w, 's, Entity, With<Facts>>,
);

/// Writes the card when the table or the cursor changed, and when the screen was just opened.
pub(super) fn write(
    (board, universe): (Res<Board>, Option<Res<Universe>>),
    fresh: Query<(), Added<Facts>>,
    (mut titles, mut subtitles, facts): Parts,
    mut commands: Commands,
) {
    if !(board.is_changed() || !fresh.is_empty()) {
        return;
    }
    let shown = universe.map_or_else(Card::default, |universe| shown(&board, &universe));
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
                for fact in &shown.rows {
                    row(parent, fact);
                }
            });
    }
}
