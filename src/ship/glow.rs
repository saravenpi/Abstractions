//! The engine glow: the colour of the nozzles follows the throttle and the heat the engines
//! built up, and the warp drive its spool. Every hull has its own: the ship of this machine from
//! its flight, a puppet from its records. The exhaust itself is drawn by `exhaust`.

use bevy::prelude::*;

use super::body::Ship;
use super::exhaust::Exhaust;
use super::flight::BOOST;
use super::helm::Helm;
use super::warp::{SPOOL_TIME, Warp};
use crate::render::Sequence;

/// The glowing parts of a ship.
#[derive(Component, Debug, Clone)]
pub struct Engine {
    /// Material of the glowing chamber faces of this hull alone.
    pub glow: Handle<StandardMaterial>,
    /// Engine level to show, from 0 (idle) to 1 (full throttle) and above.
    pub level: f32,
    /// How far the warp drive is up, from 0 to 1: the share of its spool, then all of it.
    pub warp: f32,
    /// Engine level and heat the material shows, once written.
    shown: Option<(f32, f32)>,
}

impl Engine {
    /// Returns the engine of a hull glowing with `glow`, idle and not drawn yet.
    pub fn new(glow: Handle<StandardMaterial>) -> Self {
        Self {
            glow,
            level: 0.0,
            warp: 0.0,
            shown: None,
        }
    }
}

/// Linear color of the glow at full throttle: a blue far above 1, so that it blooms.
const FULL: Vec3 = Vec3::new(2.2, 5.6, 9.5);

/// Share of the full color an idle engine keeps.
const IDLE: f32 = 0.02;

/// Linear color of a nozzle at full heat with the engine off: dull red-orange, which cools to dark.
const EMBER: Vec3 = Vec3::new(1.4, 0.35, 0.06);

/// Level of the engines in warp.
const WARP_LEVEL: f32 = 1.6;

/// Returns the glow color for an engine level from 0 (idle) to 1 (full throttle) and above.
///
/// The material is unlit, so the color is what reaches the screen and values above 1 bloom.
pub fn color(level: f32) -> Color {
    let rgb = FULL * (IDLE + level.max(0.0));
    Color::linear_rgb(rgb.x, rgb.y, rgb.z)
}

/// Returns the color of the nozzles for an engine `level` and the `heat` the engine built up, from
/// 0 to 1: the blue of the thrust, and under it the ember of the metal, which cools after.
pub fn nozzle_color(level: f32, heat: f32) -> Color {
    let ember = EMBER * (heat * heat * (1.0 - level.min(1.0)));
    let rgb = FULL * (IDLE + level.max(0.0)) + ember;
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

/// Returns how far the warp drive of a ship is up: the share of its spool time done, and all of
/// it once it is engaged.
fn drive(warp: Warp) -> f32 {
    match warp {
        Warp::Off => 0.0,
        Warp::Spooling { left } => (1.0 - left / SPOOL_TIME).clamp(0.0, 1.0) as f32,
        Warp::Active { .. } => 1.0,
    }
}

/// Sets the engine level of the ship of this machine, and its warp. The warp sequence lights the
/// engines as the drive charges and lets them fade as it drops out.
pub fn throttle(
    (helm, sequence): (Res<Helm>, Option<Res<Sequence>>),
    mut ships: Query<(&Ship, &mut Engine)>,
) {
    let charged = sequence.as_deref().map_or(0.0, Sequence::thrust);
    for (ship, mut engine) in &mut ships {
        engine.level = level(ship, helm.controls.boost).max(WARP_LEVEL * charged);
        engine.warp = drive(ship.warp).max(charged);
    }
}

/// Sets the glow color of every hull whose engine level or nozzle heat changed.
pub fn glow(
    mut engines: Query<(&mut Engine, Option<&Exhaust>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (mut engine, exhaust) in &mut engines {
        let (level, heat) = (engine.level, exhaust.map_or(0.0, |exhaust| exhaust.heat));
        let moved = |(old_level, old_heat): (f32, f32)| {
            (old_level - level).abs() >= 0.01 || (old_heat - heat).abs() >= 0.01
        };
        if engine.shown.is_some_and(|old| !moved(old)) {
            continue;
        }
        engine.shown = Some((level, heat));
        if let Some(mut material) = materials.get_mut(&engine.glow) {
            material.base_color = nozzle_color(level, heat);
        }
    }
}
