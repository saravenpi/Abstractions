//! Body list tests in a headless app: a ringed giant, its moon and its rings are spawned with
//! the system, drawn where they truly are as time goes by, and the rings lie in its equator.

use bevy::asset::AssetPlugin;
use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::Body;
use super::material::BodyMaterial;
use crate::render::air::Air;
use crate::render::observer::Observer;
use crate::render::vantage::{LocalSystem, Nearby, Vantage};
use crate::render::warp::Lens;
use crate::space::Clock;
use crate::universe::{BodyId, PlanetClass, PlanetSpec, Rings, Universe};

/// Returns the home system of seed 7 with its home planet made a ringed giant with one moon.
fn system() -> (Nearby, PlanetSpec, PlanetSpec) {
    let universe = Universe::new(7);
    let home = universe.home();
    let mut near = Nearby::read(&universe, home.star);
    near.moons.retain(|moon| moon.id.index != home.index);
    let giant = &mut near.system.planets[usize::from(home.index)];
    giant.record.class = PlanetClass::GasGiant;
    giant.rings = Some(Rings {
        inner: 1.5 * giant.radius,
        outer: 2.5 * giant.radius,
        density: 0.7,
        color: Vec3::new(0.8, 0.7, 0.6),
    });
    let giant = giant.clone();
    let moon = PlanetSpec {
        id: BodyId {
            moon: 1,
            ..giant.id
        },
        radius: giant.radius * 0.25,
        orbit_radius: giant.radius * 12.0,
        orbit_seconds: 4000.0,
        rings: None,
        host: Some(giant.orbit()),
        ..giant.clone()
    };
    near.moons.push(moon.clone());
    (near, giant, moon)
}

/// Returns the camera 40 giant radii above the giant at `time`, render axes being system axes.
fn vantage(near: &Nearby, giant: &PlanetSpec, time: f64) -> Vantage {
    let center = giant.position(time);
    let observer = Observer {
        eye: center + DVec3::new(0.0, 40.0, 3.0).normalize() * giant.radius * 40.0,
        center,
        spin: DQuat::IDENTITY,
        to_render: DQuat::IDENTITY,
    };
    Vantage::of(&observer, near, giant, Vec3::ZERO)
}

/// Returns the transform of the body entity `body`, which must be shown.
fn shown(app: &mut App, body: Body) -> Transform {
    let mut query = app.world_mut().query::<(&Body, &Transform, &Visibility)>();
    let found = query.iter(app.world()).find(|(b, ..)| **b == body);
    let (_, pose, visibility) = found.expect("body spawned");
    assert_eq!(*visibility, Visibility::Visible, "{body:?}");
    *pose
}

/// Returns a headless app that draws the bodies of `near`.
fn sky(near: &Nearby, giant: &PlanetSpec) -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Mesh>()
        .insert_resource(Lens::default())
        .insert_resource(Clock::default())
        .insert_resource(vantage(near, giant, 0.0))
        .insert_resource(LocalSystem(Some(near.clone())))
        .add_plugins((crate::render::atmosphere::plugin, super::plugin));
    app
}

/// The giant, its moon and its rings are spawned and shown, the moon in the direction it truly
/// lies in as it goes round its planet, the rings centred on the giant, as large as they are
/// against it and turned with the system.
#[test]
fn moons_and_rings_follow_their_planet() {
    let (near, giant, moon) = system();
    let mut app = sky(&near, &giant);
    for time in [0.0, 1000.0, 2500.0] {
        let seen = vantage(&near, &giant, time);
        app.insert_resource(Clock { seconds: time });
        app.insert_resource(seen);
        app.update();
        app.update();
        let planet = shown(&mut app, Body::Planet(giant.id));
        let orbiter = shown(&mut app, Body::Planet(moon.id));
        let rings = shown(&mut app, Body::Rings(giant.id));
        let toward = (moon.position(time) - seen.observer.eye).normalize();
        let along = orbiter.translation.normalize().dot(toward.as_vec3());
        assert!(along > 0.99999, "{time}: {along}");
        assert_eq!(rings.translation, planet.translation);
        assert_eq!(rings.rotation, Quat::IDENTITY);
        assert!((rings.scale.x / planet.scale.x - 2.5).abs() < 1.0e-4);
    }
    let mut bodies = app.world_mut().query::<&Body>();
    let all: Vec<Body> = bodies.iter(app.world()).copied().collect();
    let ringed = near.bodies().filter(|body| body.rings.is_some()).count();
    let bodies = near.system.planets.len() + near.moons.len();
    assert_eq!(all.len(), 1 + 2 * bodies + ringed);
}

/// Every planet and moon is lit through its own air by its own star, from where that star
/// stands for it: its shader gets the block of its own place, and nothing of the camera's.
#[test]
fn every_body_is_lit_as_its_own_place() {
    let (near, giant, _) = system();
    let mut app = sky(&near, &giant);
    app.update();
    app.update();
    let star = near.system.star.color;
    let mut lit = app
        .world_mut()
        .query::<(&Body, &MeshMaterial3d<BodyMaterial>)>();
    let materials = app.world().resource::<Assets<BodyMaterial>>();
    let mut planets = 0;
    for (body, handle) in lit.iter(app.world()) {
        let Body::Planet(id) = *body else {
            continue;
        };
        let material = materials.get(&handle.0).expect("a body has its material");
        let spec = near.find(id).expect("a drawn body is in the system");
        let air = Air::of(&spec.record);
        let toward = material.body.light.truncate();
        assert_eq!(material.place.star, toward.extend(air.depth()), "{id:?}");
        assert_eq!(material.place.colour, star.extend(air.share()), "{id:?}");
        assert_eq!(material.place.air.x, spec.atmosphere_height as f32);
        assert_eq!(material.place.air.z, 1.0);
        planets += 1;
    }
    assert!(planets >= 2, "{planets}");
}
