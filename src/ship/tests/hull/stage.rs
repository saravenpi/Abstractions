//! A headless physics stage for the hull: static boxes standing for a house, a flat roof and the
//! canopy of the base, around the +Y pole of a planet, and a ship flown among them step by step.

use avian3d::prelude::*;
use bevy::math::DVec3;
use bevy::prelude::*;

use crate::ship::body::Ship;
use bevy::ecs::system::RunSystemOnce;

use crate::ship::flight::{Controls, Report, STEP, Scene};
use crate::ship::hull_cast::{Built, Solids};
use crate::ship::sim::stride;
use crate::space::{Anchor, Frame};
use crate::universe::PlanetSpec;

/// A solid box: its centre and its full size in metres, in render axes, +Y up at the pole.
pub type Block = (Vec3, Vec3);

/// The canopy of the station of the base: a slab 0.1 m thick on four thin posts, its top 2.6 m up.
pub const CANOPY: [Block; 5] = [
    (Vec3::new(0.0, 2.55, 0.0), Vec3::new(4.4, 0.1, 3.6)),
    (Vec3::new(-1.8, 1.25, -1.4), Vec3::new(0.12, 2.5, 0.12)),
    (Vec3::new(1.8, 1.25, -1.4), Vec3::new(0.12, 2.5, 0.12)),
    (Vec3::new(1.8, 1.25, 1.4), Vec3::new(0.12, 2.5, 0.12)),
    (Vec3::new(-1.8, 1.25, 1.4), Vec3::new(0.12, 2.5, 0.12)),
];

/// Height in metres above the sea of the top of [`CANOPY`].
pub const CANOPY_TOP: f64 = 2.6;

/// Returns the anchor of the stage: render origin at the sea level point of the +Y pole of
/// `planet`, render axes the frame axes.
pub fn anchor(planet: &PlanetSpec) -> Anchor {
    Anchor {
        frame: Frame::Surface(planet.id),
        origin: DVec3::Y * planet.radius,
        rotation: bevy::math::DQuat::IDENTITY,
        flat: false,
    }
}

/// Returns a headless physics app that holds `blocks` as static boxes, with the collider trees
/// already built.
pub fn stage(blocks: &[Block]) -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default(), TransformPlugin))
        .add_plugins(PhysicsPlugins::default())
        .init_asset::<Mesh>();
    app.finish();
    app.cleanup();
    for (at, size) in blocks {
        let solid = (RigidBody::Static, Collider::cuboid(size.x, size.y, size.z));
        app.world_mut()
            .spawn((solid, Transform::from_translation(*at)));
    }
    (0..3).for_each(|_| app.update());
    app
}

/// Flies `ship` over `planet`, whose ground is flat at `ground` metres above the sea, among the
/// solids of `app` for `seconds` with `controls`, the way the flight systems step it, and
/// returns the ship and the report after every step.
pub fn flown(
    app: &mut App,
    mut ship: Ship,
    controls: Controls,
    (planet, ground): (PlanetSpec, f64),
    seconds: f64,
) -> Vec<(Ship, Report)> {
    app.insert_resource(Built(Some(anchor(&planet))));
    let steps = (seconds / STEP).round() as usize;
    let fly = move |solids: Solids| {
        let land = move |_: DVec3| ground;
        let scene = Scene {
            planet: Some(&planet),
            ground: &land,
            wells: &[],
            elapsed: 0.0,
        };
        let mut each = || {
            let report = stride(&mut ship, &controls, &scene, Some(&solids));
            (ship.clone(), report)
        };
        (0..steps).map(|_| each()).collect::<Vec<_>>()
    };
    app.world_mut()
        .run_system_once(fly)
        .expect("the flight runs")
}
