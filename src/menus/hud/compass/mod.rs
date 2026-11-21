//! The compass: a thin bar at the top of the screen with a mark for every marker around the
//! player, each where it lies, as an icon with its distance on the line under it and no name:
//! names are on the map and on the marker over the world.
//!
//! A mark slides along the bar as the player turns, fades out toward the ends of the bar and
//! hides past them. A mark never leaves its bearing: where two meet, the one that ranks first is
//! drawn whole in front and the other dimmed behind it, and a distance that would print over
//! another is left out. Friends past a limit fold into the nearest one with a count. On a planet
//! an N marks north. The bar shows even while empty. The ship only shows on the planet
//! it stands on and while the player is on foot; favourites only nearby. The home mark only
//! shows on the planet of the home base, on foot or in its air, and nowhere else.

mod aim;
mod beacons;
mod herds;
mod north;
mod show;
mod space;
#[cfg(test)]
mod tests_course;
#[cfg(test)]
mod tests_herds;
#[cfg(test)]
mod tests_space;

pub(super) use aim::Around;
#[cfg(test)]
pub(super) use north::North;
pub(super) use north::point as north;
pub(super) use show::{icon, update};
pub(super) use space::BEHIND;

use std::time::Duration;

use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;

use super::MarkerSet;
use super::context::Element;
use super::fade::gate;
use super::needle::WIDTH;
use crate::menus::MarkerKind;
use crate::screens::Screen;
use crate::ship::ShipSet;
use crate::ui::{ICON_FONT, text, theme};

/// Width in pixels of the room of the north mark, which is centred on its place on the bar.
const ROOM: f32 = 24.0;

/// Height in pixels of the icon of a mark, small enough to leave the view clear.
const MARK: f32 = theme::ICON_SMALL;

/// Height in pixels of the row of icons, over the line of the bar.
const ROW: f32 = theme::ROW_HEIGHT;

/// Height in pixels of the bar: the row of icons, then one line of distances under its line.
pub(super) const HEIGHT: f32 = ROW + theme::LINE_SMALL;

/// Width in pixels of the room of one mark, centred on its place on the bar: wide enough for
/// the longest short distance under the icon, such as "12 400 km".
pub(super) const LANE: f32 = 96.0;

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

/// Marks the count of friends folded into the mark of the pool at this slot, empty for most.
#[derive(Component)]
pub(super) struct Count(usize);

/// Marks the distance under the mark of the pool at this slot.
#[derive(Component)]
pub(super) struct Far(usize);

/// The row of the mark of the pool at `slot`: its icon centred on the place of the mark, and
/// the count beside it.
fn head(slot: usize) -> impl Bundle {
    (
        Node {
            width: percent(100),
            height: px(ROW),
            column_gap: px(theme::S1),
            padding: UiRect::left(px((LANE - MARK) * 0.5)),
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
                Count(slot),
                text("", theme::FONT_SMALL, theme::TEXT),
                TextLayout::no_wrap(),
            ),
        ],
    )
}

/// One hidden mark of the pool at `slot`: its row, then its distance on the line under it.
fn pin(slot: usize) -> impl Bundle {
    (
        Pin { slot, kind: None },
        Node {
            display: Display::None,
            position_type: PositionType::Absolute,
            width: px(LANE),
            height: px(HEIGHT),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            ..default()
        },
        children![
            head(slot),
            (
                Far(slot),
                text("", theme::FONT_SMALL, theme::TEXT_2),
                TextLayout::no_wrap(),
            ),
        ],
    )
}

/// The line of the bar, or the tick at its middle: a rule `size` pixels wide and high, `left`
/// percent along the bar, standing on the bottom of the row of icons.
fn rule(left: f32, size: Vec2, color: Color) -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            left: percent(left),
            top: px(ROW - size.y),
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
        gate(Element::Compass),
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            top: px(theme::EDGE),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            ..default()
        },
        Pickable::IGNORE,
        children![(
            Bar,
            Visibility::Hidden,
            Node {
                width: px(WIDTH),
                height: px(HEIGHT),
                ..default()
            },
            BackgroundColor(theme::PANEL),
            children![
                rule(0.0, Vec2::new(WIDTH, theme::LINE), theme::TEXT_2),
                rule(50.0, Vec2::new(theme::LINE, theme::S2), theme::TEXT),
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
        (
            (beacons::home, beacons::ship).after(ShipSet::Place),
            herds::herds.run_if(on_timer(Duration::from_secs_f32(herds::EVERY))),
            beacons::course.after(super::eye::capture),
        )
            .in_set(MarkerSet::Fill)
            .run_if(in_state(Screen::Gameplay)),
    );
}
