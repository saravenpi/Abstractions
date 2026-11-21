//! The tail: two engine pods that grow out of the sides of the tail cone, each ending in a
//! nozzle with a real bell: a collar where it leaves the pod, an outer wall, a lip, an inner
//! wall that flares to the exit, and the glowing face of the combustion chamber at the bottom.
//!
//! One main engine gives 220 kN at full throttle, half the 441 kN of three standard gravities
//! on 15 tonnes. At a chamber pressure of 7 MPa and a thrust coefficient of 1.6 that is a throat
//! of 0.0196 square metres, a radius of 8 cm. The exit is 30 cm in radius, an area of 0.283
//! square metres and an expansion ratio of 14.

use bevy::prelude::*;

use super::loft::{Section, annulus, cap, round, skin};
use crate::ship::solid::Solid;

/// Where the axes of the two engines are: x and y.
pub const ENGINES: [Vec2; 2] = [Vec2::new(-0.52, 0.2), Vec2::new(0.52, 0.2)];

/// Where the engines end and the exhaust starts, along z.
pub const NOZZLE_Z: f32 = 3.7;

/// Radius in metres of the exit of a nozzle.
pub const EXIT: f32 = 0.3;

/// Sides of a pod and of a bell.
const SIDES: u32 = 10;

/// Stations of a pod from its front, inside the tail cone, to its end: z and radius.
const POD: [(f32, f32); 5] = [
    (1.5, 0.2),
    (2.1, 0.4),
    (3.0, 0.46),
    (3.4, 0.43),
    (3.52, 0.36),
];

/// The inner wall of a bell from the chamber face to the exit: z and radius.
const BELL: [(f32, f32); 4] = [(3.42, 0.17), (3.52, 0.22), (3.62, 0.265), (NOZZLE_Z, EXIT)];

/// Thickness of the wall of a bell.
const WALL: f32 = 0.03;

/// The parts of one engine: the pod, the bell and the face of the chamber.
pub struct Motor {
    /// The pod, with its collar.
    pub pod: Solid,
    /// The bell.
    pub bell: Solid,
    /// The glowing face of the chamber.
    pub face: Solid,
}

/// Returns the ring of a station of an engine at `at`: z and radius.
fn station(at: Vec2, (z, radius): (f32, f32)) -> Section {
    round(at.extend(z), radius, SIDES)
}

/// Returns the motor on the axis `at`.
pub fn motor(at: Vec2) -> Motor {
    let sections: Vec<Section> = POD.iter().map(|&s| station(at, s)).collect();
    let inner: Vec<Section> = BELL.iter().map(|&s| station(at, s)).collect();
    let outer: Vec<Section> = BELL[1..]
        .iter()
        .map(|&(z, r)| station(at, (z, r + WALL)))
        .collect();
    let (mut pod, mut bell, mut face) = (Solid::default(), Solid::default(), Solid::default());
    skin(&mut pod, &sections, false, |_, _| 0);
    cap(&mut pod, &sections[0], Vec3::NEG_Z);
    annulus(&mut pod, &sections[sections.len() - 1], &outer[0], Vec3::Z);
    skin(&mut bell, &outer, false, |_, _| 0);
    annulus(
        &mut bell,
        &outer[outer.len() - 1],
        &inner[inner.len() - 1],
        Vec3::Z,
    );
    skin(&mut bell, &inner, true, |_, _| 0);
    cap(&mut face, &inner[0], Vec3::Z);
    Motor { pod, bell, face }
}

/// Returns the radius of the pod `z` metres along the ship, between its stations.
pub fn ring_radius(z: f32) -> f32 {
    let i = POD
        .iter()
        .rposition(|s| s.0 <= z)
        .unwrap_or(0)
        .min(POD.len() - 2);
    let (a, b) = (POD[i], POD[i + 1]);
    a.1 + (b.1 - a.1) * ((z - a.0) / (b.0 - a.0)).clamp(0.0, 1.0)
}

/// Returns the corner points of the pod on the axis `at`, for its collider.
pub fn corners(at: Vec2) -> Vec<Vec3> {
    POD.iter().flat_map(|&s| station(at, s)).collect()
}

/// Returns the ring of the outside of the lip of the nozzle on the axis `at`.
pub fn exit(at: Vec2) -> Section {
    station(at, (NOZZLE_Z, EXIT + WALL))
}
