//! The machinery of the ship: the engines with their glow and exhaust, and the landing gear.

use bevy::prelude::*;

use super::frame::lathe;
use crate::ship::solid::Solid;

/// Places of the two engines on the tail: x and y of their axes.
const ENGINES: [Vec2; 2] = [Vec2::new(-0.52, 0.2), Vec2::new(0.52, 0.2)];

/// Where the engines end and the exhaust starts, along z.
pub const NOZZLE_Z: f32 = 3.7;

/// Height in metres of a gear strut above its foot: the nose strut ends in the fuselage and
/// the two others in the wings.
const STRUT: f32 = 1.12;

/// Returns the engines and their glowing ends. The front of each engine is inside the tail.
pub fn engines() -> [Solid; 2] {
    let (mut dark, mut glow) = (Solid::default(), Solid::default());
    let station = |z: f32, r: f32| [z, r, r, 0.0];
    let (front, back) = (Vec3::new(0.0, 0.0, 2.6), Vec3::new(0.0, 0.0, NOZZLE_Z));
    let mut engine = Solid::default();
    let shell = [station(2.8, 0.42), station(NOZZLE_Z, 0.34)];
    lathe(&mut engine, &shell, (front, back), 6);
    let mut disc = Solid::default();
    let end = [station(NOZZLE_Z + 0.01, 0.27)];
    lathe(&mut disc, &end, (back, back + Vec3::Z * 0.06), 6);
    for at in ENGINES {
        dark.merge(&engine, at.extend(0.0));
        glow.merge(&disc, at.extend(0.0));
    }
    [dark, glow]
}

/// Returns the strut and the pad of the gear leg that stands on `foot`.
pub fn leg(foot: Vec3) -> [Solid; 2] {
    let (mut strut, mut pad) = (Solid::default(), Solid::default());
    strut.slab(
        foot + Vec3::new(-0.07, 0.08, -0.07),
        foot + Vec3::new(0.07, STRUT, 0.07),
    );
    pad.slab(
        foot + Vec3::new(-0.26, 0.0, -0.32),
        foot + Vec3::new(0.26, 0.1, 0.32),
    );
    [strut, pad]
}

/// Builds the exhaust of both engines: unit cones that start at the nozzles and point back.
pub fn plume_mesh() -> Mesh {
    let mut plume = Solid::default();
    let mut cone = Solid::default();
    let mouth = [[0.0, 0.25, 0.25, 0.0]];
    lathe(&mut cone, &mouth, (Vec3::NEG_Z * 0.05, Vec3::Z), 6);
    for at in ENGINES {
        plume.merge(&cone, at.extend(0.0));
    }
    plume.into_mesh()
}
