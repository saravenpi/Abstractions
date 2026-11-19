//! The coordinates line under the compass: latitude, longitude and height on a planet, such
//! as `12.4° N  103.2° E  245 m`, and in space the system and the distance to the nearest
//! planet.

use std::fmt::Write;

use bevy::math::DVec3;
use bevy::prelude::*;

use super::minimap::Reader;
use super::units::height;
use crate::computer::distance;
use crate::space::{Clock, Frame};
use crate::ui::{text, theme};
use crate::universe::{StarId, SystemSpec, Universe};

/// Marks the text of the coordinates line.
#[derive(Component)]
pub(super) struct Coords;

/// The coordinates line, hidden until there is something to say.
pub(super) fn line() -> impl Bundle {
    (
        Coords,
        Node {
            display: Display::None,
            ..default()
        },
        text("", theme::FONT_SMALL, theme::MUTED),
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

/// Writes the system around `at` in its frame and the distance to the surface of its nearest
/// planet into `out`.
fn system(out: &mut String, spec: &SystemSpec, at: DVec3, time: f64) {
    let gap = |planet: &crate::universe::PlanetSpec| {
        (planet.position(time) - at).length() - planet.radius
    };
    let near = spec
        .planets
        .iter()
        .map(|planet| (gap(planet), planet))
        .min_by(|a, b| a.0.total_cmp(&b.0));
    let _ = write!(out, "{}", spec.star.name);
    if let Some((gap, planet)) = near {
        let _ = write!(out, "  {} {}", planet.name, distance(gap.max(0.0)));
    }
}

/// Writes the system the place `place` lies in, looked up into `known` when it changed, and
/// its nearest planet. Writes nothing between the systems.
fn space(
    out: &mut String,
    place: (Frame, DVec3),
    universe: &Universe,
    known: &mut Option<(StarId, SystemSpec)>,
    time: f64,
) {
    let (Frame::System(star), at) = place else {
        return;
    };
    if known.as_ref().is_none_or(|(id, _)| *id != star) {
        *known = Some((star, universe.system(star)));
    }
    if let Some((_, spec)) = known.as_ref() {
        system(out, spec, at, time);
    }
}

/// What the coordinates line reads besides the place of the player.
type Sources<'w> = (Option<Res<'w, Universe>>, Option<Res<'w, Clock>>);

/// Writes the coordinates of the player on a planet, or the system in space, and hides the
/// line in the space between the systems. The system is looked up once each time the player
/// enters another one.
pub(super) fn update(
    reader: Reader,
    (universe, clock): Sources,
    mut known: Local<Option<(StarId, SystemSpec)>>,
    mut words: Local<String>,
    mut lines: Query<(&mut Node, &mut Text), With<Coords>>,
) {
    words.clear();
    let time = clock.map_or(0.0, |clock| clock.seconds);
    if let Some((pose, globe)) = reader.pose().zip(reader.globe()) {
        ground(&mut words, pose.at, globe.radius());
    } else if let Some((place, universe)) = reader.place().zip(universe.as_deref()) {
        space(&mut words, place, universe, &mut known, time);
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
