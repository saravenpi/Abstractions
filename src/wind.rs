//! The wind: one slowly turning direction and a gusting strength shared by clouds, water and particles.
//!
//! The direction lives in the tangent plane under the render anchor: render X and Z. When the
//! anchor moves on its planet the direction is turned with the render axes, so the wind keeps
//! blowing the same way over the ground.

use std::f32::consts::{PI, TAU};

use bevy::prelude::*;

use crate::space::{Anchor, Shift};

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
}

impl Default for Wind {
    fn default() -> Self {
        wind_at(0.0)
    }
}

impl Wind {
    /// Returns the wind as a velocity in meters per second along the tangent plane of the
    /// anchor, in render axes. Turn it by the tilt of the local vertical for a place away from
    /// the render origin.
    pub fn velocity(&self) -> Vec3 {
        Vec3::new(self.direction.x, 0.0, self.direction.y) * (1.0 + 5.0 * self.strength)
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

/// Advances the wind with the game clock and keeps it turned with the render anchor.
fn blow(
    time: Res<Time>,
    anchor: Option<Res<Anchor>>,
    mut veer: Local<Veer>,
    mut wind: ResMut<Wind>,
) {
    let clock = wind_at(time.elapsed_secs());
    let direction = match anchor {
        Some(anchor) => veer.track(&anchor, clock.direction),
        None => clock.direction,
    };
    *wind = Wind { direction, ..clock };
}

/// Registers the wind resource and its system.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Wind>().add_systems(Update, blow);
}

#[cfg(test)]
mod tests {
    use std::f64::consts::PI as PI64;
    use std::sync::Arc;

    use bevy::math::{DQuat, DVec3};

    use super::*;
    use crate::space::rebased;
    use crate::universe::Universe;
    use crate::world::globe::Face;
    use crate::world::{Globe, Terrain};

    /// The wind direction stays a unit vector and the strength stays in range.
    #[test]
    fn wind_stays_in_range() {
        for step in 0..500 {
            let wind = wind_at(step as f32 * 7.3);
            assert!((wind.direction.length() - 1.0).abs() < 1e-4);
            assert!((0.0..=1.0).contains(&wind.strength));
        }
    }

    /// Returns the compass angle of render direction `wind` at render position `at`: its angle
    /// from the planet east in the tangent plane there.
    fn compass(anchor: &Anchor, at: Vec3, wind: Vec2) -> f64 {
        let up = anchor.to_frame(at).normalize();
        let blown = anchor.rotation.inverse() * Vec3::new(wind.x, 0.0, wind.y).as_dvec3();
        let east = DVec3::Y.cross(up).normalize();
        blown.dot(up.cross(east)).atan2(blown.dot(east))
    }

    /// Across a rebase, and across an anchor whose axes were also twisted around the vertical,
    /// the wind keeps its compass direction where the player stands within 1e-3 rad.
    #[test]
    fn wind_is_continuous_across_a_rebase() {
        let globe = Arc::new(Globe::new(4, 3000.0, 22.0));
        let view = Terrain::on(globe.clone(), globe.face_chart(Face::PosX)).around(40.0, -25.0);
        let stand = Vec3::new(40.0, 0.0, -25.0);
        let old = Anchor::surface(Universe::new(4).home(), &view, stand);
        let walked = stand + Vec3::new(28.0, 0.0, 21.0);
        let new = rebased(&old, 3000.0, walked).expect("a rebase past the limit");
        let twisted = Anchor {
            rotation: DQuat::from_rotation_y(0.8) * new.rotation,
            ..new
        };
        for next in [new, twisted] {
            let blowing = wind_at(77.0).direction;
            let mut veer = Veer::default();
            let before = compass(&old, walked, veer.track(&old, blowing));
            let at = next.to_render(old.to_frame(walked));
            let after = compass(&next, at, veer.track(&next, blowing));
            let change = (after - before + PI64).rem_euclid(2.0 * PI64) - PI64;
            assert!(change.abs() < 1e-3, "{before} -> {after}");
            assert_eq!(veer.track(&next, blowing), veer.track(&next, blowing));
        }
    }
}
