//! The deck of a gas giant: nothing puts the body there, neither the ship hand-over nor the rescue.

use std::sync::Arc;

use bevy::math::DVec3;
use bevy::prelude::*;

use super::tangent_anchor;
use crate::player::spawn::{Home, rescue};
use crate::player::{Motion, Player, place};
use crate::universe::{BodyId, PlanetClass, PlanetRecord};
use crate::world::globe::Face;
use crate::world::{Globe, Planet, Terrain};

/// Returns a 3 km gas giant.
fn giant() -> Arc<Globe> {
    let record = PlanetRecord {
        class: PlanetClass::GasGiant,
        ..PlanetRecord::TEMPERATE
    };
    Arc::new(Globe::new(3, super::RADIUS, 9.8).with_record(record))
}

/// The ship cannot stand the body on a giant, and a body there is never rescued onto its deck.
#[test]
fn nothing_stands_the_body_on_a_giant() {
    let anchor = tangent_anchor(DVec3::Y);
    let globe = giant();
    let mut world = World::new();
    world.insert_resource(anchor);
    world.insert_resource(Planet {
        body: BodyId::default(),
        globe: globe.clone(),
        day_seconds: 600.0,
    });
    let start = Transform::from_xyz(4.0, 900.0, 2.0);
    let body = world
        .spawn((Player, start, Motion::default(), Home(DVec3::ZERO)))
        .id();
    place(&mut world, Vec3::new(18.0, 0.4, -9.0), Vec3::X);
    assert_eq!(*world.get::<Transform>(body).unwrap(), start);
    let terrain = Terrain::on(globe.clone(), globe.face_chart(Face::PosY));
    for height in [-400.0, -3.0, 0.5, 50.0] {
        let at = Vec3::Y * height;
        assert_eq!(rescue(&anchor, &terrain, at, DVec3::Y * 4000.0), None);
    }
}
