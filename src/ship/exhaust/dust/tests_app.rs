//! Tests of the dust in the setup of the game: the home planet of a seed as the game streams it,
//! with its life and its base, the wind, and a ship hovering on its lift nozzles. Puffs rise
//! within a second over the landing pad, over the ground of the base beside it and over sand.

use std::sync::Arc;
use std::time::Duration;

use bevy::ecs::system::RunSystemOnce;
use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::super::Exhaust;
use super::puff::Mote;
use super::{raise, spawn};
use crate::home::Base;
use crate::space::{Anchor, Frame, Placed};
use crate::universe::Universe;
use crate::wind::Wind;
use crate::world::{Globe, Surface, Terrain};

/// Seed of the world: the one the probe shoots the dust on.
const SEED: u64 = 57;

/// Returns the home planet of [`SEED`] as the game streams it, with its life and the ground of
/// its base reserved, and the base.
fn home() -> (Arc<Globe>, Base) {
    let universe = Universe::new(SEED);
    let body = universe.home();
    let spec = universe.planet(body);
    let life = crate::life::of(&spec, &universe.system(body.star).star);
    let globe = Arc::new(Globe::of(&spec).with_life(life));
    let base = Base::on(&globe, body);
    let reserved = Arc::new(globe.as_ref().clone().reserving(base.reserve()));
    (reserved, base)
}

/// Returns the first point of sand at least 0.3 m above the sea within 3 km of the pad, along one
/// of 32 directions.
fn sand(globe: &Globe, base: &Base) -> Option<DVec3> {
    (0..32).find_map(|k| {
        let turn = f64::from(k) * std::f64::consts::TAU / 32.0;
        let out = base.east * turn.cos() + base.north * turn.sin();
        (1..600)
            .map(|i| (base.pad + out * f64::from(i) * 5.0).normalize())
            .find(|dir| globe.surface(*dir) == Surface::Sand && globe.height(*dir) > 0.3)
    })
}

/// Returns a world with the home planet, its base, the wind and the pool of dust, and a ship
/// hovering on its lift nozzles with its hull at the planet position `hull`.
fn hovering(globe: &Arc<Globe>, base: &Base, hull: DVec3) -> App {
    let mut app = App::new();
    let up = hull.normalize();
    app.init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<StandardMaterial>>()
        .init_resource::<Time>()
        .init_resource::<Wind>()
        .insert_resource(*base)
        .insert_resource(Terrain::near(globe, base.pad))
        .insert_resource(Anchor {
            frame: Frame::Surface(base.body),
            origin: base.pad,
            rotation: DQuat::from_rotation_arc(base.pad.normalize(), DVec3::Y),
            flat: false,
        });
    let world = app.world_mut();
    world.run_system_once(spawn).expect("the pool of dust");
    let placed = Placed {
        pos: hull,
        rot: DQuat::from_rotation_arc(DVec3::Y, up),
    };
    let blowing = Exhaust {
        lift: 0.4,
        air: 1.0,
        ..Exhaust::default()
    };
    world.spawn((placed, blowing));
    app
}

/// Runs the dust of `app` for one second at ten frames a second and returns the puffs in the air.
fn puffs_after_a_second(mut app: App) -> usize {
    let world = app.world_mut();
    for _ in 0..10 {
        world
            .resource_mut::<Time>()
            .advance_by(Duration::from_millis(100));
        world.run_system_once(raise).expect("the dust raised");
    }
    world
        .query::<&Mote>()
        .iter(world)
        .filter(|mote| mote.life > 0.0)
        .count()
}

/// A ship hovering 5 m over the landing pad, over the ground of the base 30 m beside it and over
/// a beach near the base raises puffs within a second: a light grit off the pad, dust or grit
/// off the ground, and a full cloud off the sand.
#[test]
fn a_hover_at_5_m_raises_puffs_within_a_second() {
    let (globe, base) = home();
    let up = base.pad.normalize();
    let on_ground =
        |dir: DVec3| dir * (globe.radius() + f64::from(globe.height(dir).max(0.0)) + 5.0);
    let beside = (base.pad + base.east * 30.0).normalize();
    let beach = sand(&globe, &base).expect("sand within 3 km of the base");
    let spots = [
        ("pad", base.pad + up * 5.0),
        ("beside the pad", on_ground(beside)),
        ("sand", on_ground(beach)),
    ];
    for (name, hull) in spots {
        let count = puffs_after_a_second(hovering(&globe, &base, hull));
        assert!(count > 0, "no puff over the {name}");
    }
}
