//! Lighting the bodies each frame: every planet and moon from its star through the shadows of
//! its family, with the light the ground gets, its clouds drifting over the ball of its real
//! surface, the storm of a gas giant turning with it, and the rings from their star.

use bevy::ecs::system::SystemParam;
use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::Body;
use super::giant::Giant;
use super::material::{BodyMaterial, BodyUniform};
use super::rings::RingMaterial;
use crate::render::air::Air;
use crate::render::celestial::smoothstep;
use crate::render::clouds::{CloudMaps, toward_second};
use crate::render::palette::Palette;
use crate::render::scatter::peak;
use crate::render::sky::linear;
use crate::render::vantage::Scene;
use crate::settings::{Persistent, Settings};
use crate::universe::PlanetSpec;
use crate::weather::drift;
use crate::world::{Link, spin};

/// Angular radii in radians between which a baked body goes from a dot in the mean colour of
/// its surface to a disc that shows the surface and its clouds: from about one pixel of the
/// game view to three, under which single triangles would flicker.
pub const DETAIL: (f32, f32) = (0.004, 0.012);

/// Returns how much of its baked surface a body shows when its radius covers `angle` radians.
pub fn detail(angle: f32) -> f32 {
    smoothstep(DETAIL.0, DETAIL.1, angle)
}

/// Returns the linear colour of the light of a star of linear colour `star` times its
/// illuminance in lux: what the sun light of the ground carries in open space.
pub fn star_light(star: Vec3) -> Vec3 {
    linear(peak(star)).truncate() * Palette::SPACE_LUX
}

/// What the clouds of the bodies are drawn from.
#[derive(SystemParam)]
pub(super) struct Skies<'w> {
    /// The settings, which can switch the clouds off.
    settings: Option<Res<'w, Persistent<Settings>>>,
    /// The cover maps of the bodies whose clouds show.
    maps: Option<ResMut<'w, CloudMaps>>,
}

impl Skies<'_> {
    /// Returns the cover map of `body` once it is ready, and asks for it to be kept, while the
    /// body `shows` its clouds: under the camera or big enough on screen. Nothing with clouds
    /// switched off, and nothing for a body without clouds.
    fn map(&mut self, body: &PlanetSpec, shows: bool) -> Option<Handle<Image>> {
        let off = self.settings.as_ref().is_some_and(|s| !s.graphics.clouds);
        match shows && !off {
            true => self.maps.as_mut()?.want(body.id),
            false => None,
        }
    }
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

/// Lights every planet and moon from the direction of its star, dimmed where another body of
/// its planet hides the star, and turns its spin axis, its surface, its clouds and the storm of
/// a giant into render axes.
pub(super) fn planets(
    scene: Scene,
    mut skies: Skies,
    planets: Query<(&Body, &MeshMaterial3d<BodyMaterial>)>,
    mut materials: ResMut<Assets<BodyMaterial>>,
) {
    let Some(near) = scene.local.0.as_ref() else {
        return;
    };
    let (vantage, time) = (&*scene.vantage, scene.clock.seconds);
    let to_render = vantage.observer.to_render;
    for (body, handle) in &planets {
        let found = match *body {
            Body::Planet(id) => scene.body(id).zip(materials.get_mut(&handle.0)),
            _ => None,
        };
        let Some(((planet, pos), mut material)) = found else {
            continue;
        };
        let uniform = &mut material.body;
        uniform.light = vantage
            .sunward(pos)
            .extend(near.eclipse(pos, planet.id, time));
        uniform.axis = (to_render * DVec3::Y).as_vec3().extend(uniform.axis.w);
        if let Some(giant) = Giant::of(&planet.record, planet.seed) {
            let spot = to_render * (spin(planet.day_seconds, time) * giant.spot);
            uniform.storm = spot.as_vec3().extend(giant.size);
        }
        let to_fixed = (to_render * Link::orbiting(planet, time).turn).inverse();
        let angle = (planet.radius / pos.distance(vantage.observer.eye).max(1.0)) as f32;
        let map = skies.map(planet, vantage.is(planet.id) || detail(angle) > 0.0);
        let sky = (near.system.star.color, angle, map.is_some(), time);
        dress(uniform, planet, to_fixed, sky);
        material.cover = map;
    }
}

/// Lights the rings of every ringed planet from the direction of its star.
pub(super) fn rings(
    scene: Scene,
    rings: Query<(&Body, &MeshMaterial3d<RingMaterial>)>,
    mut materials: ResMut<Assets<RingMaterial>>,
) {
    for (body, handle) in &rings {
        let Body::Rings(id) = *body else {
            continue;
        };
        let Some((_, pos)) = scene.body(id) else {
            continue;
        };
        if let Some(mut material) = materials.get_mut(&handle.0) {
            material.light(pos);
        }
    }
}
