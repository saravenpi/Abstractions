//! The ground under a foot, read from the world: built floors, wet earth, dry land, water to
//! wade in and the sea floor too deep to stand on.

use super::ground::{floor, natural};
use crate::audio::StepSurface;
use crate::world::Terrain;
use crate::world::structures::FloorKind;

/// Built floors give their own steps, and wet earth is mud.
#[test]
fn floors_map_to_steps() {
    assert_eq!(floor(FloorKind::Wood, 0.0), StepSurface::Wood);
    assert_eq!(floor(FloorKind::Stone, 0.0), StepSurface::Rock);
    assert_eq!(floor(FloorKind::Sand, 0.0), StepSurface::Sand);
    assert_eq!(floor(FloorKind::Earth, 0.0), StepSurface::Soil);
    assert_eq!(floor(FloorKind::Earth, 1.0), StepSurface::Mud);
}

/// The ground of the home world under the spawn gives a dry step, the sea floor far under the
/// water gives none, and a foot a little under the surface of the sea wades in water as deep
/// as it is.
#[test]
fn the_ground_is_read_from_the_world() {
    let terrain = Terrain::new(7);
    let spawn = terrain.find_spawn();
    let feet = |depth: f32| bevy::math::Vec3::new(spawn.x, -depth, spawn.z);
    let ground = bevy::math::Vec3::new(spawn.x, terrain.height(spawn.x, spawn.z), spawn.z);
    let (dry, depth) = natural(&terrain, ground, 0.0).expect("the spawn has ground");
    assert!(dry != StepSurface::Water && depth == 0.0, "{dry:?}");
    assert!(natural(&terrain, feet(5.0), 0.0).is_none());
    assert_eq!(
        natural(&terrain, feet(0.2), 0.0),
        Some((StepSurface::Water, 0.2))
    );
}
