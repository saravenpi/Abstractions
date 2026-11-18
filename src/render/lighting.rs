//! The sun and moon directional lights, aimed by the real star and tinted by the palette.

use bevy::camera::visibility::Visibility;
use bevy::light::{CascadeShadowConfigBuilder, DirectionalLight, DirectionalLightShadowMap};
use bevy::prelude::*;

use super::SunDir;
use super::moonlight::Moonlight;
use super::palette::{Palette, srgb};
use crate::screens::Screen;
use crate::settings::{Persistent, Settings};

/// Lowest light height above the horizon, as a sine, at which shadows are still cast.
const SHADOW_HEIGHT: f32 = 0.02;

/// Marker for the world's directional sun light.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Sun;

/// Marker for the world's directional moon light.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Moon;

/// Transform and light of a directional light entity.
type LightParts = (&'static mut Transform, &'static mut DirectionalLight);

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
}

impl Aim {
    /// Points the light from `dir` toward the origin and sets its color, strength and shadows.
    fn apply(self, transform: &mut Transform, light: &mut DirectionalLight) {
        transform.look_to(-self.dir, Vec3::Y);
        light.color = self.color;
        light.illuminance = self.lux;
        let shadows = self.shadows && self.height > SHADOW_HEIGHT;
        if light.shadow_maps_enabled != shadows {
            light.shadow_maps_enabled = shadows;
        }
    }
}

/// Returns the shadow setup shared by both lights: two cascades out to 110 meters, enough at a third of the resolution.
fn cascades() -> bevy::light::CascadeShadowConfig {
    CascadeShadowConfigBuilder {
        num_cascades: 2,
        minimum_distance: 0.1,
        first_cascade_far_bound: 28.0,
        maximum_distance: 110.0,
        overlap_proportion: 0.2,
    }
    .build()
}

/// Spawns the sun and the moon when the world starts loading.
fn spawn_lights(mut commands: Commands) {
    let light = DirectionalLight {
        shadow_maps_enabled: false,
        ..default()
    };
    commands.spawn((
        Sun,
        light,
        cascades(),
        Transform::default(),
        Visibility::default(),
        DespawnOnEnter(Screen::Title),
    ));
    commands.spawn((
        Moon,
        light,
        cascades(),
        Transform::default(),
        Visibility::default(),
        DespawnOnEnter(Screen::Title),
    ));
}

/// Aims the sun at the star and the moon at the real moon, or away from the star on a planet
/// without moons, tints both from the palette and applies the shadow setting.
fn update_lights(
    sky: (Res<Palette>, Res<SunDir>, Res<Moonlight>),
    settings: Res<Persistent<Settings>>,
    mut suns: Query<LightParts, SunOnly>,
    mut moons: Query<LightParts, MoonOnly>,
) {
    let (palette, sun, moon) = sky;
    let shadows = settings.graphics.shadows;
    for (mut transform, mut light) in &mut suns {
        let aim = Aim {
            dir: sun.dir,
            height: sun.height,
            color: srgb(palette.sun_color),
            lux: palette.sun_lux,
            shadows,
        };
        aim.apply(&mut transform, &mut light);
    }
    for (mut transform, mut light) in &mut moons {
        let aim = Aim {
            dir: moon.0.map_or(-sun.dir, |glow| glow.dir),
            height: moon.0.map_or(-sun.height, |glow| glow.height),
            color: Color::srgb(0.62, 0.72, 1.0),
            lux: palette.moon_lux,
            shadows,
        };
        aim.apply(&mut transform, &mut light);
    }
}

/// Registers the lights and their update.
pub(super) fn plugin(app: &mut App) {
    app.insert_resource(DirectionalLightShadowMap { size: 2048 })
        .add_systems(OnEnter(Screen::Loading), spawn_lights)
        .add_systems(Update, update_lights);
}
