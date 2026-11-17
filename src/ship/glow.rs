//! The engine glow: its brightness and the length of the exhaust follow the throttle.

use bevy::prelude::*;

use super::body::Ship;
use super::flight::BOOST;
use super::helm::Helm;
use super::warp::Warp;

/// The glowing parts of a ship.
#[derive(Component, Debug, Clone)]
pub struct Engine {
    /// Material shared by the nozzle discs and the exhaust.
    pub glow: Handle<StandardMaterial>,
}

/// Marker for the exhaust, a child of the ship stretched along z by the engine level.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Plume;

/// Linear color of the glow at full throttle: a blue far above 1, so that it blooms.
const FULL: Vec3 = Vec3::new(2.2, 5.6, 9.5);

/// Share of the full color an idle engine keeps.
const IDLE: f32 = 0.05;

/// Length in metres of the exhaust at full throttle.
const PLUME_LENGTH: f32 = 2.4;

/// Level of the engines in warp.
const WARP_LEVEL: f32 = 1.6;

/// Returns the glow color for an engine level from 0 (idle) to 1 (full throttle) and above.
///
/// The material is unlit, so the color is what reaches the screen and values above 1 bloom.
pub fn color(level: f32) -> Color {
    let rgb = FULL * (IDLE + level.max(0.0));
    Color::linear_rgb(rgb.x, rgb.y, rgb.z)
}

/// Returns the engine level of a ship: its throttle, raised by the boost and in warp.
fn level(ship: &Ship, boost: bool) -> f32 {
    let throttle = ship.throttle.abs() as f32;
    match ship.warp {
        Warp::Active { .. } => WARP_LEVEL,
        _ if boost => throttle * (BOOST as f32).sqrt(),
        _ => throttle,
    }
}

/// Sets the glow color and the exhaust length when the engine level changes.
pub fn glow(
    helm: Res<Helm>,
    ship: Single<(&Ship, &Engine)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut plumes: Query<&mut Transform, With<Plume>>,
    mut shown: Local<Option<f32>>,
) {
    let (ship, engine) = *ship;
    let level = level(ship, helm.controls.boost);
    if shown.is_some_and(|old| (old - level).abs() < 0.01) {
        return;
    }
    *shown = Some(level);
    if let Some(mut material) = materials.get_mut(&engine.glow) {
        material.base_color = color(level);
    }
    for mut plume in &mut plumes {
        plume.scale.z = 0.02 + PLUME_LENGTH * level;
    }
}
