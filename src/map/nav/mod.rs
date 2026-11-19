//! Going in and out of the levels of the map. The map shows one level at a time: the universe,
//! a galaxy, a system or a planet. A double click on a place goes one level into it, Back or
//! Backspace comes out to what was shown before, and each part of the path over the map jumps
//! straight to that level. The wheel never changes the level.

mod bar;
mod go;

pub use bar::path;

use bevy::prelude::*;

use crate::computer::{Computer, Lens, Tab, Target};
use crate::map::level::Level;
use crate::universe::{BodyId, GalaxyId, StarId, Universe};

/// Seconds the map eases onto a place before it goes into it.
pub const DIVE_TIME: f32 = 0.35;

/// A level of the map, with the place it shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Place {
    /// The chart of the known galaxies.
    Universe,
    /// The chart of the known systems of a galaxy.
    Galaxy(GalaxyId),
    /// The chart of a star and its planets.
    System(StarId),
    /// The flat map of a planet or a moon.
    Planet(BodyId),
}

impl Place {
    /// Returns the level around this one, or nothing for the universe.
    pub fn parent(self) -> Option<Place> {
        match self {
            Place::Planet(body) => Some(Place::System(body.star)),
            Place::System(star) => Some(Place::Galaxy(star.galaxy)),
            Place::Galaxy(_) => Some(Place::Universe),
            Place::Universe => None,
        }
    }

    /// Returns the level a double click on `target` goes into.
    pub fn of(target: Target) -> Place {
        match target {
            Target::Body(body) => Place::Planet(body),
            Target::Star(star) => Place::System(star),
            Target::Galaxy(galaxy) => Place::Galaxy(galaxy),
        }
    }

    /// Returns the level shown by `level` and a chart of `tab` on `inside`, where a chart
    /// with nothing inside shows the place of the ship, whose star is `ship`.
    pub fn shown(level: Level, (tab, inside): (Tab, Option<Target>), ship: StarId) -> Place {
        let star = match inside {
            Some(Target::Star(star)) => star,
            _ => ship,
        };
        match (level, tab, inside) {
            (Level::Planet(body), ..) => Place::Planet(body),
            (_, Tab::Universe, _) => Place::Universe,
            (_, Tab::Galaxy, Some(Target::Galaxy(galaxy))) => Place::Galaxy(galaxy),
            (_, Tab::Galaxy, _) => Place::Galaxy(star.galaxy),
            (_, Tab::System, _) => Place::System(star),
        }
    }

    /// Returns the levels from the universe down to this one.
    pub fn path(self) -> Vec<Place> {
        let mut path: Vec<Place> =
            std::iter::successors(Some(self), |place| place.parent()).collect();
        path.reverse();
        path
    }

    /// Returns the name of the level, as the path over the map writes it.
    pub fn name(self, universe: &Universe) -> String {
        match self {
            Place::Universe => "Universe".to_string(),
            Place::Galaxy(galaxy) => Target::Galaxy(galaxy).name(universe),
            Place::System(star) => Target::Star(star).name(universe),
            Place::Planet(body) => universe.planet(body).name,
        }
    }
}

/// A level once shown, and how it was looked at, or nothing for the way it opens.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stop {
    /// The level.
    pub place: Place,
    /// The lens it was looked through.
    pub lens: Option<Lens>,
}

/// The levels shown before the one shown now, the last one last, and a place the map eases
/// onto before it goes into it, with the seconds left.
#[derive(Resource, Debug, Clone, Default, PartialEq)]
pub struct Trail {
    /// The levels to go back to.
    pub stops: Vec<Stop>,
    /// The place the map goes into once it has eased onto it.
    pub pending: Option<(Place, f32)>,
}

impl Trail {
    /// Returns the level Back goes to from `here`: the one shown before, or else the level
    /// around this one. Nothing at the top with nothing before.
    pub fn back(&mut self, here: Place) -> Option<Stop> {
        let around = || here.parent().map(|place| Stop { place, lens: None });
        self.stops.pop().or_else(around)
    }
}

/// Registers the way in and out of the levels: the trail starts empty each time the map opens.
pub(super) fn plugin(app: &mut App) {
    let fresh = |mut trail: ResMut<Trail>| *trail = Trail::default();
    app.init_resource::<Trail>()
        .add_observer(bar::on_press)
        .add_systems(OnEnter(Computer::Open), fresh);
}

/// The systems of the open map that go into a place asked for, go back on Backspace and write
/// the path over the map.
pub(super) fn systems() -> impl IntoScheduleConfigs<bevy::ecs::system::ScheduleSystem, ()> {
    (go::descend, bar::back_key, bar::write).chain()
}

#[cfg(test)]
#[path = "../nav_go_tests.rs"]
mod go_tests;
#[cfg(test)]
#[path = "../nav_tests.rs"]
mod tests;
