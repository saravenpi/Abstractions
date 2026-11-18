//! The map of the computer screen: the chart of the open tab drawn as dots beside the table.
//!
//! The dots are plain squares on a black field, like the rest of the interface. The row under
//! the cursor or under the pointer is drawn larger with its name, and the destination wears a
//! frame. A planet whose record is known is drawn as the mark of its class.
//!
//! The dots sit on a sheet at a share of its width and height, so zooming and panning move
//! and size the sheet alone and every dot keeps its size in pixels.

use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

use super::board::Board;
use super::chart::{Dot, GRID, mark};
use super::layout::{SMALLEST, map_side};
use super::pointer::MapView;
use super::route::Destination;
use crate::ui::{Icon, Tone, icon, text, theme};
use crate::universe::PlanetSpec;

/// Width and height of the map in the units the lens counts in. The map is drawn at the size
/// the window leaves it, and its pixels are shares of that size.
pub const SIZE: f32 = 240.0;

/// Sides in pixels of an orbit dot, a body, the body under the cursor and the frame of the destination.
const SIDES: [f32; 4] = [1.0, 4.0, 8.0, 16.0];

/// Side in pixels of the frame around the ship while the pointer is on it.
const SHIP_FRAME: f32 = 28.0;

/// The name the map gives the ship.
pub(super) const SHIP: &str = "Ship";

/// Marks the field the map is seen through. It knows where the pointer is over it.
#[derive(Component)]
pub(super) struct Field;

/// Marks the sheet the dots are drawn on, which the lens moves and sizes inside the field.
#[derive(Component)]
pub(super) struct Sheet;

/// The empty map: a framed square that cuts what leaves it, holding the sheet. `draw` fills
/// the sheet.
pub(super) fn field() -> impl Bundle {
    let sheet = Node {
        position_type: PositionType::Absolute,
        width: percent(100),
        height: percent(100),
        ..default()
    };
    let side = px(map_side(SMALLEST));
    (
        Field,
        RelativeCursorPosition::default(),
        Node {
            width: side,
            height: side,
            flex_shrink: 0.0,
            align_self: AlignSelf::Center,
            border: UiRect::all(px(theme::LINE)),
            overflow: Overflow::clip(),
            ..default()
        },
        BorderColor::all(theme::HAIRLINE),
        BackgroundColor(theme::STRIP),
        children![(Sheet, sheet)],
    )
}

/// Returns a node of `size` pixels whose top left corner is `corner` pixels away from the
/// chart point `at`.
fn spot(at: Vec2, corner: Vec2, size: Vec2) -> Node {
    let share = (Vec2::new(at.x, -at.y) * 0.5 + 0.5) * 100.0;
    Node {
        position_type: PositionType::Absolute,
        left: percent(share.x),
        top: percent(share.y),
        margin: UiRect {
            left: px(corner.x),
            top: px(corner.y),
            ..default()
        },
        width: px(size.x),
        height: px(size.y),
        ..default()
    }
}

/// Spawns a frame `side` pixels wide around the chart point `at`.
fn frame(sheet: &mut ChildSpawnerCommands, at: Vec2, side: f32) {
    let node = Node {
        border: UiRect::all(px(theme::LINE)),
        ..spot(at, Vec2::splat(-0.5 * side), Vec2::splat(side))
    };
    sheet.spawn((node, BorderColor::all(theme::TEXT)));
}

/// Spawns `title` to the right of the chart point `at`, `gap` pixels away.
fn name(sheet: &mut ChildSpawnerCommands, at: Vec2, gap: f32, title: &str) {
    let node = Node {
        width: auto(),
        height: auto(),
        ..spot(at, Vec2::new(gap, -0.5 * SIDES[3]), Vec2::ZERO)
    };
    sheet.spawn((node, text(title, theme::FONT_SMALL, theme::TEXT)));
}

/// Spawns a star, a galaxy or a planet at `at` in `color`: a square `side` pixels wide, or
/// the mark of the class of `world`, on cells half as wide, when its record is known.
fn body(
    sheet: &mut ChildSpawnerCommands,
    at: Vec2,
    (side, color): (f32, Color),
    world: Option<&PlanetSpec>,
) {
    let Some(planet) = world else {
        let node = spot(at, Vec2::splat(-0.5 * side), Vec2::splat(side));
        sheet.spawn((node, BackgroundColor(color)));
        return;
    };
    let unit = side * 0.5;
    let corner = Vec2::splat(-0.5 * unit * f32::from(GRID));
    for [x, y, w, h] in mark(planet.record.class).iter().copied() {
        let from = corner + Vec2::new(f32::from(x), f32::from(y)) * unit;
        let size = Vec2::new(f32::from(w), f32::from(h)) * unit;
        sheet.spawn((spot(at, from, size), BackgroundColor(color)));
    }
}

/// Spawns a dot that is not a row: a point of an orbit, or the ship, which wears a frame and
/// its name while the pointer is on it.
fn other(sheet: &mut ChildSpawnerCommands, dot: Dot, at: Vec2, pointed: bool) {
    if dot != Dot::Ship {
        let node = spot(at, Vec2::splat(-0.5 * SIDES[0]), Vec2::splat(SIDES[0]));
        sheet.spawn((node, BackgroundColor(theme::FAINT)));
        return;
    }
    let node = spot(
        at,
        Vec2::splat(-0.5 * theme::ICON),
        Vec2::splat(theme::ICON),
    );
    sheet.spawn(node).with_child(icon(Icon::Send, Tone::Plain));
    if pointed {
        frame(sheet, at, SHIP_FRAME);
        name(sheet, at, SHIP_FRAME * 0.5 + theme::GAP_XS, SHIP);
    }
}

/// Spawns one dot of the chart, with its frame and its name when it has them.
fn dot(
    sheet: &mut ChildSpawnerCommands,
    (dot, at): (Dot, Vec2),
    (board, view): (&Board, &MapView),
    chosen: &Destination,
) {
    let pointed = view.hover == Some(dot);
    let Dot::Row(index) = dot else {
        other(sheet, dot, at, pointed);
        return;
    };
    let Some(line) = board.lines.get(index) else {
        return;
    };
    let under = pointed || index == board.cursor;
    let side = SIDES[1 + usize::from(under)];
    let color = match line.visited || under {
        true => theme::TEXT,
        false => theme::MUTED,
    };
    let world = board.worlds.get(index).and_then(Option::as_ref);
    body(sheet, at, (side, color), world);
    if chosen.0 == Some(line.target) {
        frame(sheet, at, SIDES[3]);
    }
    if under {
        name(sheet, at, SIDES[3] * 0.5, &line.title);
    }
}

/// Draws the chart of the open tab again whenever the table, the cursor, the destination or
/// the mark under the pointer changed, and when the screen was just opened.
pub(super) fn draw(
    mut commands: Commands,
    (board, chosen, view): (Res<Board>, Res<Destination>, Res<MapView>),
    fresh: Query<(), Added<Sheet>>,
    sheets: Query<Entity, With<Sheet>>,
    mut pointed: Local<Option<Dot>>,
) {
    let moved = *pointed != view.hover;
    if !(board.is_changed() || chosen.is_changed() || moved || !fresh.is_empty()) {
        return;
    }
    *pointed = view.hover;
    for sheet in &sheets {
        let mut sheet = commands.entity(sheet);
        sheet.despawn_related::<Children>().with_children(|sheet| {
            for placed in &view.dots {
                dot(sheet, *placed, (&board, &view), &chosen);
            }
        });
    }
}
