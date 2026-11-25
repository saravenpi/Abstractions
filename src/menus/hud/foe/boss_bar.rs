//! Layout builders for the framed top-centre boss health bar.

use bevy::prelude::*;

use super::boss::{BossFill, BossFrame, BossPhase, BossTitle, BossTrail, BossWarn};
use crate::ui::{text, theme};

/// Width in logical pixels of the framed boss health bar track.
const WIDTH: f32 = 480.0;

/// Gold/crimson border accent framing the boss bar.
const BORDER: Color = Color::srgba(0.84, 0.56, 0.22, 0.92);

/// Vivid crimson/ember current health fill.
const CRIMSON: Color = Color::srgba(0.86, 0.16, 0.12, 1.0);

/// Smooth white/gold trailing damage fill.
const TRAIL: Color = Color::srgba(0.98, 0.88, 0.62, 0.92);

/// Glowing ember tint for the Phase II badge and special warning callout.
const EMBER: Color = Color::srgba(1.0, 0.78, 0.28, 1.0);

/// Returns the top-centre framed `480 px` classic boss health bar bundle, initially hidden.
pub(super) fn bar() -> impl Bundle {
    let frame = Node {
        display: Display::None,
        position_type: PositionType::Absolute,
        top: px(54.0),
        left: percent(50),
        margin: UiRect::left(px(-(WIDTH * 0.5 + 14.0))),
        padding: UiRect::axes(px(12.0), px(8.0)),
        border: UiRect::all(px(2.0)),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        row_gap: px(theme::S1),
        ..default()
    };
    let warn = Node {
        display: Display::None,
        margin: UiRect::top(px(2.0)),
        ..default()
    };
    let root = (BossFrame, frame, BorderColor::all(BORDER), Pickable::IGNORE);
    let callout = (
        BossWarn,
        warn,
        text("", theme::FONT_SMALL, EMBER),
        TextLayout::no_wrap(),
    );
    (root, children![header(), track(), callout])
}

/// Returns the boss title row and glowing `PHASE II` badge.
fn header() -> impl Bundle {
    let head = Node {
        width: px(WIDTH),
        justify_content: JustifyContent::SpaceBetween,
        align_items: AlignItems::Center,
        ..default()
    };
    let badge = Node {
        display: Display::None,
        padding: UiRect::axes(px(6.0), px(2.0)),
        border: UiRect::all(px(1.0)),
        ..default()
    };
    let title = (
        BossTitle,
        text("", theme::FONT_SMALL, theme::TEXT),
        TextLayout::no_wrap(),
    );
    let phase = (BossPhase, badge, BorderColor::all(EMBER));
    let tag = (
        text("PHASE II", theme::FONT_SMALL, EMBER),
        TextLayout::no_wrap(),
    );
    (head, children![title, (phase, children![tag])])
}

/// Returns the multi-layer 480 px health bar track: dark track, trailing bar, crimson HP fill and
/// vertical 50% phase tick.
fn track() -> impl Bundle {
    let bar = Node {
        width: px(WIDTH),
        height: px(14.0),
        position_type: PositionType::Relative,
        overflow: Overflow::clip(),
        ..default()
    };
    let layer = || Node {
        position_type: PositionType::Absolute,
        left: px(0.0),
        top: px(0.0),
        height: percent(100),
        width: percent(100),
        ..default()
    };
    let tick = Node {
        position_type: PositionType::Absolute,
        left: percent(50),
        top: px(0.0),
        width: px(2.0),
        height: percent(100),
        ..default()
    };
    let bg = BackgroundColor(Color::srgba(0.12, 0.08, 0.09, 0.95));
    (
        (bar, bg),
        children![
            (BossTrail, layer(), BackgroundColor(TRAIL)),
            (BossFill, layer(), BackgroundColor(CRIMSON)),
            (tick, BackgroundColor(BORDER))
        ],
    )
}
