//! The words of the planet level over the map: the caption, which gives the climate of the
//! planet shown, and the latitude and longitude under the pointer, which also say when that
//! ground is not explored.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::explored::Explored;
use super::explored::grid::{depth_for, leaf_of};
use super::geo::coords as words;
use super::level::Level;
use super::surface::Surface;
use crate::computer::climate_line;
use crate::space::Clock;
use crate::ui::{text, theme};
use crate::universe::{BodyId, Universe};

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
        text("", theme::FONT_SMALL, theme::TEXT_2),
        TextLayout::no_wrap(),
    )
}

/// The line that says the latitude and longitude under the pointer.
pub fn coords() -> impl Bundle {
    (
        Coords,
        text("", theme::FONT_SMALL, theme::TEXT_2),
        TextLayout::no_wrap(),
    )
}

/// Returns the caption of the level: the climate at `time` seconds of the planet shown, which
/// the path names, and nothing on a chart, whose list already names it.
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

/// What the line under the pointer says when the ground there is not explored.
pub const UNSEEN: &str = "Not explored yet.";

/// Returns true when the ground of `body` in the direction `dir` is not explored.
pub fn unexplored(explored: &Explored, universe: &Universe, body: BodyId, dir: DVec3) -> bool {
    let leaf = leaf_of(dir, depth_for(universe.planet(body).radius));
    !explored
        .leaves(body)
        .is_some_and(|leaves| leaves.contains(leaf))
}

/// What the line under the pointer is written from: what is explored and the universe.
type Ground<'w> = (Option<Res<'w, Explored>>, Option<Res<'w, Universe>>);

/// Writes the latitude and longitude under the pointer on a planet, with a sentence when that
/// ground is not explored, and nothing elsewhere.
pub(super) fn coords_text(
    (surface, level): (Res<Surface>, Res<Level>),
    (explored, universe): Ground,
    mut texts: Query<&mut Text, With<Coords>>,
) {
    if !(surface.is_changed() || level.is_changed()) {
        return;
    }
    let point = surface.point.filter(|_| surface.over);
    let said = match (*level, point) {
        (Level::Planet(body), Some(dir)) => {
            let seen = explored.zip(universe);
            let blind = seen.is_some_and(|(e, u)| unexplored(&e, &u, body, dir));
            let note = [String::new(), format!("  {UNSEEN}")][usize::from(blind)];
            format!("{}{note}", words(dir))
        }
        _ => String::new(),
    };
    for mut text in &mut texts {
        text.set_if_neq(Text(said.clone()));
    }
}
