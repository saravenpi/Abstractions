//! A flock of birds as one moving point: its path, its altitude and its formation.

use bevy::prelude::*;

use super::anim::curve;
use super::env::{Ctx, Sky};
use super::steer::{approach, bearing, turn_toward, wrap_angle};
use super::wander::{PARKED, ground};

/// Most birds each flock can show: crows first, then gulls.
pub const CAPACITY: [usize; 2] = [7, 6];

/// Cruise speed of each flock in meters per second.
const SPEED: [f32; 2] = [9.0, 8.0];

/// Distance from the camera beyond which a flock is moved back out of sight.
const LEAVE: f32 = 150.0;

/// Distance from the camera beyond which a flock turns back toward it.
const RETURN: f32 = 110.0;

/// How far ahead the ground is probed, in meters.
const LOOKAHEAD: f32 = 25.0;

/// One flock: where it is and where it heads.
#[derive(Debug, Clone, Copy)]
pub struct Flock {
    /// Center of the formation.
    pub center: Vec3,
    /// Direction of travel on the XZ plane.
    pub heading: f32,
    /// Smoothed turn rate in radians per second, positive to the right.
    pub turn: f32,
    /// Seconds of circling left.
    circle: f32,
    /// Seconds until the next chance to start circling.
    calm: f32,
    /// Direction of circling, `1.0` or `-1.0`.
    side: f32,
    /// Birds in the flock right now.
    pub size: usize,
    /// Altitude above the ground in meters.
    cruise: f32,
    /// Highest ground under or ahead of the flock.
    ground: f32,
    /// Seconds until the next terrain lookup.
    probe: f32,
}

/// Both flocks.
#[derive(Resource, Debug, Clone, Copy)]
pub struct Flocks(pub [Flock; 2]);

impl Default for Flocks {
    /// Parks both flocks far away until their first update.
    fn default() -> Self {
        let parked = Flock {
            center: PARKED,
            heading: 0.0,
            turn: 0.0,
            circle: 0.0,
            calm: 5.0,
            side: 1.0,
            size: 5,
            cruise: 25.0,
            ground: 0.0,
            probe: 0.0,
        };
        Self([parked; 2])
    }
}

/// Returns where the bird in `slot` flies relative to the flock center, in a loose V.
pub fn formation(slot: usize, heading: f32) -> Vec3 {
    let dir = Vec2::from_angle(heading);
    let side = if slot % 2 == 1 { 1.0 } else { -1.0 };
    let lateral = side * slot.div_ceil(2) as f32 * 3.0;
    let flat = -dir * slot as f32 * 1.8 + Vec2::new(-dir.y, dir.x) * lateral;
    Vec3::new(flat.x, ((slot * 7) % 5) as f32 * 0.6 - 1.2, flat.y)
}

impl Flock {
    /// Moves the flock out of sight beyond the camera, upwind, on a heading that crosses the area.
    fn reposition(&mut self, index: usize, sky: &Sky, ctx: &mut Ctx) {
        let back = -sky.wind;
        let lateral = Vec2::new(-back.y, back.x) * ctx.rng.range(-60.0, 60.0);
        let flat =
            Vec2::new(sky.camera.x, sky.camera.z) + back * ctx.rng.range(125.0, 140.0) + lateral;
        self.cruise = ctx.rng.range(15.0, 45.0);
        self.ground = ground(ctx, flat.x, flat.y);
        self.center = Vec3::new(flat.x, self.ground + self.cruise, flat.y);
        self.heading = bearing(self.center, sky.camera) + ctx.rng.range(-0.6, 0.6);
        let spread = (CAPACITY[index] - 2) as f32;
        self.size = (3 + (ctx.rng.next() * spread) as usize).min(CAPACITY[index]);
        self.circle = 0.0;
        self.calm = ctx.rng.range(5.0, 15.0);
        self.probe = 0.5;
    }

    /// Bends the heading: back toward the camera when far, in circles now and then, else in slow curves.
    fn steer(&mut self, index: usize, far: f32, sky: &Sky, ctx: &mut Ctx) {
        let dt = sky.dt;
        if far > RETURN {
            self.heading = turn_toward(self.heading, bearing(self.center, sky.camera), 0.4 * dt);
            return;
        }
        self.calm -= dt;
        if self.calm <= 0.0 {
            self.calm = ctx.rng.range(10.0, 25.0);
            self.circle = if ctx.rng.next() < 0.3 {
                ctx.rng.range(12.0, 25.0)
            } else {
                0.0
            };
            self.side = if ctx.rng.next() < 0.5 { 1.0 } else { -1.0 };
        }
        let rate = if self.circle > 0.0 {
            self.circle -= dt;
            self.side * 0.5
        } else {
            curve(sky.now, index as f32 * 3.1)
        };
        self.heading = wrap_angle(self.heading + rate * dt);
    }

    /// Flies the flock one frame.
    pub fn advance(&mut self, index: usize, sky: &Sky, ctx: &mut Ctx) {
        let dt = sky.dt.max(1e-4);
        let far = Vec2::new(self.center.x - sky.camera.x, self.center.z - sky.camera.z).length();
        if far > LEAVE {
            self.reposition(index, sky, ctx);
        }
        self.probe -= dt;
        if self.probe <= 0.0 {
            self.probe = 0.5;
            let ahead =
                self.center + Vec3::new(self.heading.cos(), 0.0, self.heading.sin()) * LOOKAHEAD;
            let here = ground(ctx, self.center.x, self.center.z);
            self.ground = here.max(ground(ctx, ahead.x, ahead.z));
        }
        let before = self.heading;
        self.steer(index, far, sky, ctx);
        let rate = wrap_angle(self.heading - before) / dt;
        self.turn += (rate - self.turn) * (1.0 - (-2.0 * dt).exp());
        let step = Vec2::from_angle(self.heading) * SPEED[index] * dt;
        self.center.x += step.x;
        self.center.z += step.y;
        self.center.y = approach(self.center.y, self.ground + self.cruise, 3.0 * dt);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The lead bird sits at the center and the rest trail behind it.
    #[test]
    fn followers_trail_the_leader() {
        let heading: f32 = 0.8;
        let dir = Vec3::new(heading.cos(), 0.0, heading.sin());
        assert_eq!(formation(0, heading).xz(), Vec2::ZERO);
        for slot in 1..7 {
            assert!(formation(slot, heading).dot(dir) < 0.0);
        }
    }
}
