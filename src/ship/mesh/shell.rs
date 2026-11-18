//! The hull and the glass as they are drawn: the outline of `frame`, open over the cockpit so
//! that the tub, the seat and the pilot inside are seen from outside and from the seat.

use bevy::prelude::*;

use super::frame::{CANOPY, CANOPY_ENDS, CANOPY_SIDES, FUSELAGE, SIDES, SPINE, ring};
use crate::ship::solid::Solid;

/// Where the mouth of the cockpit starts and ends along z. The joint between the nose and the
/// body lies between the two.
pub const MOUTH: (f32, f32) = (-2.2, -0.9);

/// Half width of the mouth at its front, at the joint and at its back. It stays inside the
/// foot of the canopy, so the glass covers all of it.
pub const HALF: [f32; 3] = [0.26, 0.36, 0.36];

/// How far down its lower side faces the glass goes: just under the skin of the hull and
/// outside the walls of the tub, so that no pane hangs inside the cockpit.
const RIM: f32 = 0.58;

/// Returns the station of the fuselage at `z`, between the two stations around it.
pub fn station(z: f32) -> [f32; 4] {
    let after = FUSELAGE.iter().position(|s| s[0] >= z).unwrap_or(1).max(1);
    let (a, b) = (FUSELAGE[after - 1], FUSELAGE[after]);
    let t = (z - a[0]) / (b[0] - a[0]);
    std::array::from_fn(|i| a[i] + (b[i] - a[i]) * t)
}

/// Returns the three stations of the mouth: its front, the joint and its back.
pub fn mouth() -> [[f32; 4]; 3] {
    [station(MOUTH.0), FUSELAGE[1], station(MOUTH.1)]
}

/// Returns the point of the roof of the hull at `station` that is `half` metres to the right
/// of the middle, or to the left for a negative `half`: the lip of the mouth.
pub fn lip(station: [f32; 4], half: f32) -> Vec3 {
    let (top, corner) = (ring(station, 2, SIDES), ring(station, 1, SIDES));
    let at = top.lerp(corner, half.abs() / corner.x);
    Vec3::new(at.x * half.signum(), at.y, at.z)
}

/// Adds the skin of the fuselage between the stations `a` and `b`. With `halves` the two roof
/// faces are left open between the lips, which are that far from the middle at `a` and at `b`.
fn band(solid: &mut Solid, a: [f32; 4], b: [f32; 4], halves: Option<(f32, f32)>) {
    for i in 0..SIDES {
        let out = ring([0.0, 1.0, 1.0, 0.0], i, SIDES) + ring([0.0, 1.0, 1.0, 0.0], i + 1, SIDES);
        let (from, to) = (ring(a, i, SIDES), ring(b, i, SIDES));
        match halves.filter(|_| i == 1 || i == 2) {
            None => solid.quad(
                [from, ring(a, i + 1, SIDES), ring(b, i + 1, SIDES), to],
                out,
            ),
            Some((wa, wb)) => {
                let (edge, side) = if i == 1 { (1, 1.0) } else { (3, -1.0) };
                let (lips, ends) = (
                    (lip(a, wa * side), lip(b, wb * side)),
                    (ring(a, edge, SIDES), ring(b, edge, SIDES)),
                );
                solid.quad([ends.0, lips.0, lips.1, ends.1], out);
            }
        }
    }
}

/// Adds the fan that closes the fuselage from `end` to `station`, facing `out`.
fn cap(solid: &mut Solid, end: Vec3, station: [f32; 4], out: Vec3) {
    for i in 0..SIDES {
        solid.tri(
            [end, ring(station, i, SIDES), ring(station, i + 1, SIDES)],
            out,
        );
    }
}

/// Returns the two halves of the fuselage as they are drawn, the body and the nose, with the
/// mouth of the cockpit open in their roof and nothing across the joint between them.
pub fn hull() -> [Solid; 2] {
    let (mut body, mut nose) = (Solid::default(), Solid::default());
    let [front, joint, back] = mouth();
    cap(&mut nose, SPINE[0], FUSELAGE[0], Vec3::NEG_Z);
    band(&mut nose, FUSELAGE[0], front, None);
    band(&mut nose, front, joint, Some((HALF[0], HALF[1])));
    band(&mut body, joint, back, Some((HALF[1], HALF[2])));
    band(&mut body, back, FUSELAGE[2], None);
    for pair in FUSELAGE[2..].windows(2) {
        band(&mut body, pair[0], pair[1], None);
    }
    cap(&mut body, SPINE[2], FUSELAGE[4], Vec3::Z);
    [body, nose]
}

/// Returns the six points of the glass at a station of the canopy, from its right rim over
/// the roof to its left rim.
pub fn arch(station: [f32; 4]) -> [Vec3; 6] {
    let at = |i: u32| ring(station, i, CANOPY_SIDES);
    [
        at(0).lerp(at(5), RIM),
        at(0),
        at(1),
        at(2),
        at(3),
        at(3).lerp(at(4), RIM),
    ]
}

/// Returns the canopy glass: the part of the bubble above the hull, open underneath.
pub fn glass() -> Solid {
    let mut glass = Solid::default();
    let (front, back) = (arch(CANOPY[0]), arch(CANOPY[1]));
    let middle = Vec3::new(0.0, 0.5, 0.5 * (CANOPY[0][0] + CANOPY[1][0]));
    for i in 0..5 {
        let pane = [front[i], front[i + 1], back[i + 1], back[i]];
        glass.quad(pane, (pane[0] + pane[2]) * 0.5 - middle);
        glass.tri([CANOPY_ENDS.0, front[i], front[i + 1]], Vec3::NEG_Z);
        glass.tri([CANOPY_ENDS.1, back[i], back[i + 1]], Vec3::Z);
    }
    glass
}
