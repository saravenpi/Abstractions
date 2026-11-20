//! Lighting the bodies each frame: every planet and moon from its star through the shadows of
//! its family, with the light the ground gets, its clouds drifting over the ball of its real
//! surface, the storm of a gas giant turning with it, and the rings from their star.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

mod relight;

use self::relight::relit;
use super::Body;
use super::material::BodyMaterial;
use super::rings::RingMaterial;
use crate::render::celestial::smoothstep;
use crate::render::clouds::CloudMaps;
use crate::render::daylight::{PlaceUniform, starlight};
use crate::render::moonlight::{Moonlight, STARLIGHT};
use crate::render::palette::Palette;
use crate::render::scatter::peak;
use crate::render::sky::linear;
use crate::render::vantage::Scene;
use crate::settings::{Persistent, Settings};
use crate::universe::PlanetSpec;

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

/// Lights every planet and moon from the direction of its star, dimmed where another body of
/// its planet hides the star, and turns its spin axis, its surface, its clouds and the storm of
/// a giant into render axes. Each is lit through its own air, and the one under the camera
/// shares the night of the ground, under `moon`. A material is written only when what it
/// holds changes, and a body that is hidden keeps the light it was last given.
pub(super) fn planets(
    scene: Scene,
    mut skies: Skies,
    moon: Option<Res<Moonlight>>,
    planets: Query<(&Body, &MeshMaterial3d<BodyMaterial>, &Visibility)>,
    mut materials: ResMut<Assets<BodyMaterial>>,
) {
    let Some(near) = scene.local.0.as_ref() else {
        return;
    };
    let vantage = &*scene.vantage;
    for (body, handle, seen) in &planets {
        let Body::Planet(id) = *body else {
            continue;
        };
        let (Some((planet, pos)), Some(old)) = (scene.body(id), materials.get(&handle.0)) else {
            continue;
        };
        if *seen == Visibility::Hidden && old.place != PlaceUniform::default() {
            continue;
        }
        let below = moon.as_deref().filter(|_| vantage.is(planet.id));
        let stars = below.map_or(STARLIGHT, starlight);
        let angle = (planet.radius / pos.distance(vantage.observer.eye).max(1.0)) as f32;
        let map = skies.map(planet, vantage.is(planet.id) || detail(angle) > 0.0);
        let next = relit(old, (&scene, near), (planet, pos), (stars, angle, map));
        let changed = next.body != old.body || next.place != old.place || next.cover != old.cover;
        if changed && let Some(mut material) = materials.get_mut(&handle.0) {
            *material = next;
        }
    }
}

/// Lights the rings of every ringed planet from the direction of its star, when that changes
/// what their material holds, and not while they are hidden once they have been lit.
pub(super) fn rings(
    scene: Scene,
    rings: Query<(&Body, &MeshMaterial3d<RingMaterial>, &Visibility)>,
    mut materials: ResMut<Assets<RingMaterial>>,
) {
    for (body, handle, seen) in &rings {
        let Body::Rings(id) = *body else {
            continue;
        };
        let (Some((_, pos)), Some(old)) = (scene.body(id), materials.get(&handle.0)) else {
            continue;
        };
        if *seen == Visibility::Hidden && old.ring.light != Vec4::ZERO {
            continue;
        }
        let mut next = old.clone();
        next.light(pos);
        if next.ring.light != old.ring.light
            && let Some(mut material) = materials.get_mut(&handle.0)
        {
            material.ring.light = next.ring.light;
        }
    }
}
