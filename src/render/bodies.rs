//! The star, the planets and the moons of the current system, drawn as spheres in their true
//! directions with the detail their size on the screen needs, with the clouds and the rings of
//! the planets that have them.

mod bake;
mod giant;
mod light;
mod lod;
mod material;
mod rings;
mod spawn;
mod sphere;
mod surface;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_app;
#[cfg(test)]
mod tests_bake;
#[cfg(test)]
mod tests_giant;
#[cfg(test)]
mod tests_lod;
#[cfg(test)]
mod tests_sphere;
#[cfg(test)]
mod tests_star;
#[cfg(test)]
mod tests_surface;

use bevy::asset::embedded_asset;
use bevy::math::{DQuat, DVec3};
use bevy::pbr::MaterialPlugin;
use bevy::prelude::*;

pub use material::BodyMaterial;
pub use rings::RingMaterial;

use super::air::Air;
use super::atmosphere::halo_scale;
use super::vantage::{LocalSystem, Scene, Vantage};
use super::warp::Lens;
use crate::universe::{BodyId, PlanetSpec, system_reach};
use crate::world::Link;
use crate::world::globe::far::{handover, sky_from};

/// Angular radius in radians under which the disc of the star is not drawn: a ball under two pixels
/// across is not steady, and the point the star field draws for it is far wider and brighter.
pub const STAR_POINT: f32 = 0.0009;

/// Smallest angular radius in radians a planet is drawn with: about a pixel, a steady dot.
const PLANET_ANGLE: f32 = 0.0035;

/// Share of its radius the ground of the body under the camera is drawn at while the world may
/// still draw its quadtree: just under the sea and the lowest ground, so the quadtree covers it.
pub const SUNK: f64 = 0.998;

/// Times its hand-over within which the ground of the body under the camera stays sunk.
pub const SINK: f64 = 1.1;

/// Radii of a star that its warp bubble reaches. A star has no Hill sphere and does not move, so
/// this stands for it: its bubble is carried between the shares of it that planets use.
const STAR_REACH: f64 = 200.0;

/// A body of the current system drawn by the sky.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Body {
    /// The star.
    Star,
    /// A planet or a moon.
    Planet(BodyId),
    /// The halo of air around a planet or a moon.
    Halo(BodyId),
    /// The cloud shell around a solid planet or moon with clouds.
    Clouds(BodyId),
    /// The rings of a planet.
    Rings(BodyId),
}

/// A body ready to be placed, drawn at its true distance and size: the depth of the camera has no
/// far plane, so nothing is pulled in or shrunk.
struct Orb {
    pos: DVec3,
    turn: DQuat,
    radius: f64,
    carried: f64,
    least_angle: f32,
    shown: bool,
}

/// Returns the angular radius in radians of a ball of `radius` metres seen from `distance`
/// metres: the radius over the distance, the size the star is drawn with at any distance.
pub fn angle(radius: f64, distance: f64) -> f32 {
    (radius / distance.max(1.0e-9)) as f32
}

/// Returns the star of the current system ready to be placed: a disc of its true angular size,
/// which shrinks as the camera leaves it and is gone once it is under [`STAR_POINT`]. No star
/// is drawn larger than it is, so between the stars there is no sun.
fn star(scene: &Scene) -> Option<Orb> {
    let radius = scene.local.0.as_ref()?.system.star.radius;
    Some(Orb {
        pos: DVec3::ZERO,
        turn: DQuat::IDENTITY,
        radius,
        carried: radius * STAR_REACH,
        least_angle: 0.0,
        shown: angle(radius, scene.vantage.observer.eye.length()) >= STAR_POINT,
    })
}

impl Body {
    /// Returns where this body is in the star system, its size and whether the sky draws it now.
    fn orb(self, scene: &Scene) -> Option<Orb> {
        let id = match self {
            Body::Star => return star(scene),
            Body::Planet(id) | Body::Halo(id) | Body::Clouds(id) | Body::Rings(id) => id,
        };
        let (planet, pos) = scene.body(id)?;
        let size = angle(planet.radius, pos.distance(scene.vantage.observer.eye));
        let (radius, grow) = self.reach(planet, size)?;
        let radius = radius * self.sunk(planet, pos, &scene.vantage);
        let star = &scene.local.0.as_ref()?.system.star;
        let turn = match self {
            Body::Planet(_) => Link::orbiting(planet, scene.clock.seconds).turn,
            _ => DQuat::IDENTITY,
        };
        Some(Orb {
            pos,
            turn,
            radius,
            carried: planet.hill_radius(),
            least_angle: PLANET_ANGLE * grow,
            shown: self.shown(planet, pos, &scene.vantage, system_reach(star)),
        })
    }

    /// Returns the radius in metres of what this body draws around `planet`, whose radius covers
    /// `size` radians, and that radius over the radius of the planet, or nothing for the rings of
    /// a planet without rings.
    fn reach(self, planet: &PlanetSpec, size: f32) -> Option<(f64, f32)> {
        match self {
            Body::Halo(_) => Some(halo_scale(planet, size)).map(|k| (planet.radius * k as f64, k)),
            Body::Rings(_) => planet
                .rings
                .map(|rings| (rings.outer, (rings.outer / planet.radius) as f32)),
            Body::Clouds(_) => {
                bake::shell_radius(planet).map(|shell| (shell, (shell / planet.radius) as f32))
            }
            _ => Some((planet.radius, 1.0)),
        }
    }

    /// Returns the share of its radius this body of `planet`, at `pos` metres from the star, is
    /// drawn at for the camera `vantage`: [`SUNK`] for the ground of the body under the camera within
    /// [`SINK`] hand-overs, where the world draws its quadtree over it or dithers it out, else 1.
    fn sunk(self, planet: &PlanetSpec, pos: DVec3, vantage: &Vantage) -> f64 {
        let within = pos.distance(vantage.observer.eye) < SINK * handover(planet.radius);
        match self {
            Body::Planet(_) if within && vantage.is(planet.id) => SUNK,
            _ => 1.0,
        }
    }

    /// Returns whether the sky draws this body of `planet`, at `pos` metres from the star, now,
    /// for the camera `vantage` of a system that reaches `reach` metres out.
    ///
    /// The ground of the planet or moon below the camera shows from [`sky_from`], where the world
    /// may still draw the quadtree over it and draws nothing once it has dropped it, its clouds
    /// from its [`handover`], where the clouds of the world end, and its halo from above its air.
    /// A body without air has no halo. Rings show from everywhere, the deck of their own planet
    /// included. Nothing of a planet shows from outside its star system: from there it is far
    /// under a pixel, and the dot it is kept at inside would pile up with the others into a false
    /// light where the star is.
    fn shown(self, planet: &PlanetSpec, pos: DVec3, vantage: &Vantage, reach: f64) -> bool {
        let below = vantage.is(planet.id);
        if vantage.observer.eye.length() > reach {
            return false;
        }
        match self {
            Body::Halo(_) => {
                let airy = planet.atmosphere_height > 0.0 && Air::of(&planet.record).share() > 0.0;
                airy && (!below || vantage.air <= 0.0)
            }
            Body::Rings(_) => true,
            Body::Clouds(_) => {
                !below || pos.distance(vantage.observer.eye) > handover(planet.radius)
            }
            _ => !below || pos.distance(vantage.observer.eye) > sky_from(planet.radius),
        }
    }
}

/// Places every body where it is seen from the camera, never smaller than its least angle: in
/// its true direction at rest, and through the warp lens at speed, a halo and rings with their
/// planet. Rings lie in the plane of the equator, which is the plane of the system, and a
/// planet or a moon is turned as its day turns it, so its baked surface faces the right way.
fn follow(
    scene: Scene,
    lens: Res<Lens>,
    mut bodies: Query<(&Body, &mut Transform, &mut GlobalTransform, &mut Visibility)>,
) {
    let vantage = &scene.vantage;
    let observer = &vantage.observer;
    for (body, mut transform, mut global, mut visibility) in &mut bodies {
        let Some(orb) = body.orb(&scene) else {
            continue;
        };
        let seen = lens.body(observer.to_render * (orb.pos - observer.eye), orb.carried);
        let at = seen.offset.as_vec3();
        let size = (orb.radius as f32 * seen.scale).max(orb.least_angle * at.length());
        let turn = match body {
            Body::Rings(_) | Body::Planet(_) => (observer.to_render * orb.turn).as_quat(),
            _ => Quat::IDENTITY,
        };
        let pose = Transform::from_translation(vantage.camera + at).with_rotation(turn);
        super::pin(
            pose.with_scale(Vec3::splat(size)),
            &mut transform,
            &mut global,
        );
        visibility.set_if_neq(match orb.shown {
            true => Visibility::Visible,
            false => Visibility::Hidden,
        });
    }
}

/// Embeds the shaders, registers the materials and runs the bodies once the camera is tracked.
pub(super) fn plugin(app: &mut App) {
    embedded_asset!(app, "bodies.wesl");
    embedded_asset!(app, "rings.wesl");
    app.add_plugins((
        MaterialPlugin::<BodyMaterial>::default(),
        MaterialPlugin::<RingMaterial>::default(),
    ))
    .init_resource::<sphere::Spheres>()
    .add_systems(
        PostUpdate,
        (
            spawn::populate.run_if(resource_changed::<LocalSystem>),
            (bake::finish, sphere::finish),
            follow.after(super::warp::run),
            (lod::shape, lod::sharpen).after(follow),
            light::planets.after(follow),
            light::rings,
        )
            .after(super::sun::track),
    );
}
