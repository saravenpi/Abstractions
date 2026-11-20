//! The coordinates line in the top left corner: latitude, longitude and height on a planet,
//! such as `12.4° N  103.2° E  245 m`, on a strip. It shows only on or near a ground.

use std::fmt::Write;

use bevy::math::DVec3;
use bevy::prelude::*;

use super::context::Element;
use super::fade::gate;
use super::minimap::Reader;
use super::units::height;
use crate::ui::{text, theme};

/// Marks the text of the coordinates line.
#[derive(Component)]
pub(super) struct Coords;

/// The top-left corner of the screen: the coordinates line over the climate line, as far from
/// the edges as the minimap is in the top-right corner. Each line has its own strip.
pub(super) fn corner() -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            top: px(theme::EDGE),
            left: px(theme::EDGE),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexStart,
            row_gap: px(theme::S1),
            ..default()
        },
        Pickable::IGNORE,
        children![line(), super::climate::line()],
    )
}

/// The coordinates line, hidden until there is something to say.
fn line() -> impl Bundle {
    (
        Coords,
        gate(Element::Coords),
        Node {
            display: Display::None,
            padding: UiRect::axes(px(theme::S3), px(theme::S2)),
            ..default()
        },
        BackgroundColor(theme::PANEL),
        text("", theme::FONT_SMALL, theme::TEXT),
        TextLayout::no_wrap(),
    )
}

/// Returns the latitude and the longitude in degrees of the planet-fixed direction `dir`,
/// north and east positive: the inverse of `computer::season::direction`.
pub(super) fn latlon(dir: DVec3) -> (f64, f64) {
    let dir = dir.normalize();
    let lat = dir.y.clamp(-1.0, 1.0).asin();
    let lon = (-dir.z).atan2(dir.x);
    (lat.to_degrees(), lon.to_degrees())
}

/// Writes the place `at`, in metres from the centre of a planet of `radius` metres, into
/// `out` as latitude, longitude and height above the sea.
pub(super) fn ground(out: &mut String, at: DVec3, radius: f64) {
    let (lat, lon) = latlon(at);
    let north = if lat < 0.0 { 'S' } else { 'N' };
    let east = if lon < 0.0 { 'W' } else { 'E' };
    let rise = height(at.length() - radius);
    let _ = write!(
        out,
        "{:.1}° {north}  {:.1}° {east}  {rise}",
        lat.abs(),
        lon.abs()
    );
}

/// Writes the coordinates of the player on a planet and hides the line off one. Which contexts
/// show the line is `context::shown`.
pub(super) fn update(
    reader: Reader,
    mut words: Local<String>,
    mut lines: Query<(&mut Node, &mut Text), With<Coords>>,
) {
    words.clear();
    if let Some((pose, globe)) = reader.pose().zip(reader.globe()) {
        ground(&mut words, pose.at, globe.radius());
    }
    let display = [Display::None, Display::Flex][usize::from(!words.is_empty())];
    for (mut node, mut text) in &mut lines {
        if node.display != display {
            node.display = display;
        }
        if text.0 != *words {
            text.0.clone_from(&words);
        }
    }
}
