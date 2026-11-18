//! The wind: one slowly turning direction and a gusting strength shared by clouds, water and particles.
//!
//! The direction lives in the tangent plane under the render anchor: render X and Z. When the
//! anchor moves on its planet the direction is turned with the render axes, so the wind keeps
//! blowing the same way over the ground.

use std::f32::consts::{PI, TAU};

use bevy::prelude::*;

pub use self::air::{NO_AIR, THICKEST, Thickness, breeze, thickness};
#[cfg(test)]
pub use self::air::{PRESSURES, samples};
use self::follow::{to_render, weathered};
use crate::space::{Anchor, Frame, Shift};
use crate::universe::{BodyId, Universe};
use crate::weather::{Forecast, Weather};

/// Seconds for the wind direction to turn a full circle.
const TURN_SECONDS: f32 = 900.0;

/// Seconds between two gust peaks.
const GUST_SECONDS: f32 = 23.0;

/// The wind blowing over the world.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Wind {
    /// Unit direction the wind blows toward, on the render XZ plane, the tangent plane of the anchor.
    pub direction: Vec2,
    /// Strength from 0 (still) to 1 (strong gust).
    pub strength: f32,
    /// Share of the standing breeze the air of the planet carries: 0 without air, 1 in the home
    /// air and in thicker air. It is what leaves smoke, flames and leaves still on a bare rock.
    pub air: f32,
}

impl Default for Wind {
    fn default() -> Self {
        wind_at(0.0)
    }
}

impl Wind {
    /// Returns the wind as a velocity in meters per second along the tangent plane of the
    /// anchor, in render axes. Turn it by the tilt of the local vertical for a place away from
    /// the render origin. In air it never drops under the standing breeze of 1 m/s; without
    /// air it is zero.
    pub fn velocity(&self) -> Vec3 {
        Vec3::new(self.direction.x, 0.0, self.direction.y) * (self.air + 5.0 * self.strength)
    }
}

/// Returns the wind at `seconds` of game time, before it is turned with the anchor.
pub fn wind_at(seconds: f32) -> Wind {
    let angle = seconds / TURN_SECONDS * TAU + 0.7;
    let gust = (seconds / GUST_SECONDS * TAU).sin();
    let swell = (seconds / (GUST_SECONDS * 3.7) * TAU).sin();
    Wind {
        direction: Vec2::from_angle(angle),
        strength: (0.45 + 0.3 * gust + 0.2 * swell).clamp(0.0, 1.0),
        air: 1.0,
    }
}

/// How far the wind has been turned so far to follow the render axes over the planet.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Veer {
    /// Anchor the turn was last brought up to date with.
    seen: Option<Anchor>,
    /// Angle in radians added to the clock direction of the wind.
    turn: f32,
}

impl Veer {
    /// Returns `blowing`, the clock direction of the wind, turned to the render axes of
    /// `anchor`. When the anchor moved within its frame since the last call, the turn first
    /// takes up the twist of the render axes around the vertical, so the direction over the
    /// ground stays what it was. A flat anchor or one in a new frame carries nothing over.
    pub fn track(&mut self, anchor: &Anchor, blowing: Vec2) -> Vec2 {
        let old = self.seen.replace(*anchor);
        let moved = old.filter(|old| old.frame == anchor.frame && !anchor.flat && old != anchor);
        if let Some(old) = moved {
            let was = Vec2::from_angle(self.turn).rotate(blowing);
            let now = Shift::between(&old, anchor).direction(Vec3::new(was.x, 0.0, was.y));
            let twist = was.angle_to(Vec2::new(now.x, now.z));
            self.turn = (self.turn + twist + PI).rem_euclid(TAU) - PI;
        }
        Vec2::from_angle(self.turn).rotate(blowing)
    }
}

/// Returns `wind` as it blows in air of `thickness` against the home planet: its strength and
/// its standing breeze follow [`breeze`], so it is still without air and unchanged at home.
pub fn carried(wind: Wind, thickness: f32) -> Wind {
    Wind {
        strength: (wind.strength * breeze(thickness)).clamp(0.0, 1.0),
        air: breeze(thickness).min(1.0),
        ..wind
    }
}

/// Keeps [`Thickness`] on the air of the planet under the render anchor. The planet is looked
/// up only when the anchor changes planet or the universe changes.
fn weigh(
    anchor: Option<Res<Anchor>>,
    universe: Option<Res<Universe>>,
    mut seen: Local<Option<(Universe, BodyId)>>,
    mut air: ResMut<Thickness>,
) {
    let on = anchor.as_deref().map(|anchor| anchor.frame);
    let (Some(Frame::Surface(body)), Some(universe)) = (on, universe) else {
        *seen = None;
        air.set_if_neq(Thickness::default());
        return;
    };
    if *seen != Some((*universe, body)) {
        *seen = Some((*universe, body));
        air.set_if_neq(Thickness(thickness(&universe.planet(body).record) as f32));
    }
}

/// Advances the wind with the game clock, keeps it turned with the render anchor, leans it
/// toward the prevailing wind of the weather and its storm gusts, and lets it blow as hard as
/// the air of the planet carries it.
fn blow(
    time: Res<Time>,
    place: (Option<Res<Anchor>>, Option<Res<Weather>>),
    air: Res<Thickness>,
    mut veer: Local<Veer>,
    mut wind: ResMut<Wind>,
) {
    let clock = wind_at(time.elapsed_secs());
    let (anchor, weather) = place;
    let direction = match anchor.as_deref() {
        Some(anchor) => veer.track(anchor, clock.direction),
        None => clock.direction,
    };
    let turned = Wind { direction, ..clock };
    let blown = match (anchor.as_deref(), weather.as_deref()) {
        (Some(anchor), Some(sky)) => weathered(turned, to_render(anchor, sky.wind), sky.storm),
        _ => turned,
    };
    *wind = carried(blown, air.0);
}

/// Registers the wind and air resources and their systems.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Wind>()
        .init_resource::<Thickness>()
        .add_systems(Update, (weigh, blow).chain().after(Forecast));
}

mod air;
mod follow;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_follow;
