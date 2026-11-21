//! The sun and moon directional lights, aimed by the real star and the real moon and tinted by
//! the palette. A night without a moon leaves the moon light dark.
//!
//! The sun gives what the rule of `daylight.wesl` gives the camera, so the ground shader, which
//! corrects the light for the place it shades, finds nothing to correct at the camera. The
//! lights run after the place light, in the same schedule, and before Bevy builds the shadow
//! cascades from their pose.

mod quality;
mod reach;
mod tune;
mod twin;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_reach;

pub use self::reach::reach;

use bevy::camera::visibility::Visibility;
use bevy::light::{DirectionalLight, DirectionalLightShadowMap, SimulationLightSystems};
use bevy::prelude::*;

use self::quality::cascades;
use self::tune::{MOON_SHADOW_LUX, SHADOW_HEIGHT};
use super::SunDir;
use super::daylight::{PlaceLight, PlaceLit, PlaceUniform};
use super::moonlight::Moonlight;
use super::palette::{Palette, srgb};
use crate::screens::Screen;
use crate::settings::{Persistent, Settings};

/// Marker for the world's directional sun light.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Sun;

/// Marker for the world's directional moon light.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Moon;

/// Transform and light of a directional light entity.
type LightParts = (
    &'static mut Transform,
    &'static mut GlobalTransform,
    &'static mut DirectionalLight,
);

/// Filter matching the sun only.
type SunOnly = (With<Sun>, Without<Moon>);

/// Filter matching the moon only.
type MoonOnly = (With<Moon>, Without<Sun>);

/// What one light should look like this frame.
struct Aim {
    dir: Vec3,
    height: f32,
    color: Color,
    lux: f32,
    shadows: bool,
    floor: f32,
}

impl Aim {
    /// Points the light from `dir` toward the origin and sets its color, strength and shadows.
    /// It has no shadow below its `floor` of lux. The global transform is written with the local
    /// one: the light is a root entity and moves after transform propagation.
    fn apply(self, pose: (&mut Transform, &mut GlobalTransform), light: &mut DirectionalLight) {
        let (transform, global) = pose;
        transform.look_to(-self.dir, Vec3::Y);
        *global = GlobalTransform::from(*transform);
        light.color = self.color;
        light.illuminance = self.lux;
        let shadows = self.shadows && self.height > SHADOW_HEIGHT && self.lux > self.floor;
        if light.shadow_maps_enabled != shadows {
            light.shadow_maps_enabled = shadows;
        }
    }
}

/// Spawns the sun and the moon when the world starts loading.
fn spawn_lights(mut commands: Commands, settings: Res<Persistent<Settings>>) {
    let own = &settings.graphics.lighting;
    let config = cascades(own.shadow_reach, own.shadow_detail);
    let light = DirectionalLight {
        shadow_maps_enabled: false,
        ..default()
    };
    commands.spawn((
        Sun,
        light,
        config.clone(),
        Transform::default(),
        Visibility::default(),
        DespawnOnEnter(Screen::Title),
    ));
    commands.spawn((
        Moon,
        light,
        config,
        Transform::default(),
        Visibility::default(),
        DespawnOnEnter(Screen::Title),
    ));
}

/// Returns the share of the light of the star that reaches the camera once a moon or a distant
/// star has taken its part: [`SunDir::lit`] above the horizon, where the planet casts no shadow
/// and `daylight.wesl` already counts the rest. Below it the planet's shadow is in the place
/// block, so nothing is taken twice.
fn beam(sun: &SunDir) -> f32 {
    if sun.height >= 0.0 {
        sun.lit.min(1.0)
    } else {
        1.0
    }
}

/// Returns the colour and the illuminance of the sun light: what the place block gives the
/// camera, the same linear light the ground shader takes as its reference, times the
/// [`beam`]. Without a lit planet, as in space, it is the light of the palette.
fn sun_light(place: &PlaceUniform, sun: &SunDir, palette: &Palette) -> (Color, f32) {
    if place.air.z <= 0.0 {
        return (srgb(palette.sun_color), palette.sun_lux);
    }
    let light = place.eye_sun.truncate() * beam(sun);
    let lux = light.max_element();
    if lux <= 0.0 {
        return (Color::WHITE, 0.0);
    }
    let tint = light / lux;
    (Color::linear_rgb(tint.x, tint.y, tint.z), lux)
}

/// Aims the sun at the star and the moon light at the real moon, which the palette leaves dark
/// on a planet without moons, tints both and applies the shadow setting. The sun takes the
/// light of the place block, the moon the palette.
fn update_lights(
    sky: (Res<Palette>, Res<SunDir>, Res<Moonlight>, Res<PlaceLight>),
    settings: Res<Persistent<Settings>>,
    mut suns: Query<LightParts, SunOnly>,
    mut moons: Query<LightParts, MoonOnly>,
) {
    let (palette, sun, moon, place) = sky;
    let shadows = settings.graphics.shadows;
    let (color, lux) = sun_light(&place.0, &sun, &palette);
    for (mut transform, mut global, mut light) in &mut suns {
        let aim = Aim {
            dir: sun.dir,
            height: sun.height,
            color,
            lux,
            shadows,
            floor: 0.0,
        };
        aim.apply((&mut *transform, &mut *global), &mut light);
    }
    for (mut transform, mut global, mut light) in &mut moons {
        let aim = Aim {
            dir: moon.0.map_or(Vec3::X, |glow| glow.dir),
            height: moon.0.map_or(0.0, |glow| glow.height),
            color: Color::srgb(0.62, 0.72, 1.0),
            lux: palette.moon_lux,
            shadows,
            floor: MOON_SHADOW_LUX,
        };
        aim.apply((&mut *transform, &mut *global), &mut light);
    }
}

/// Registers the lights, their twins on the water mirror layer and their update, which runs after
/// the place light is worked out and before the shadow cascades are built from the pose of the
/// lights.
pub(super) fn plugin(app: &mut App) {
    app.add_plugins(quality::plugin)
        .init_resource::<DirectionalLightShadowMap>()
        .add_systems(OnEnter(Screen::Loading), (spawn_lights, twin::spawn))
        .add_systems(
            PostUpdate,
            (update_lights, twin::follow)
                .chain()
                .after(PlaceLit)
                .before(SimulationLightSystems::UpdateDirectionalLightCascades),
        );
}
