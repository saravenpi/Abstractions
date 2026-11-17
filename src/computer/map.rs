//! The map of the computer screen: the chart of the open tab drawn as dots beside the table.
//!
//! The dots are plain squares on a black field, like the rest of the interface. The row under
//! the cursor is drawn larger with its name, and the destination wears a frame.

use bevy::prelude::*;

use super::board::Board;
use super::chart::{Dot, chart};
use super::route::Destination;
use crate::ui::{Icon, Tone, icon, text, theme};

/// Width and height of the map in pixels.
pub(super) const SIZE: f32 = 300.0;

/// Sides in pixels of an orbit dot, a body, the body under the cursor and the frame of the destination.
const SIDES: [f32; 4] = [1.0, 4.0, 8.0, 16.0];

/// Marks the field the dots are drawn in.
#[derive(Component)]
pub(super) struct Field;

/// The empty map: a framed square. `draw` fills it.
pub(super) fn field() -> impl Bundle {
    (
        Field,
        Node {
            width: px(SIZE),
            height: px(SIZE),
            flex_shrink: 0.0,
            border: UiRect::all(px(theme::LINE)),
            overflow: Overflow::clip(),
            ..default()
        },
        BorderColor::all(theme::HAIRLINE),
        BackgroundColor(theme::STRIP),
    )
}

/// Returns a node `side` pixels wide and high, centred on the chart point `at`.
fn square(at: Vec2, side: f32) -> Node {
    let middle = (Vec2::new(at.x, -at.y) * 0.5 + 0.5) * SIZE;
    Node {
        position_type: PositionType::Absolute,
        left: px(middle.x - side * 0.5),
        top: px(middle.y - side * 0.5),
        width: px(side),
        height: px(side),
        ..default()
    }
}

/// Spawns the frame around the destination at `at`.
fn frame(field: &mut ChildSpawnerCommands, at: Vec2) {
    let node = Node {
        border: UiRect::all(px(theme::LINE)),
        ..square(at, SIDES[3])
    };
    field.spawn((node, BorderColor::all(theme::TEXT)));
}

/// Spawns the name of the row under the cursor beside its dot at `at`.
fn name(field: &mut ChildSpawnerCommands, at: Vec2, title: &str) {
    let node = Node {
        width: auto(),
        height: auto(),
        margin: UiRect::left(px(SIDES[3])),
        ..square(at, SIDES[3])
    };
    field.spawn((node, text(title, theme::FONT_SMALL, theme::TEXT)));
}

/// Spawns one dot of the chart, with its frame and its name when it has them.
fn dot(
    field: &mut ChildSpawnerCommands,
    (dot, at): (Dot, Vec2),
    board: &Board,
    chosen: &Destination,
) {
    let Dot::Row(index) = dot else {
        if dot == Dot::Ship {
            let node = square(at, theme::ICON);
            field.spawn(node).with_child(icon(Icon::Send, Tone::Plain));
        } else {
            field.spawn((square(at, SIDES[0]), BackgroundColor(theme::FAINT)));
        }
        return;
    };
    let Some(line) = board.lines.get(index) else {
        return;
    };
    let under = index == board.cursor;
    let side = SIDES[1 + usize::from(under)];
    let color = if line.visited || under {
        theme::TEXT
    } else {
        theme::MUTED
    };
    field.spawn((square(at, side), BackgroundColor(color)));
    if chosen.0 == Some(line.target) {
        frame(field, at);
    }
    if under {
        name(field, at, &line.title);
    }
}

/// Draws the chart of the open tab again whenever the table, the cursor or the destination
/// changed, and when the screen was just opened.
pub(super) fn draw(
    mut commands: Commands,
    (board, chosen): (Res<Board>, Res<Destination>),
    fresh: Query<(), Added<Field>>,
    fields: Query<Entity, With<Field>>,
) {
    if !(board.is_changed() || chosen.is_changed() || !fresh.is_empty()) {
        return;
    }
    let dots = chart(board.tab, &board.lines, board.ship);
    for field in &fields {
        commands
            .entity(field)
            .despawn_related::<Children>()
            .with_children(|field| {
                for placed in &dots {
                    dot(field, *placed, &board, &chosen);
                }
            });
    }
}
