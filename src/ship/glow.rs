//! The engine glow: its brightness and the length of the exhaust follow the throttle. Every
//! hull has its own: the ship of this machine from its flight, a puppet from its records.

use bevy::prelude::*;

use super::body::Ship;
use super::flight::BOOST;
use super::helm::Helm;
use super::warp::Warp;
use crate::render::Sequence;

/// The glowing parts of a ship.
#[derive(Component, Debug, Clone)]
pub struct Engine {
    /// Material shared by the nozzle discs and the exhaust of this hull alone.
    pub glow: Handle<StandardMaterial>,
    /// Engine level to show, from 0 (idle) to 1 (full throttle) and above.
    pub level: f32,
    /// Engine level the material and the exhaust show, once written.
    shown: Option<f32>,
}

impl Engine {
    /// Returns the engine of a hull glowing with `glow`, idle and not drawn yet.
    pub fn new(glow: Handle<StandardMaterial>) -> Self {
        Self {
            glow,
            level: 0.0,
            shown: None,
        }
    }
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

/// Returns the engine level of a hull from its records: the `throttle`, raised by the share
/// of `boost` and by the share of `warp` the drive reached, full in warp.
pub fn level_of(throttle: f32, boost: f32, warp: f32) -> f32 {
    if warp >= 1.0 {
        return WARP_LEVEL;
    }
    let lift = 1.0 + ((BOOST as f32).sqrt() - 1.0) * boost.clamp(0.0, 1.0);
    (throttle.abs() * lift).max(WARP_LEVEL * warp.max(0.0))
}

/// Sets the engine level of the ship of this machine. The warp sequence lights the engines as
/// the drive charges and lets them fade as it drops out.
pub fn throttle(
    (helm, sequence): (Res<Helm>, Option<Res<Sequence>>),
    mut ships: Query<(&Ship, &mut Engine)>,
) {
    let charged = sequence.as_deref().map_or(0.0, Sequence::thrust);
    for (ship, mut engine) in &mut ships {
        engine.level = level(ship, helm.controls.boost).max(WARP_LEVEL * charged);
    }
}

/// Sets the glow color and the exhaust length of every hull whose engine level changed.
pub fn glow(
    mut engines: Query<(Entity, &mut Engine)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut plumes: Query<(&ChildOf, &mut Transform), With<Plume>>,
) {
    for (hull, mut engine) in &mut engines {
        let level = engine.level;
        if engine.shown.is_some_and(|old| (old - level).abs() < 0.01) {
            continue;
        }
        engine.shown = Some(level);
        if let Some(mut material) = materials.get_mut(&engine.glow) {
            material.base_color = color(level);
        }
        for (_, mut plume) in plumes
            .iter_mut()
            .filter(|(parent, _)| parent.parent() == hull)
        {
            plume.scale.z = 0.02 + PLUME_LENGTH * level;
        }
    }
}
