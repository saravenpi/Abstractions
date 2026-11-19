//! The words of the planet level over the map: the caption, which gives the climate of the
//! planet shown, and the latitude and longitude under the pointer.

use bevy::prelude::*;

use super::geo::coords as words;
use super::level::Level;
use super::surface::Surface;
use crate::computer::climate_line;
use crate::space::Clock;
use crate::ui::{text, theme};
use crate::universe::Universe;

/// Marks the line that names the level shown.
#[derive(Component)]
pub struct Caption;

/// Marks the line that says the latitude and longitude under the pointer.
#[derive(Component)]
pub struct Coords;

/// The line that names the level shown.
pub fn caption() -> impl Bundle {
    (
        Caption,
        text("", theme::FONT_SMALL, theme::MUTED),
        TextLayout::no_wrap(),
    )
}

/// The line that says the latitude and longitude under the pointer.
pub fn coords() -> impl Bundle {
    (
        Coords,
        text("", theme::FONT_SMALL, theme::MUTED),
        TextLayout::no_wrap(),
    )
}

/// Returns the caption of the level: the climate at `time` seconds of the planet shown, which
/// the breadcrumb names, and nothing on a chart, whose list already names it.
pub fn title(level: Level, (universe, time): (Option<&Universe>, f64)) -> String {
    match (level, universe) {
        (Level::Planet(body), Some(universe)) => {
            let planet = universe.planet(body);
            climate_line(&planet, crate::climate::season(&planet, time))
        }
        _ => String::new(),
    }
}

/// Writes the caption when the level changes or the screen was just spawned.
pub(super) fn caption_text(
    level: Res<Level>,
    (universe, clock): (Option<Res<Universe>>, Option<Res<Clock>>),
    fresh: Query<(), Added<Caption>>,
    mut texts: Query<&mut Text, With<Caption>>,
) {
    if !(level.is_changed() || !fresh.is_empty()) {
        return;
    }
    let time = clock.map_or(0.0, |clock| clock.seconds);
    let said = title(*level, (universe.as_deref(), time));
    for mut text in &mut texts {
        text.set_if_neq(Text(said.clone()));
    }
}

/// Writes the latitude and longitude under the pointer on a planet, and nothing elsewhere.
pub(super) fn coords_text(
    surface: Res<Surface>,
    level: Res<Level>,
    mut texts: Query<&mut Text, With<Coords>>,
) {
    if !(surface.is_changed() || level.is_changed()) {
        return;
    }
    let point = surface
        .point
        .filter(|_| matches!(*level, Level::Planet(_)) && surface.over);
    let said = point.map_or(String::new(), words);
    for mut text in &mut texts {
        text.set_if_neq(Text(said.clone()));
    }
}
