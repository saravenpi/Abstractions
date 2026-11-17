//! The glow: the face colour on the screen, the antenna and the thruster, and the weak light they cast.

use std::f32::consts::PI;

use bevy::prelude::*;
use bevy::solari::prelude::RaytracingMesh3d;

use super::Companion;
use super::face::machine::Face;
use super::motion::blend;
use super::orb::{orb, orb_area};
use super::screen::Display;
use super::sense::CompanionSense;
use super::tune::{GLOW_HDR, LIGHT_DAY_SHARE, LIGHT_POWER, SCREEN_HDR};
use crate::render::RayTracing;

/// How fast the colour follows the expression, per second.
const TINT_RATE: f32 = 6.0;

/// Radius in meters of the emitter that stands for the light in the ray traced scene.
pub const EMITTER_RADIUS: f32 = 0.05;

/// Marks the point light in front of the screen.
#[derive(Component, Debug, Clone, Copy)]
pub struct Lamp;

/// Marks the entity that carries the ray traced emitter while ray tracing is on.
#[derive(Component, Debug, Clone, Copy)]
pub struct Emitter;

/// Returns the light power in lumens for a night level from 0 (day) to 1 (night).
pub fn lumens(night: f32) -> f32 {
    LIGHT_POWER * (LIGHT_DAY_SHARE + (1.0 - LIGHT_DAY_SHARE) * night.clamp(0.0, 1.0))
}

/// Returns the radiance in nits the emitter needs to give off `lumens` in the colour `tint`.
///
/// A convex emitter of area `A` matches a point light of `P` lumens at `P / (pi * A)` nits.
pub fn radiance(tint: Vec3, lumens: f32) -> LinearRgba {
    let nits = lumens / (PI * orb_area(EMITTER_RADIUS));
    LinearRgba::rgb(tint.x * nits, tint.y * nits, tint.z * nits)
}

/// Returns the unlit colour of `tint` at `gain` times white.
fn hdr(tint: Vec3, gain: f32) -> Color {
    Color::linear_rgb(tint.x * gain, tint.y * gain, tint.z * gain)
}

/// Eases the colour toward the expression's and applies it to the screen, the glowing parts and the light.
///
/// The light has no shadows and weakens by day. Its ray traced twin gets the same power and colour.
pub fn shine(
    time: Res<Time>,
    sense: Res<CompanionSense>,
    bot: Single<(&Face, &mut Display), With<Companion>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut lamp: Single<&mut PointLight, With<Lamp>>,
) {
    let (face, mut display) = bot.into_inner();
    let expr = face.shown();
    let want = expr.tint() * expr.glow();
    let tint = display.tint.lerp(want, blend(TINT_RATE, time.delta_secs()));
    let power = lumens(sense.night);
    lamp.color = hdr(tint, 1.0);
    lamp.intensity = power;
    if tint.distance(display.tint) < 1e-4 && (power - display.power).abs() < 1.0 {
        return;
    }
    (display.tint, display.power) = (tint, power);
    let paints = [
        (&display.face, hdr(tint, SCREEN_HDR), LinearRgba::BLACK),
        (&display.glow, hdr(tint, GLOW_HDR), LinearRgba::BLACK),
        (&display.emitter, Color::BLACK, radiance(tint, power)),
    ];
    for (handle, base, emissive) in paints {
        if let Some(mut material) = materials.get_mut(handle) {
            material.base_color = base;
            material.emissive = emissive;
        }
    }
}

/// Puts the emitter in the ray traced scene while ray tracing is on, and takes it out when it is off.
///
/// Solari has no point lights and ignores visibility, so the light exists there only through this mesh.
pub fn trace(
    mut commands: Commands,
    ray_tracing: Res<RayTracing>,
    mut meshes: ResMut<Assets<Mesh>>,
    display: Single<&Display, With<Companion>>,
    emitter: Single<(Entity, Has<RaytracingMesh3d>), With<Emitter>>,
) {
    let (entity, traced) = *emitter;
    if ray_tracing.0 && !traced {
        let mesh = meshes.add(orb(EMITTER_RADIUS, true));
        let material = MeshMaterial3d(display.emitter.clone());
        commands
            .entity(entity)
            .try_insert((RaytracingMesh3d(mesh), material));
    } else if traced && !ray_tracing.0 {
        commands
            .entity(entity)
            .try_remove::<(RaytracingMesh3d, MeshMaterial3d<StandardMaterial>)>();
    }
}
