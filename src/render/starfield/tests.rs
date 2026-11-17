//! Starfield tests: the mesh faces the camera and the stars come from the real universe.

use std::time::{Duration, Instant};

use bevy::math::DVec3;
use bevy::mesh::{Indices, VertexAttributeValues};
use bevy::prelude::*;

use super::Starfield;
use super::build::{Building, adopt, gather, launch};
use super::specks::{Speck, mesh, star_speck};
use crate::render::vantage::{LocalSystem, Nearby, Vantage};
use crate::space::Clock;
use crate::universe::{GLY, GalaxyId, StarId, Universe};

/// The starfield mesh has one inward-facing quad per speck and stays out of the ray tracing scene.
#[test]
fn starfield_faces_the_camera() {
    let dirs = [Vec3::X, Vec3::NEG_Y, Vec3::new(0.6, 0.0, -0.8)];
    let speck = |dir| Speck {
        dir,
        size: 0.01,
        color: Vec3::ONE,
        soft: 1.0,
    };
    let built = mesh(&dirs.map(speck));
    let Some(VertexAttributeValues::Float32x3(points)) = built.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        panic!("positions missing");
    };
    let Some(Indices::U32(indices)) = built.indices() else {
        panic!("indices missing");
    };
    assert_eq!((points.len(), indices.len()), (12, 18));
    assert!(!built.enable_raytracing);
    for corner in indices.chunks(3) {
        let [a, b, c] = [0, 1, 2].map(|i| Vec3::from(points[corner[i] as usize]));
        let inward = (b - a)
            .cross(c - a)
            .normalize()
            .dot(-dirs[corner[0] as usize / 4]);
        assert!(inward > 0.999, "{corner:?}");
    }
}

/// Nearer stars are brighter, and the stars gathered around the home star leave out that star itself.
#[test]
fn stars_come_from_the_universe() {
    let universe = Universe::new(11);
    let star = universe.system(universe.home().star).star;
    let near = star_speck(&star, star.pos + DVec3::X * 2.0 * GLY);
    let far = star_speck(&star, star.pos + DVec3::X * 30.0 * GLY);
    assert!((near.dir - Vec3::NEG_X).length() < 1e-5);
    assert!(near.size > far.size && near.color.length() > far.color.length());
    let seen = gather(&universe, star.id.galaxy, star.pos, star.pos);
    let others = universe.stars_near(star.id.galaxy, star.pos, 40.0).len() - 1;
    assert!(seen.len() >= others.min(4000));
    assert!(
        seen.iter()
            .all(|speck| speck.dir.is_normalized() && speck.size > 0.0)
    );
}

/// Returns an app that builds the starfield of a camera at `place`, and its starfield entity.
fn sky_at(place: (GalaxyId, DVec3), star: StarId) -> (App, Entity) {
    let universe = Universe::new(7);
    let vantage = Vantage {
        galaxy: place.0,
        galactic: place.1,
        ..Vantage::default()
    };
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Clock>()
        .insert_resource(universe)
        .insert_resource(LocalSystem(Some(Nearby::read(&universe, star))))
        .insert_resource(vantage)
        .add_systems(Update, (adopt, launch).chain());
    let field = (Starfield::default(), Visibility::Hidden);
    let field = app.world_mut().spawn(field).id();
    (app, field)
}

/// The mesh is built off the main thread: the frame that asks for it gets none, a later frame
/// swaps it in, and no second build starts while the camera stays where it was.
#[test]
fn starfield_is_built_in_the_background() {
    let universe = Universe::new(7);
    let star = universe.system(universe.home().star).star;
    let place = (star.id.galaxy, star.pos);
    let (mut app, field) = sky_at(place, star.id);
    app.update();
    let world = app.world();
    assert!(world.get::<Building>(field).is_some() && world.get::<Mesh3d>(field).is_none());
    let start = Instant::now();
    while app.world().get::<Mesh3d>(field).is_none() {
        assert!(start.elapsed() < Duration::from_secs(60), "never built");
        std::thread::sleep(Duration::from_millis(2));
        app.update();
    }
    app.update();
    let world = app.world();
    assert_eq!(world.get::<Starfield>(field).unwrap().built, Some(place));
    assert!(
        world.get::<Building>(field).is_none(),
        "a second build started"
    );
    assert_eq!(world.get::<Visibility>(field), Some(&Visibility::Visible));
}
