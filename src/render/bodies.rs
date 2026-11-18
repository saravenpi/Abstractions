//! The star, the planets and the moons of the current system, drawn as spheres in their true
//! directions, with the rings of the planets that have them.

mod bake;
mod giant;
mod light;
mod material;
mod rings;
mod spawn;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_app;
#[cfg(test)]
mod tests_bake;
#[cfg(test)]
mod tests_giant;

use bevy::asset::embedded_asset;
use bevy::math::{DQuat, DVec3};
use bevy::pbr::MaterialPlugin;
use bevy::prelude::*;

pub use material::BodyMaterial;
pub use rings::RingMaterial;

use super::air::Air;
use super::atmosphere::HALO_SCALE;
use super::vantage::{LocalSystem, Scene, Vantage};
use super::warp::Lens;
use crate::universe::{BodyId, PlanetSpec};
use crate::world::Link;
use crate::world::globe::far::handover;

pub use crate::world::globe::far::FAR;

/// Smallest angular radius in radians the star is drawn with, the size of the old sun disc.
const STAR_ANGLE: f32 = 0.04;

/// Smallest angular radius in radians a planet is drawn with: about one pixel of the game view,
/// so a far planet stays a steady dot.
const PLANET_ANGLE: f32 = 0.0035;

/// A body of the current system drawn by the sky.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Body {
    /// The star.
    Star,
    /// A planet or a moon.
    Planet(BodyId),
    /// The halo of air around a planet or a moon.
    Halo(BodyId),
    /// The rings of a planet.
    Rings(BodyId),
}

/// Where a body is drawn: an offset from the camera and the factor its size shrinks by.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Drawn {
    /// Offset from the camera in metres, in the axes of the true offset.
    pub at: Vec3,
    /// Drawn size over true size.
    pub scale: f32,
}

/// Returns where to draw a body that truly sits `offset` metres from the camera.
///
/// Up to [`FAR`] it stays where it is. Beyond, it is pulled in along its direction to between one
/// and two times [`FAR`], farther bodies staying behind nearer ones, and shrunk by the same factor
/// so that it covers the same angle. That keeps 32-bit positions and the depth range exact.
pub fn place(offset: DVec3) -> Drawn {
    let (far, distance) = (f64::from(FAR), offset.length());
    if distance <= far {
        return Drawn {
            at: offset.as_vec3(),
            scale: 1.0,
        };
    }
    let scale = far * (2.0 - far / distance) / distance;
    Drawn {
        at: (offset * scale).as_vec3(),
        scale: scale as f32,
    }
}
/// A body ready to be placed.
struct Orb {
    pos: DVec3,
    turn: DQuat,
    radius: f64,
    solid: f64,
    least_angle: f32,
    shown: bool,
}

/// Returns the star of the current system ready to be placed.
fn star(scene: &Scene) -> Option<Orb> {
    let radius = scene.local.0.as_ref()?.system.star.radius;
    Some(Orb {
        pos: DVec3::ZERO,
        turn: DQuat::IDENTITY,
        radius,
        solid: radius,
        least_angle: STAR_ANGLE,
        shown: true,
    })
}

impl Body {
    /// Returns where this body is in the star system, its size and whether the sky draws it now.
    fn orb(self, scene: &Scene) -> Option<Orb> {
        let id = match self {
            Body::Star => return star(scene),
            Body::Planet(id) | Body::Halo(id) | Body::Rings(id) => id,
        };
        let (planet, pos) = scene.body(id)?;
        let (radius, grow) = self.reach(planet)?;
        let turn = match self {
            Body::Planet(_) => Link::orbiting(planet, scene.clock.seconds).turn,
            _ => DQuat::IDENTITY,
        };
        Some(Orb {
            pos,
            turn,
            radius,
            solid: planet.radius,
            least_angle: PLANET_ANGLE * grow,
            shown: self.shown(planet, pos, &scene.vantage),
        })
    }

    /// Returns the radius in metres of what this body draws around `planet`, and that radius
    /// over the radius of the planet, or nothing for the rings of a planet without rings.
    fn reach(self, planet: &PlanetSpec) -> Option<(f64, f32)> {
        match self {
            Body::Halo(_) => Some((planet.radius * f64::from(HALO_SCALE), HALO_SCALE)),
            Body::Rings(_) => planet
                .rings
                .map(|rings| (rings.outer, (rings.outer / planet.radius) as f32)),
            _ => Some((planet.radius, 1.0)),
        }
    }

    /// Returns whether the sky draws this body of `planet`, at `pos` metres from the star, now.
    ///
    /// The planet or moon below the camera is left to the terrain within its [`handover`],
    /// and its halo only shows from above its air. A body without air has no halo. Rings show
    /// from everywhere, the deck of their own planet included.
    fn shown(self, planet: &PlanetSpec, pos: DVec3, vantage: &Vantage) -> bool {
        let below = vantage.is(planet.id);
        match self {
            Body::Halo(_) => {
                let airy = planet.atmosphere_height > 0.0 && Air::of(&planet.record).share() > 0.0;
                airy && (!below || vantage.air <= 0.0)
            }
            Body::Rings(_) => true,
            _ => {
                !below || pos.distance(vantage.observer.eye) > handover(planet.radius, planet.rings)
            }
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
        let seen = lens.body(observer.to_render * (orb.pos - observer.eye), orb.solid);
        let drawn = place(seen.offset);
        let size = orb.radius as f32 * drawn.scale * seen.scale;
        let size = size.max(orb.least_angle * drawn.at.length());
        let turn = match body {
            Body::Rings(_) | Body::Planet(_) => (observer.to_render * orb.turn).as_quat(),
            _ => Quat::IDENTITY,
        };
        let pose = Transform::from_translation(vantage.camera + drawn.at).with_rotation(turn);
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
    .add_systems(
        PostUpdate,
        (
            spawn::populate.run_if(resource_changed::<LocalSystem>),
            bake::finish,
            follow.after(super::warp::run),
            light::planets,
            light::rings,
        )
            .after(super::sun::track),
    );
}
