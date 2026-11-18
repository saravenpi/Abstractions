//! The star and the planets of the current system, drawn as spheres in their true directions.

mod material;
#[cfg(test)]
mod tests;

use bevy::asset::embedded_asset;
use bevy::camera::visibility::NoFrustumCulling;
use bevy::ecs::system::SystemParam;
use bevy::light::NotShadowCaster;
use bevy::math::DVec3;
use bevy::pbr::MaterialPlugin;
use bevy::prelude::*;

pub use material::BodyMaterial;

use super::air::Air;
use super::atmosphere::{HALO_SCALE, HaloMaterial};
use super::vantage::{LocalSystem, Scene};
use super::warp::Lens;
use crate::screens::Screen;

/// Distance in metres past which a body is pulled in: farther than anything the terrain streams.
pub const FAR: f32 = 1.0e6;

/// Distance in planet radii up to which the planet below is drawn by the terrain, not here.
const HANDOVER: f64 = 20.0;

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
    /// The planet of this rank.
    Planet(usize),
    /// The halo of air around the planet of this rank.
    Halo(usize),
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
    radius: f64,
    solid: f64,
    least_angle: f32,
    shown: bool,
}

impl Body {
    /// Returns where this body is in the star system, its size and whether the sky draws it now.
    ///
    /// The planet below the camera is left to the terrain within [`HANDOVER`] radii, and its halo
    /// only shows from above its air. A planet without air has no halo.
    fn orb(self, scene: &Scene) -> Option<Orb> {
        let vantage = &scene.vantage;
        let (index, halo) = match self {
            Body::Star => {
                return Some(Orb {
                    pos: DVec3::ZERO,
                    radius: scene.local.0.as_ref()?.system.star.radius,
                    solid: scene.local.0.as_ref()?.system.star.radius,
                    least_angle: STAR_ANGLE,
                    shown: true,
                });
            }
            Body::Planet(index) => (index, false),
            Body::Halo(index) => (index, true),
        };
        let (planet, pos) = scene.planet(index)?;
        let below = index == vantage.body;
        let away = pos.distance(vantage.observer.eye) > HANDOVER * planet.radius;
        let grow = if halo { HALO_SCALE } else { 1.0 };
        let airy = planet.atmosphere_height > 0.0 && Air::of(&planet.record).share() > 0.0;
        Some(Orb {
            pos,
            radius: planet.radius * f64::from(grow),
            solid: planet.radius,
            least_angle: PLANET_ANGLE * grow,
            shown: match halo {
                true => airy && (!below || vantage.air <= 0.0),
                false => !below || away,
            },
        })
    }
}

/// The asset stores the bodies are built in.
#[derive(SystemParam)]
struct Stores<'w> {
    meshes: ResMut<'w, Assets<Mesh>>,
    bodies: ResMut<'w, Assets<BodyMaterial>>,
    halos: ResMut<'w, Assets<HaloMaterial>>,
}

/// Replaces the bodies when the camera enters another star system.
fn populate(
    mut commands: Commands,
    local: Res<LocalSystem>,
    old: Query<Entity, With<Body>>,
    mut stores: Stores,
) {
    for entity in &old {
        commands.entity(entity).despawn();
    }
    let Some(local) = local.0.as_ref() else {
        return;
    };
    let ball = stores.meshes.add(super::shell(false));
    let common = || {
        (
            Mesh3d(ball.clone()),
            Transform::default(),
            Visibility::Hidden,
            NotShadowCaster,
            NoFrustumCulling,
            DespawnOnEnter(Screen::Title),
        )
    };
    let star = BodyMaterial::star(&local.system.star);
    commands.spawn((
        Body::Star,
        MeshMaterial3d(stores.bodies.add(star)),
        common(),
    ));
    for (index, planet) in local.system.planets.iter().enumerate() {
        let lit = stores.bodies.add(BodyMaterial::planet(planet));
        let halo = stores.halos.add(HaloMaterial::default());
        commands.spawn((Body::Planet(index), MeshMaterial3d(lit), common()));
        commands.spawn((Body::Halo(index), MeshMaterial3d(halo), common()));
    }
}

/// Places every body where it is seen from the camera, never smaller than its least angle: in
/// its true direction at rest, and through the warp lens at speed, a halo with its planet.
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
        let pose = Transform::from_translation(vantage.camera + drawn.at);
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

/// Lights every planet from the direction of its star.
fn light_planets(
    scene: Scene,
    planets: Query<(&Body, &MeshMaterial3d<BodyMaterial>)>,
    mut materials: ResMut<Assets<BodyMaterial>>,
) {
    for (body, handle) in &planets {
        let Body::Planet(index) = *body else {
            continue;
        };
        let Some((_, pos)) = scene.planet(index) else {
            continue;
        };
        if let Some(mut material) = materials.get_mut(&handle.0) {
            material.body.light = scene.vantage.sunward(pos).extend(0.0);
        }
    }
}

/// Embeds the shader, registers the material and runs the bodies once the camera is tracked.
pub(super) fn plugin(app: &mut App) {
    embedded_asset!(app, "bodies.wesl");
    app.add_plugins(MaterialPlugin::<BodyMaterial>::default())
        .add_systems(
            PostUpdate,
            (
                populate.run_if(resource_changed::<LocalSystem>),
                follow.after(super::warp::run),
                light_planets,
            )
                .after(super::sun::track),
        );
}
