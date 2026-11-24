//! What the streamer reads of the running game: where the focus is and what it sees, and what
//! time it is.

use bevy::ecs::system::SystemParam;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::sight::Eye;
use crate::world::Terrain;
use crate::world::focus::Focus;

/// The focus and the clock, for the systems of the streaming chain.
#[derive(SystemParam)]
pub struct Scene<'w, 's> {
    focus: Focus<'w, 's>,
    time: Option<Res<'w, Time>>,
    terrain: Option<Res<'w, Terrain>>,
}

impl Scene<'_, '_> {
    /// Returns the focus position in metres from the planet centre, in the planet frame.
    pub fn at(&self) -> Option<DVec3> {
        self.focus.planet()
    }

    /// Returns the margin in metres the fades are written with at this focus: wider than the
    /// farthest node of the planet can be from it, see [`super::dither::span`].
    pub fn span(&self) -> f32 {
        match (self.at(), self.terrain.as_ref()) {
            (Some(at), Some(terrain)) => {
                let radius = terrain.globe().radius();
                super::dither::span(at.length() - radius, radius)
            }
            _ => super::dither::SPAN,
        }
    }

    /// Returns the seconds the last frame took.
    pub fn delta(&self) -> f64 {
        self.time.as_ref().map_or(0.0, |time| time.delta_secs_f64())
    }

    /// Returns the seconds the game has run.
    pub fn now(&self) -> f64 {
        self.time
            .as_ref()
            .map_or(0.0, |time| time.elapsed_secs_f64())
    }

    /// Returns what the focus can see through the air, or nothing without a focus on a planet.
    pub fn eye(&self) -> Option<Eye> {
        let (at, terrain) = (self.at()?, self.terrain.as_ref()?);
        Some(Eye::of(terrain.globe(), at))
    }
}
