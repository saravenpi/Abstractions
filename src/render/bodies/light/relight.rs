//! Lighting one baked planet for a frame: the light of its star through its own air, its spin,
//! its clouds and the storm of a giant, worked out on a copy of its material.

use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::super::giant::Giant;
use super::super::material::{BodyMaterial, BodyUniform};
use super::{detail, star_light};
use crate::render::air::Air;
use crate::render::clouds::toward_second;
use crate::render::daylight::{PlaceUniform, Sky};
use crate::render::vantage::{Nearby, Scene};
use crate::universe::PlanetSpec;
use crate::weather::drift;
use crate::world::{Link, spin};

/// Returns the block the shader lights the ball of `planet` with: its own air under a star of
/// linear colour `star` in the direction `toward`, on nights the stars light by `stars`. The
/// ball is lit point by point by the rule that lights its ground, whatever the camera stands in.
fn lit_by(planet: &PlanetSpec, star: Vec3, toward: Vec3, stars: f32) -> PlaceUniform {
    let size = (planet.radius as f32, planet.atmosphere_height as f32);
    let sky = Sky::of(star, &Air::of(&planet.record), size, stars);
    sky.uniform(Vec3::ZERO, toward, (1.0, 0.0))
}

/// Writes into `uniform` what the baked ball of `planet` is drawn from this frame: the rotation
/// `to_fixed` from render axes to its own, the light `star` of its star, the radians `angle`
/// its radius covers, which say how much of the surface shows, and for its clouds whether
/// their cover map is ready and the `seconds` of universe time that drift and morph them.
fn dress(
    uniform: &mut BodyUniform,
    planet: &PlanetSpec,
    to_fixed: DQuat,
    sky: (Vec3, f32, bool, f64),
) {
    let (star, angle, cloudy, seconds) = sky;
    let fixed = Mat3::from_quat(to_fixed.as_quat());
    uniform.fixed_x = fixed.x_axis.extend(0.0);
    uniform.fixed_y = fixed.y_axis.extend(0.0);
    uniform.fixed_z = fixed.z_axis.extend(0.0);
    uniform.sun = star_light(star).extend(f32::from(u8::from(cloudy)));
    let clouds = Air::of(&planet.record).clouds_from_space();
    uniform.cloud = clouds.extend(toward_second(seconds));
    uniform.drift = drift(seconds).extend(0.0);
    uniform.map.z = uniform.glow.w * detail(angle);
}

/// Turns the spin axis of `planet` and the storm of a giant into render axes through
/// `to_render` at `time` seconds of universe time.
fn orient(uniform: &mut BodyUniform, planet: &PlanetSpec, to_render: DQuat, time: f64) {
    uniform.axis = (to_render * DVec3::Y).as_vec3().extend(uniform.axis.w);
    if let Some(giant) = Giant::of(&planet.record, planet.seed) {
        let spot = to_render * (spin(planet.day_seconds, time) * giant.spot);
        uniform.storm = spot.as_vec3().extend(giant.size);
    }
}

/// Returns `old` lit for this frame: from the direction of its star, dimmed where another body of
/// its planet hides the star, its spin axis, its surface, its clouds and the storm of a giant
/// turned into render axes, through its own air under the night of `stars`. `found` is the
/// planet and its position in metres from the star, and `sky` the nights the stars light by, the
/// radians its radius covers and its cover map.
pub(super) fn relit(
    old: &BodyMaterial,
    (scene, near): (&Scene, &Nearby),
    found: (&PlanetSpec, DVec3),
    sky: (f32, f32, Option<Handle<Image>>),
) -> BodyMaterial {
    let (planet, pos) = found;
    let (stars, angle, map) = sky;
    let (vantage, time) = (&*scene.vantage, scene.clock.seconds);
    let to_render = vantage.observer.to_render;
    let toward = vantage.sunward(pos);
    let mut next = old.clone();
    next.place = lit_by(planet, near.system.star.color, toward, stars);
    next.body.light = toward.extend(near.eclipse(pos, planet.id, time));
    orient(&mut next.body, planet, to_render, time);
    let to_fixed = (to_render * Link::orbiting(planet, time).turn).inverse();
    let clouds = (near.system.star.color, angle, map.is_some(), time);
    dress(&mut next.body, planet, to_fixed, clouds);
    next.cover = map;
    next
}
