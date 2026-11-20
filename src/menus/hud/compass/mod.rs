//! The compass: a thin bar at the top of the screen with a mark for every marker around the
//! player, each where it lies, with its name and distance under it.
//!
//! A mark slides along the bar as the player turns, fades out toward the ends of the bar and
//! hides past them. Every mark sits on one level, its icon over the line and its distance under
//! it, even where marks overlap. On a planet an N marks north. The bar shows even
//! while empty. The ship only shows on the planet it stands on and while the player is on foot;
//! favourites only nearby. The home mark only shows on the planet of the home base, on foot or in
//! its air, and nowhere else.

mod aim;
mod beacons;
mod herds;
mod north;
mod show;
#[cfg(test)]
mod tests_herds;

#[cfg(test)]
pub(super) use north::North;
pub(super) use north::point as north;
pub(super) use show::update;

use bevy::prelude::*;

use super::MarkerSet;
use super::needle::WIDTH;
use crate::menus::MarkerKind;
use crate::screens::Screen;
use crate::ship::ShipSet;
use crate::ui::{ICON_FONT, text, theme};

/// Width in pixels of the room of one mark, which is centred on its place on the bar.
const ROOM: f32 = 96.0;

/// Height in pixels of the icon of a mark, small enough to leave the view clear.
const MARK: f32 = 14.0;

/// Height in pixels of the bar with one row of marks.
const HEIGHT: f32 = MARK * 2.0 + theme::GAP_M;

/// Marks the bar itself, which shows whenever the HUD does, empty or not.
#[derive(Component)]
pub(super) struct Bar;

/// One mark of the pool: the slot it shows among the marks on the bar this frame, and the
/// kind of the marker it shows, if any.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub(super) struct Pin {
    /// Its place in the pool.
    pub slot: usize,
    /// What the marker it shows stands for, or nothing while hidden.
    pub kind: Option<MarkerKind>,
}

/// Marks the icon of the mark of the pool at this slot.
#[derive(Component)]
pub(super) struct Glyph(usize);

/// Marks the label of the mark of the pool at this slot: its name and distance.
#[derive(Component)]
pub(super) struct Far(usize);

/// One hidden mark of the pool at `slot`: its icon over its label.
fn pin(slot: usize) -> impl Bundle {
    (
        Pin { slot, kind: None },
        Node {
            display: Display::None,
            position_type: PositionType::Absolute,
            width: px(ROOM),
            top: px(0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            ..default()
        },
        children![
            (
                Glyph(slot),
                Text::default(),
                TextFont::from_font_size(MARK).with_font(ICON_FONT),
                TextLayout::no_wrap(),
                TextColor(Color::WHITE),
            ),
            (
                Far(slot),
                text("", theme::FONT_SMALL, theme::TEXT_2),
                TextLayout::no_wrap(),
            ),
        ],
    )
}

/// The line of the bar, or the tick at its middle: a rule `size` pixels wide and high, `left`
/// percent along the bar.
fn rule(left: f32, size: Vec2, color: Color) -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            left: percent(left),
            top: px(MARK + theme::GAP_XS),
            width: px(size.x),
            height: px(size.y),
            ..default()
        },
        BackgroundColor(color),
    )
}

/// The compass, high in the middle of the screen.
pub(super) fn bar() -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            top: px(theme::GAP_M),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            ..default()
        },
        children![(
            Bar,
            Visibility::Hidden,
            Node {
                width: px(WIDTH),
                height: px(HEIGHT),
                ..default()
            },
            children![
                rule(0.0, Vec2::new(WIDTH, theme::LINE), theme::HAIRLINE),
                rule(50.0, Vec2::new(theme::LINE, theme::GAP_S), theme::TEXT_2),
                north::mark(),
                pin(0),
                pin(1),
            ],
        ),],
    )
}

/// Registers the home, ship and herd markers, written once the ship has placed the world.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (beacons::home, beacons::ship, herds::herds)
            .after(ShipSet::Place)
            .in_set(MarkerSet::Fill)
            .run_if(in_state(Screen::Gameplay)),
    );
}
