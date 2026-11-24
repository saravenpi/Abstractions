//! Verifies that ship hull casts collide with remote ships marked `OtherHull` and ignore
//! unmarked kinematic bodies.

use avian3d::prelude::*;
use bevy::ecs::system::RunSystemOnce;
use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use crate::ship::OtherHull;
use crate::ship::airframe;
use crate::ship::hull_cast::{Built, Solids};
use crate::space::{Anchor, Frame};
use crate::universe::{PlanetSpec, Universe};

/// Verifies `Solids::cast` hits a kinematic ship marked with `OtherHull` and ignores unmarked ones.
#[test]
fn hull_cast_hits_other_hull_and_skips_unmarked_kinematic() {
    let universe = Universe::new(7);
    let planet: PlanetSpec = universe.planet(universe.home());
    let anchor = Anchor {
        frame: Frame::Surface(planet.id),
        origin: DVec3::Y * planet.radius,
        rotation: DQuat::IDENTITY,
        flat: false,
    };
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default(), TransformPlugin))
        .add_plugins(PhysicsPlugins::default())
        .insert_resource(Built(Some(anchor)));
    app.finish();
    app.cleanup();
    app.world_mut().spawn((
        RigidBody::Kinematic,
        airframe(),
        Transform::from_xyz(0.0, 2.0, -14.0),
    ));
    app.world_mut().spawn((
        RigidBody::Kinematic,
        airframe(),
        OtherHull,
        Transform::from_xyz(0.0, 2.0, -30.0),
    ));
    (0..3).for_each(|_| app.update());
    let cast = move |solids: Solids| {
        let start = DVec3::Y * (planet.radius + 2.0);
        solids.cast(
            Frame::Surface(planet.id),
            (start, DQuat::IDENTITY),
            DVec3::NEG_Z * 40.0,
            false,
        )
    };
    let hit = app
        .world_mut()
        .run_system_once(cast)
        .expect("cast ran")
        .expect("remote ship hull hit");
    assert!(
        hit.distance > 18.0 && hit.distance < 32.0,
        "dist={}",
        hit.distance
    );
}
