//! The minimap as nodes: a framed square in the top right corner over the blurred game, the
//! image of the ground, the sheet of the system chart, and the marks over them.
//!
//! Every mark is a white glyph in a black box, like the structures painted on the ground, so
//! it reads on bright land as well as on dark water.

use bevy::prelude::*;
use bevy::ui::widget::NodeImageMode;

use crate::computer::spot;
use crate::ui::{Blurred, Icon, Tone, icon_sized, text, theme};

/// Side of the map in pixels in a window 1080 pixels high. It grows and shrinks with the
/// height of the window.
pub const SIDE: f32 = 160.0;

/// Smallest and largest side of the map in pixels, whatever the window.
const LIMITS: (f32, f32) = (128.0, 240.0);

/// Size in pixels of the icon of a mark: half the grid the icons are drawn on.
pub const MARK: f32 = 12.0;

/// Side in pixels of the black box behind a mark.
const BOX: f32 = 14.0;

/// One part of the minimap that is shown, hidden or moved every frame.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Part {
    /// The framed square itself, hidden while the minimap is off.
    Frame,
    /// The image of the ground.
    Ground,
    /// The player in the middle, an arrow that always points up.
    Arrow,
    /// The mark of the home base.
    Home,
    /// The mark of the ship.
    Ship,
    /// The letter that says where north is.
    North,
}

/// Marks the sheet the system chart is drawn on.
#[derive(Component)]
pub struct Chart;

/// Marks the text in the corner: the range on a planet, the name of the star in space.
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

/// A hidden mark: `content` in the middle of a black box, placed later.
fn mark(part: Part, content: impl Bundle) -> impl Bundle {
    let node = Node {
        display: Display::None,
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..spot(Vec2::ZERO, Vec2::splat(-0.5 * BOX), Vec2::splat(BOX))
    };
    (part, node, BackgroundColor(theme::INK), children![content])
}

/// The shape of the player: three bars that widen downward, a triangle drawn with squares.
fn arrow() -> impl Bundle {
    let bar = |width: f32| {
        let node = Node {
            width: px(width),
            height: px(3),
            ..default()
        };
        (node, BackgroundColor(theme::TEXT))
    };
    let node = Node {
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        ..default()
    };
    (node, children![bar(2.0), bar(6.0), bar(10.0)])
}

/// The text in the bottom left corner, on a dark strip.
fn caption() -> impl Bundle {
    let node = Node {
        position_type: PositionType::Absolute,
        left: px(0),
        bottom: px(0),
        padding: UiRect::axes(px(theme::GAP_XS), px(0)),
        ..default()
    };
    let words = text("", theme::FONT_SMALL, theme::MUTED);
    let line = (Caption, words, TextLayout::no_wrap());
    (node, BackgroundColor(theme::STRIP), children![line])
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

/// The square of the minimap: in the top right corner, as far from the edges as the compass
/// is from the top, and cutting what leaves it.
fn square() -> Node {
    let side = Val::Vh(100.0 * SIDE / 1080.0);
    Node {
        position_type: PositionType::Absolute,
        top: px(theme::GAP_M),
        right: px(theme::GAP_M),
        width: side,
        height: side,
        min_width: px(LIMITS.0),
        min_height: px(LIMITS.0),
        max_width: px(LIMITS.1),
        max_height: px(LIMITS.1),
        border: UiRect::all(px(theme::LINE)),
        overflow: Overflow::clip(),
        ..default()
    }
}

/// The minimap: a hairline frame over the blurred, darkened game, holding the ground, the
/// chart and the marks, all hidden until there is something to show.
pub fn frame() -> impl Bundle {
    let symbol = |symbol: Icon| icon_sized(symbol, Tone::Plain, MARK);
    let north = text("N", theme::FONT_SMALL, theme::TEXT);
    (
        Part::Frame,
        square(),
        BorderColor::all(theme::HAIRLINE),
        children![
            (Blurred, layer(), BackgroundColor(theme::SCRIM)),
            ground(),
            (Chart, layer()),
            mark(Part::Home, symbol(Icon::Home)),
            mark(Part::Ship, symbol(Icon::Send)),
            mark(Part::North, north),
            mark(Part::Arrow, arrow()),
            caption(),
        ],
    )
}
