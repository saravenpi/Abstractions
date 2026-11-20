//! The map of the map screen and the pieces a chart is drawn with: the field, the sheet,
//! and the marks of a body, the ship, an orbit, a frame and a name.
//!
//! The marks are plain squares on a black field, like the rest of the interface. They sit on
//! a sheet at a share of its width and height, so zooming and panning move and size the
//! sheet alone and every mark keeps its size in pixels. A planet whose record is known is
//! drawn as the mark of its class.

use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

use super::arrow::{ShipArrow, arrow};
use super::chart::{GRID, mark};
use super::layout::{SMALLEST, measure};
use super::orbits::{cell, circle};
use super::rim::pin;
use crate::ui::{text, theme};
use crate::universe::PlanetSpec;

/// Width and height of the map in the units the lens counts in. The map is drawn at the size
/// the window leaves it, and its pixels are shares of that size.
pub const SIZE: f32 = 240.0;

/// Side in pixels of the frame around the ship while the pointer is on it.
const SHIP_FRAME: f32 = 28.0;

/// Pixels of the thinnest step of the arrow of the ship.
const ARROW: f32 = 3.0;

/// The name the map gives the ship.
const SHIP: &str = "Ship";

/// Marks the field the map is seen through. It knows where the pointer is over it.
#[derive(Component)]
pub struct Field;

/// Marks the sheet the chart is drawn on, which the lens moves and sizes inside the field.
#[derive(Component)]
pub struct Sheet;

/// Marks what is drawn of the charts of a system, a galaxy and the universe: the sheet, the
/// pin toward home, the letter for north and the scale. It is hidden while the map shows a planet.
#[derive(Component)]
pub struct ChartLayer;

/// The empty map: a square that knows where the pointer is, holding the chart and the pin
/// that points home, the planet and the markers of the players. `draw`
/// fills the chart. The room around the square cuts what leaves it, so a zoomed chart runs
/// on to the edges of the screen.
pub fn field() -> impl Bundle {
    let side = px(measure(SMALLEST, (0, 0)).map);
    let cover = Node {
        position_type: PositionType::Absolute,
        width: percent(100),
        height: percent(100),
        ..default()
    };
    let sheet = (Sheet, cover.clone());
    let chart = (ChartLayer, cover, children![sheet, pin()]);
    (
        Field,
        RelativeCursorPosition::default(),
        Node {
            width: side,
            height: side,
            flex_shrink: 0.0,
            ..default()
        },
        children![crate::map::planet_layer(), chart, crate::map::pin_layer()],
    )
}

/// Returns a node of `size` pixels whose top left corner is `corner` pixels away from the
/// chart point `at`.
pub fn spot(at: Vec2, corner: Vec2, size: Vec2) -> Node {
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

/// Share of its colour a frame is laid in: a faint square of tone, not a line.
const FRAME_TONE: f32 = 0.25;

/// Spawns a faint square `side` pixels wide in `color` around the chart point `at`, which
/// marks it by tone rather than by an outline.
pub fn frame(sheet: &mut ChildSpawnerCommands, at: Vec2, (side, color): (f32, Color)) {
    let node = spot(at, Vec2::splat(-0.5 * side), Vec2::splat(side));
    let tone = color.with_alpha(color.alpha() * FRAME_TONE);
    sheet.spawn((node, BackgroundColor(tone)));
}

/// Spawns `title` to the right of the chart point `at`, `gap` pixels away.
pub fn name(sheet: &mut ChildSpawnerCommands, at: Vec2, gap: f32, title: &str) {
    let node = Node {
        width: auto(),
        height: auto(),
        ..spot(at, Vec2::new(gap, -0.6 * theme::FONT_SMALL), Vec2::ZERO)
    };
    let words = text(title, theme::FONT_SMALL, theme::TEXT);
    sheet.spawn((node, words, TextLayout::no_wrap()));
}

/// Spawns a star, a galaxy or a planet at `at` in `color`: a square `side` pixels wide, or
/// the mark of the class of `world` when its record is known, on cells half as wide, or
/// wider for a gas giant.
pub fn body(
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
    let unit = cell(side, planet);
    let corner = Vec2::splat(-0.5 * unit * f32::from(GRID));
    for [x, y, w, h] in mark(planet.record.class).iter().copied() {
        let from = corner + Vec2::new(f32::from(x), f32::from(y)) * unit;
        let size = Vec2::new(f32::from(w), f32::from(h)) * unit;
        sheet.spawn((spot(at, from, size), BackgroundColor(color)));
    }
}

/// Spawns the ship at `at`, an arrow that `arrow::turn` points where the nose points, framed
/// and named while the pointer is on it.
pub fn ship(sheet: &mut ChildSpawnerCommands, at: Vec2, pointed: bool) {
    let side = 5.0 * ARROW;
    let node = Node {
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..spot(at, Vec2::splat(-0.5 * side), Vec2::splat(side))
    };
    sheet.spawn(node).with_child((ShipArrow, arrow(ARROW)));
    if pointed {
        frame(sheet, at, (SHIP_FRAME, theme::TEXT_2));
        name(sheet, at, 0.5 * SHIP_FRAME + theme::GAP_XS, SHIP);
    }
}

/// Returns the hairline of an orbit `radius` chart units around the middle of the chart. It
/// is the one round thing of the interface: an orbit is a circle.
pub fn ring(radius: f32) -> impl Bundle {
    circle(Vec2::ZERO, radius)
}
