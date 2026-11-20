//! The collapse: a body that died falls under the gravity of its planet, tips over about its feet
//! like a rod standing on one end, bounces once or twice and lies still. A flyer drops first.

use bevy::math::DQuat;
use bevy::prelude::*;

use super::tune::BOUNCE;
use crate::fauna::beast::Beast;
use crate::space::Placed;
use crate::world::Terrain;

/// The fall of a body that just died. It is removed once the body lies still.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Slump {
    /// The place it tips about, in its flat frame: on the ground under the torso.
    pub pivot: Vec3,
    /// The unit horizontal axis it tips about: the way it faced, so it falls on its side.
    pub axis: Vec3,
    /// Metres its middle lies over the ground once down.
    pub side: f32,
    /// Metres of the rod it tips like.
    pub length: f32,
    /// Radians tipped so far.
    pub angle: f32,
    /// Radians per second it tips at.
    pub rate: f32,
    /// Radians it lies at.
    pub rest: f32,
    /// Metres it still has to fall before it touches the ground.
    pub air: f32,
    /// Metres it has fallen.
    pub fell: f32,
    /// Metres per second it falls at.
    pub sink: f32,
}

/// Returns the tip `angle` and `rate` after `dt` seconds for a rod of `length` metres under `gravity`
/// that lies at `rest`: a rod standing on its end, `1.5·g/L·sin θ`, bouncing back with a share of
/// its speed, and true once it lies still.
pub fn tip(
    (angle, rate): (f32, f32),
    (length, gravity, rest): (f32, f32, f32),
    dt: f32,
) -> (f32, f32, bool) {
    let accel = 1.5 * gravity / length.max(0.05) * angle.sin().max(0.1);
    let rate = rate + accel * dt;
    let angle = angle + rate * dt;
    if angle < rest {
        return (angle, rate, false);
    }
    let back = rate * BOUNCE;
    (rest, -back, back < 0.2)
}

/// Returns the metres still to fall, the metres fallen and the speed after `dt` seconds of free fall
/// under `gravity`.
pub fn plunge((air, fell, sink): (f32, f32, f32), gravity: f32, dt: f32) -> (f32, f32, f32) {
    let sink = sink + gravity * dt;
    let step = (sink * dt).min(air);
    let air = air - step;
    (air, fell + step, if air > 0.0 { sink } else { 0.0 })
}

/// Returns where the root of the body lies for `slump` in the frame of `beast`: tipped about its
/// pivot and lowered by what it fell.
fn lie(slump: &Slump, beast: &Beast) -> Placed {
    let turn = Quat::from_axis_angle(slump.axis, slump.angle);
    let lift = Vec3::Y * (slump.side * slump.angle / slump.rest.max(1e-3) - slump.fell);
    let origin = slump.pivot + lift - turn * slump.pivot;
    Placed {
        pos: beast.spot.planet(origin),
        rot: beast.spot.rot * DQuat::from_axis_angle(slump.axis.as_dvec3(), f64::from(slump.angle)),
    }
}

/// Lets every body that died fall one frame.
pub fn slump(
    mut commands: Commands,
    (time, terrain): (Res<Time>, Res<Terrain>),
    mut bodies: Query<(Entity, &Beast, &mut Placed, &mut Slump)>,
) {
    let (gravity, dt) = (terrain.globe().gravity() as f32, time.delta_secs());
    for (entity, beast, mut placed, mut s) in &mut bodies {
        if s.air > 0.0 {
            (s.air, s.fell, s.sink) = plunge((s.air, s.fell, s.sink), gravity, dt);
        } else {
            let (angle, rate, done) = tip((s.angle, s.rate), (s.length, gravity, s.rest), dt);
            (s.angle, s.rate) = (angle, rate);
            if done {
                commands.entity(entity).remove::<Slump>();
            }
        }
        *placed = lie(&s, beast);
    }
}
