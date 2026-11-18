//! Spawning the bodies of a star system: the star, every planet and moon with its halo, and
//! the rings of the planets that have them. Every solid body starts baking the ball of its
//! real surface, which replaces its plain sphere when it is done.

use bevy::camera::visibility::NoFrustumCulling;
use bevy::ecs::system::SystemParam;
use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use super::Body;
use super::bake;
use super::material::BodyMaterial;
use super::rings::{RingMaterial, annulus};
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

/// Returns the plain unit sphere of a star, a gas giant or a body whose ball is not baked yet:
/// white at every vertex, since the shader of the bodies reads a colour from every mesh.
fn plain() -> Mesh {
    let mut mesh = crate::render::shell(false);
    let white = vec![[1.0, 1.0, 1.0, 0.0]; mesh.count_vertices()];
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, white);
    mesh
}

/// Spawns `body` with its halo and its rings: a plain sphere `ball` in the flat colour of its
/// class, which the ball of its real surface replaces once baked.
fn planet(commands: &mut Commands, stores: &mut Stores, body: &PlanetSpec, ball: &Handle<Mesh>) {
    let lit = stores.bodies.add(BodyMaterial::planet(body));
    let halo = stores.halos.add(HaloMaterial::default());
    let mut planet = commands.spawn((Body::Planet(body.id), MeshMaterial3d(lit), common(ball)));
    if let Some(baking) = bake::start(body) {
        planet.insert(baking);
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

/// Replaces the bodies when the camera enters another star system, and starts baking the ball
/// of every solid one.
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
    let ball = stores.meshes.add(plain());
    let star = BodyMaterial::star(&local.system.star);
    commands.spawn((
        Body::Star,
        MeshMaterial3d(stores.bodies.add(star)),
        common(&ball),
    ));
    for body in local.bodies() {
        planet(&mut commands, &mut stores, body, &ball);
    }
}
