//! The minimap as nodes: a square in the top right corner over the blurred game, the image of
//! the ground, the sheet of the system chart, and the marks over them.
//!
//! The square is a picture: every mark on it is a white glyph with no box behind it and no
//! shadow, which would read as dirt on the map. The caption under it has no strip and carries
//! the HUD shadow. The ground is painted without structures.

use bevy::prelude::*;
use bevy::ui::widget::NodeImageMode;

use crate::computer::{arrow, spot};
use crate::menus::hud::context::Element;
use crate::menus::hud::fade::gate;
use crate::ui::{Blurred, Icon, Tone, Unshaded, icon_sized, text, theme};

/// Side of the map in pixels in a window 1080 pixels high. It grows and shrinks with the
/// height of the window.
pub const SIDE: f32 = 160.0;

/// Smallest and largest side of the map in pixels, whatever the window.
pub const LIMITS: (f32, f32) = (128.0, 240.0);

/// Height in pixels of the caption under the square and the gap above it.
pub const CAPTION: f32 = theme::S1 + theme::LINE_SMALL;

/// Size in pixels of the icon of a mark.
pub const MARK: f32 = theme::ICON_SMALL;

/// One part of the minimap that is shown, hidden or moved every frame.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Part {
    /// The square and its caption, hidden while the minimap is off.
    Frame,
    /// The image of the ground.
    Ground,
    /// The player in the middle, an arrow that points where the body or the ship faces.
    Arrow,
    /// The mark of the home base.
    Home,
    /// The mark of the ship.
    Ship,
    /// The letter that says where north is.
    North,
}

/// Marks an arrow that turns to where the player faces: the one in the middle of the ground
/// view and the ship on the system chart.
#[derive(Component)]
pub struct Needle;

/// Pixels of the thinnest step of an arrow.
pub const STEP: f32 = 2.0;

/// Marks the sheet the system chart is drawn on.
#[derive(Component)]
pub struct Chart;

/// Marks the caption under the square: the range on a planet, the name of the star in space.
#[derive(Component)]
pub struct Caption;

/// Returns a node that covers the whole map.
fn layer() -> Node {
    Node {
        position_type: PositionType::Absolute,
        width: percent(100),
        height: percent(100),
        ..default()
    }
}

/// A hidden mark: `content` in the middle of its room, placed later.
fn mark(part: Part, content: impl Bundle) -> impl Bundle {
    let node = Node {
        display: Display::None,
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..spot(Vec2::ZERO, Vec2::splat(-0.5 * MARK), Vec2::splat(MARK))
    };
    (part, node, children![content])
}

/// The caption under the square, its right edge on the edge margin, with no strip.
fn caption() -> impl Bundle {
    let node = Node {
        margin: UiRect::top(px(theme::S1)),
        ..default()
    };
    (
        Caption,
        node,
        text("", theme::FONT_SMALL, theme::TEXT),
        TextLayout::no_wrap(),
    )
}

/// The image of the ground, hidden until a patch is painted.
fn ground() -> impl Bundle {
    let image = ImageNode {
        image_mode: NodeImageMode::Stretch,
        ..default()
    };
    let node = Node {
        display: Display::None,
        ..layer()
    };
    (Part::Ground, image, UiTransform::default(), node)
}

/// The square of the minimap, cutting what leaves it.
fn square() -> Node {
    let side = Val::Vh(100.0 * SIDE / 1080.0);
    Node {
        width: side,
        height: side,
        min_width: px(LIMITS.0),
        min_height: px(LIMITS.0),
        max_width: px(LIMITS.1),
        max_height: px(LIMITS.1),
        overflow: Overflow::clip(),
        ..default()
    }
}

/// The picture of the minimap: a square over the blurred, darkened game, holding the ground,
/// the chart and the marks, none of which casts a shadow.
fn picture() -> impl Bundle {
    let symbol = |symbol: Icon| icon_sized(symbol, Tone::Plain, MARK);
    let north = text("N", theme::FONT_SMALL, theme::TEXT);
    (
        Unshaded,
        square(),
        children![
            (Blurred, layer(), BackgroundColor(theme::SCRIM)),
            ground(),
            (Chart, layer()),
            super::dots::layer(),
            mark(Part::Home, symbol(Icon::Home)),
            mark(Part::Ship, symbol(Icon::ShipParked)),
            mark(Part::North, north),
            mark(Part::Arrow, (Needle, arrow(STEP))),
        ],
    )
}

/// The minimap in the top right corner, as far from the edges as the compass is from the top:
/// the picture, then its caption under it, all hidden until there is something to show.
pub fn frame() -> impl Bundle {
    (
        Part::Frame,
        gate(Element::Minimap),
        Node {
            position_type: PositionType::Absolute,
            top: px(theme::EDGE),
            right: px(theme::EDGE),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexEnd,
            ..default()
        },
        children![picture(), caption()],
    )
}
