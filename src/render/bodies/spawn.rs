//! Spawning the bodies of a star system: the star, every planet and moon with its halo and its
//! cloud shell, and the rings of the planets that have them. Every solid body starts baking the
//! texture of its real surface, which replaces the flat colour of its class when it is done.

use bevy::camera::visibility::NoFrustumCulling;
use bevy::ecs::system::SystemParam;
use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use super::Body;
use super::bake::{Surface, cloudy};
use super::material::BodyMaterial;
use super::rings::{RingMaterial, annulus};
use super::sphere::sphere;
use crate::render::atmosphere::HaloMaterial;
use crate::render::vantage::LocalSystem;
use crate::screens::Screen;
use crate::universe::PlanetSpec;

/// The asset stores the bodies are built in.
#[derive(SystemParam)]
pub(super) struct Stores<'w> {
    meshes: ResMut<'w, Assets<Mesh>>,
    bodies: ResMut<'w, Assets<BodyMaterial>>,
    halos: ResMut<'w, Assets<HaloMaterial>>,
    rings: ResMut<'w, Assets<RingMaterial>>,
}

/// Returns what every body entity carries besides its material: hidden until it is placed, out
/// of the shadow maps and never culled.
fn common(mesh: &Handle<Mesh>) -> impl Bundle {
    (
        Mesh3d(mesh.clone()),
        Transform::default(),
        Visibility::Hidden,
        NotShadowCaster,
        NoFrustumCulling,
        DespawnOnEnter(Screen::Title),
    )
}

/// Segments along a face of the spheres a body is drawn on until the sphere its size needs is
/// built: 3,072 triangles, which hug a ball up to a radius of about 180 pixels.
const FIRST: u32 = 16;

/// Spawns `body` with its halo, its cloud shell and its rings, on the first spheres `balls`: the
/// ground in the flat colour of its class, which the texture of its real surface replaces once
/// baked, and the clouds, which show from then on.
fn planet(
    commands: &mut Commands,
    stores: &mut Stores,
    body: &PlanetSpec,
    balls: &[Handle<Mesh>; 2],
) {
    let [ball, shell] = balls;
    let lit = stores.bodies.add(BodyMaterial::planet(body));
    let halo = stores.halos.add(HaloMaterial::default());
    let mut planet = commands.spawn((
        Body::Planet(body.id),
        MeshMaterial3d(lit.clone()),
        common(ball),
    ));
    if let Some(surface) = Surface::start(body) {
        planet.insert(surface);
    }
    if cloudy(body) {
        commands.spawn((Body::Clouds(body.id), MeshMaterial3d(lit), common(shell)));
    }
    commands.spawn((Body::Halo(body.id), MeshMaterial3d(halo), common(ball)));
    if let Some(rings) = &body.rings {
        let mesh = stores
            .meshes
            .add(annulus((rings.inner / rings.outer) as f32));
        let material = stores.rings.add(RingMaterial::of(body, rings));
        commands.spawn((
            Body::Rings(body.id),
            MeshMaterial3d(material),
            common(&mesh),
        ));
    }
}

/// Replaces the bodies when the camera enters another star system, and starts baking the
/// surface of every solid one.
pub(super) fn populate(
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
    let balls = [false, true].map(|shell| stores.meshes.add(sphere(FIRST, shell)));
    let star = BodyMaterial::star(&local.system.star);
    commands.spawn((
        Body::Star,
        MeshMaterial3d(stores.bodies.add(star)),
        common(&balls[0]),
    ));
    for body in local.bodies() {
        planet(&mut commands, &mut stores, body, &balls);
    }
}
