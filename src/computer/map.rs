//! The map of the computer panel and the pieces a chart is drawn with: the field, the sheet,
//! and the marks of a body, the ship, an orbit, a frame and a name.
//!
//! The marks are plain squares on a black field, like the rest of the interface. They sit on
//! a sheet at a share of its width and height, so zooming and panning move and size the
//! sheet alone and every mark keeps its size in pixels. A planet whose record is known is
//! drawn as the mark of its class.

use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

use super::chart::{GRID, mark};
use super::layout::{SMALLEST, map_side};
use crate::ui::{Icon, Tone, column, icon, text, theme};
use crate::universe::PlanetSpec;

/// Width and height of the map in the units the lens counts in. The map is drawn at the size
/// the window leaves it, and its pixels are shares of that size.
pub const SIZE: f32 = 240.0;

/// Side in pixels of the frame around the ship while the pointer is on it.
const SHIP_FRAME: f32 = 28.0;

/// The name the map gives the ship.
const SHIP: &str = "Ship";

/// Marks the field the map is seen through. It knows where the pointer is over it.
#[derive(Component)]
pub struct Field;

/// Marks the sheet the chart is drawn on, which the lens moves and sizes inside the field.
#[derive(Component)]
pub struct Sheet;

/// Marks the bar whose length stands for the distance written above it.
#[derive(Component)]
pub(super) struct ScaleBar;

/// Marks the text that says which distance the scale bar stands for.
#[derive(Component)]
pub(super) struct ScaleText;

/// What lies over the chart: a letter for north, which is up, in the top right corner, and in
/// the bottom left one the scale, a distance and a bar of that length.
fn legend() -> impl Bundle {
    let bar = Node {
        height: px(2),
        ..default()
    };
    let north = Node {
        align_self: AlignSelf::FlexEnd,
        ..default()
    };
    (
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            padding: UiRect::axes(px(theme::GAP_S), px(theme::GAP_XS)),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        },
        children![
            (north, text("N", theme::FONT_SMALL, theme::MUTED)),
            (
                column(auto(), 0.0),
                children![
                    (ScaleText, text("", theme::FONT_SMALL, theme::MUTED)),
                    (ScaleBar, bar, BackgroundColor(theme::TEXT)),
                ],
            ),
        ],
    )
}

/// The empty map: a framed square that cuts what leaves it, holding the sheet under the
/// legend. `draw` fills the sheet.
pub fn field() -> impl Bundle {
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
        children![(Sheet, sheet), legend()],
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

/// Spawns a frame `side` pixels wide around the chart point `at`.
pub fn frame(sheet: &mut ChildSpawnerCommands, at: Vec2, side: f32) {
    let node = Node {
        border: UiRect::all(px(theme::LINE)),
        ..spot(at, Vec2::splat(-0.5 * side), Vec2::splat(side))
    };
    sheet.spawn((node, BorderColor::all(theme::TEXT)));
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
/// the mark of the class of `world`, on cells half as wide, when its record is known.
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
    let unit = side * 0.5;
    let corner = Vec2::splat(-0.5 * unit * f32::from(GRID));
    for [x, y, w, h] in mark(planet.record.class).iter().copied() {
        let from = corner + Vec2::new(f32::from(x), f32::from(y)) * unit;
        let size = Vec2::new(f32::from(w), f32::from(h)) * unit;
        sheet.spawn((spot(at, from, size), BackgroundColor(color)));
    }
}

/// Spawns the ship at `at`, framed and named while the pointer is on it.
pub fn ship(sheet: &mut ChildSpawnerCommands, at: Vec2, pointed: bool) {
    let node = spot(
        at,
        Vec2::splat(-0.5 * theme::ICON),
        Vec2::splat(theme::ICON),
    );
    sheet.spawn(node).with_child(icon(Icon::Send, Tone::Plain));
    if pointed {
        frame(sheet, at, SHIP_FRAME);
        name(sheet, at, 0.5 * SHIP_FRAME + theme::GAP_XS, SHIP);
    }
}

/// Returns the hairline of an orbit `radius` chart units around the middle of the chart. It
/// is the one round thing of the interface: an orbit is a circle.
pub fn ring(radius: f32) -> impl Bundle {
    let node = Node {
        position_type: PositionType::Absolute,
        left: percent((0.5 - radius * 0.5) * 100.0),
        top: percent((0.5 - radius * 0.5) * 100.0),
        width: percent(radius * 100.0),
        height: percent(radius * 100.0),
        border: UiRect::all(px(theme::LINE)),
        border_radius: BorderRadius::MAX,
        ..default()
    };
    (node, BorderColor::all(theme::HAIRLINE))
}
