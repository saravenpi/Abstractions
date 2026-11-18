//! Tests of the cloud crossing: the camera is in cloud only where the field holds a cloud, in
//! the rule and in the system that runs it over the home planet.

use std::time::Duration;

use bevy::ecs::system::SystemId;
use bevy::math::DVec3;
use bevy::prelude::*;
use bevy_persistent::StorageFormat;

use super::cross::{EASE, crossing_depth, eased};
use super::{CELL, CloudCrossing, CloudMaps, CloudMaterial};
use crate::render::observer::Observer;
use crate::render::palette::Palette;
use crate::render::vantage::{LocalSystem, Nearby, Vantage};
use crate::settings::{Persistent, Settings};
use crate::space::Clock;
use crate::universe::{PlanetSpec, Universe};
use crate::weather::{Weather, density_at};

/// Universe second the system tests stand at.
const SECOND: f64 = 5000.0;

/// The camera is in cloud only where the field holds a cloud: in clear air the depth stays
/// zero in the very middle of the band, the edge of a puff comes in over about a third of a
/// second without a flash at any frame rate, and out of the band nothing lingers.
#[test]
fn the_whiteout_needs_a_real_cloud() {
    let band = crossing_depth(1200.0, 900.0, 1500.0);
    assert_eq!(band, 1.0);
    assert_eq!(band * eased(0.0, 0.0, band, 0.016), 0.0);
    for rate in [30.0_f32, 60.0, 144.0] {
        let (mut around, mut worst) = (0.0_f32, 0.0_f32);
        for _ in 0..(EASE * rate) as u32 {
            let next = eased(around, 1.0, band, 1.0 / rate);
            (worst, around) = (worst.max(next - around), next);
        }
        assert!(
            worst < 0.12 && (0.55..0.7).contains(&around),
            "{rate}: {worst} {around}"
        );
    }
    assert_eq!(eased(1.0, 0.0, 0.0, 0.016), 0.0);
    assert_eq!(eased(0.4, 0.4, 1.0, 0.016), 0.4);
}

/// Returns default settings stored in a scratch file of their own.
fn settings(name: &str) -> Persistent<Settings> {
    let dir = format!("abstractions-clouds-test-{}-{name}", std::process::id());
    Persistent::<Settings>::builder()
        .name("settings")
        .format(StorageFormat::Toml)
        .path(std::env::temp_dir().join(dir).join("settings.toml"))
        .default(Settings::default())
        .build()
        .unwrap()
}

/// Returns the home planet of universe 7 and a world holding what the cloud layer reads, the
/// camera `altitude` metres above the sea of that planet in its direction `dir`, under an
/// overcast sky.
fn world(name: &str, dir: DVec3, altitude: f64) -> (PlanetSpec, World) {
    let universe = Universe::new(7);
    let home = universe.planet(universe.home());
    let near = Nearby::read(&universe, home.id.star);
    let observer = Observer {
        eye: dir * (home.radius + altitude),
        center: DVec3::ZERO,
        ..default()
    };
    let mut world = World::new();
    world.insert_resource(Vantage::of(&observer, &near, &home, Vec3::ZERO));
    world.insert_resource(LocalSystem(Some(near)));
    world.insert_resource(Palette::at(0.25));
    world.insert_resource(Clock { seconds: SECOND });
    world.insert_resource(settings(name));
    world.insert_resource(Time::<()>::default());
    world.insert_resource(Weather {
        cloud: 1.0,
        ..default()
    });
    world.init_resource::<CloudMaps>();
    world.init_resource::<CloudCrossing>();
    world.init_resource::<Assets<CloudMaterial>>();
    (home, world)
}

/// Runs the cloud layer of `world` for two seconds at sixty frames per second and returns
/// the depth of the crossing it ends at.
fn settle(world: &mut World, layer: SystemId) -> f32 {
    for _ in 0..120 {
        world
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f64(1.0 / 60.0));
        world.run_system(layer).unwrap();
    }
    world.resource::<CloudCrossing>().depth
}

/// Over the home planet, in the middle of the cloud band under an overcast sky, the camera is
/// deep in cloud where the field holds a puff and in clear air where it holds a gap. Above the
/// band, and with clouds switched off, it is never in cloud.
#[test]
fn the_layer_whites_out_inside_a_puff_and_not_in_a_gap() {
    let (home, _) = world("probe", DVec3::X, 0.0);
    let mid = 0.5 * (home.cloud_base + home.cloud_top);
    let cells = (home.radius + mid) as f32 / CELL;
    let ring =
        (0..2000).map(|i| DVec3::new(f64::from(i).cos(), 0.3, f64::from(i).sin()).normalize());
    let cloud = |dir: &DVec3| density_at(*dir, cells, SECOND, 1.0);
    let puff = ring.clone().find(|dir| cloud(dir) == 1.0).unwrap();
    let gap = ring.clone().find(|dir| cloud(dir) == 0.0).unwrap();
    for (dir, altitude, off, wanted) in [
        (puff, mid, false, 0.95..=1.0),
        (gap, mid, false, 0.0..=0.0),
        (puff, home.cloud_top + 50.0, false, 0.0..=0.0),
        (puff, mid, true, 0.0..=0.0),
    ] {
        let (_, mut world) = world("layer", dir, altitude);
        world.resource_mut::<Persistent<Settings>>().graphics.clouds = !off;
        let layer = world.register_system(super::drift);
        let depth = settle(&mut world, layer);
        assert!(
            wanted.contains(&depth),
            "{depth} at {altitude} m, off {off}"
        );
    }
}
