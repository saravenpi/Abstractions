//! How much detail a body gets, by the size it covers on the screen: the sphere it is drawn on,
//! fine enough that its rim stays round, and the level of its baked surface, fine enough for one
//! texel or more per game pixel.

use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;

use super::Body;
use super::bake::Surface;
use super::material::BodyMaterial;
use super::sphere::{Spheres, level as sphere_level};
use crate::render::GameCamera;
use crate::render::vantage::Vantage;
use crate::world::globe::far::{LEAVE, SIDES};

/// Reference vertical game pixels used to budget baked planet surfaces and far spheres.
const BODY_ROWS: f32 = 360.0;

/// Vertical field of view in radians the detail is chosen for without a game camera: the default
/// of the settings, 75 degrees.
const FOV: f32 = 75.0 * std::f32::consts::PI / 180.0;

/// How many times finer than it needs now a level of the surface is asked for, so it is baked
/// before the body grows into it.
pub const AHEAD: f32 = 2.0;

/// How many times finer than it needs now a level of the surface is kept, so a body that hovers
/// at one size does not bake it again and again.
pub const KEEP: f32 = 8.0;

/// Distance from the centre of the body under the camera, in its radii, within which its surface
/// is not baked finer: there the world draws it, and the camera has time to climb to [`LEAVE`].
pub const BAKE_FROM: f32 = 1.2;

/// Returns the pixels per radian a sphere or baked surface budgets for at vertical FOV `fov`.
pub fn pixels_per_radian(fov: f32) -> f32 {
    BODY_ROWS / (2.0 * (0.5 * fov).tan())
}

/// Returns the texels a face of the surface of a body of `radius` needs along a side to give a
/// texel to every game pixel at `distance` from its centre, `ppr` pixels a radian: where the
/// surface is nearest, a pixel covers `(distance - radius) / ppr` of it, and a face spans a
/// quarter turn of the radius.
pub fn need(ppr: f32, radius: f32, distance: f32) -> f32 {
    FRAC_PI_2 * ppr * radius / (distance - radius).max(radius * 1.0e-3)
}

/// Returns the first level of [`SIDES`] at least `ahead` times as fine as `need`, or the last.
pub fn level(need: f32, ahead: f32) -> usize {
    let fits = |side: &usize| *side as f32 >= need * ahead;
    SIDES.iter().position(fits).unwrap_or(SIDES.len() - 1)
}

/// Returns the game pixels the rim of a ball covers from the middle of the view, for a ball whose
/// radius over its distance is `share`, `ppr` pixels a radian: the tangent of the angle the rim
/// stands at, which grows past the share as the ball comes near.
pub fn rim_pixels(ppr: f32, share: f32) -> f32 {
    let sine = share.clamp(0.0, 0.999);
    ppr * sine / (1.0 - sine * sine).sqrt()
}

/// Returns the pixels a radian covers in the game view now.
fn ppr(cameras: &Query<&Projection, With<GameCamera>>) -> f32 {
    let fov = cameras.iter().find_map(|projection| match projection {
        Projection::Perspective(perspective) => Some(perspective.fov),
        _ => None,
    });
    pixels_per_radian(fov.unwrap_or(FOV))
}

/// Gives every shown star, planet, moon, halo and cloud shell the sphere its size on the screen
/// needs, once it is built.
pub(super) fn shape(
    vantage: Res<Vantage>,
    cameras: Query<&Projection, With<GameCamera>>,
    mut spheres: ResMut<Spheres>,
    mut bodies: Query<(&Body, &Transform, &Visibility, &mut Mesh3d)>,
) {
    let ppr = ppr(&cameras);
    for (body, transform, seen, mut mesh) in &mut bodies {
        let shell = match body {
            Body::Rings(_) => continue,
            Body::Clouds(_) => true,
            Body::Star | Body::Planet(_) | Body::Halo(_) => false,
        };
        if *seen == Visibility::Hidden {
            continue;
        }
        let distance = (transform.translation - vantage.camera).length();
        let pixels = rim_pixels(ppr, transform.scale.x / distance.max(f32::MIN_POSITIVE));
        if let Some(handle) = spheres.pick(sphere_level(pixels), shell)
            && handle != mesh.0
        {
            mesh.0 = handle;
        }
    }
}

/// Asks every solid body for the level of its surface its size on the screen will soon need, never
/// nearer than [`LEAVE`] radii where the sky first draws it, and lets go of a level it has long
/// outgrown when a coarser one is held, its material drawing from the finest level left.
pub(super) fn sharpen(
    vantage: Res<Vantage>,
    cameras: Query<&Projection, With<GameCamera>>,
    mut bodies: Query<(&Transform, &mut Surface, &MeshMaterial3d<BodyMaterial>)>,
    mut materials: ResMut<Assets<BodyMaterial>>,
) {
    let ppr = ppr(&cameras);
    for (transform, mut surface, handle) in &mut bodies {
        let (radius, distance) = (
            transform.scale.x,
            (transform.translation - vantage.camera).length(),
        );
        if distance < BAKE_FROM * radius {
            continue;
        }
        let need = need(ppr, radius, distance.max(LEAVE as f32 * radius));
        surface.ask(level(need, AHEAD));
        let keep = level(need, KEEP);
        let coarser = surface.held.iter().take(keep + 1).any(Option::is_some);
        if coarser && surface.held.iter().skip(keep + 1).any(Option::is_some) {
            surface.keep(keep);
            if let Some(mut material) = materials.get_mut(&handle.0) {
                material.show(surface.finest());
            }
        }
    }
}
