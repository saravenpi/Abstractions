//! The hull as it is drawn: the outline of `frame`, open over the cockpit so that the tub, the
//! seat and the pilot inside are seen from outside and from the seat.

use bevy::prelude::*;

use super::frame::{FUSELAGE, JOINT, POST, PRE, SIDES, SPINE, rim};
use super::plating::{cone, plating};
use crate::ship::solid::Solid;

/// Where the mouth of the cockpit starts and ends along z. The joint between the nose and the
/// body lies between the two.
pub const MOUTH: (f32, f32) = (-2.2, -0.9);

/// Half width of the mouth at its front, at the joint and at its back. It stays inside the
/// foot of the canopy, so the glass covers all of it.
pub const HALF: [f32; 3] = [0.33, 0.38, 0.38];

/// Returns the station of the fuselage at `z`, between the two stations around it.
pub fn station(z: f32) -> [f32; 4] {
    let after = FUSELAGE.iter().position(|s| s[0] >= z).unwrap_or(1).max(1);
    let (a, b) = (FUSELAGE[after - 1], FUSELAGE[after]);
    let t = (z - a[0]) / (b[0] - a[0]);
    std::array::from_fn(|i| a[i] + (b[i] - a[i]) * t)
}

/// Returns the three stations of the mouth: its front, the joint and its back.
pub fn mouth() -> [[f32; 4]; 3] {
    [station(MOUTH.0), FUSELAGE[JOINT], station(MOUTH.1)]
}

/// Returns the point of the roof of the hull at `station` that is `half` metres to the right
/// of the middle, or to the left for a negative `half`: the lip of the mouth.
pub fn lip(station: [f32; 4], half: f32) -> Vec3 {
    let (top, corner) = (rim(station, 2), rim(station, 1));
    let at = top.lerp(corner, half.abs() / corner.x);
    Vec3::new(at.x * half.signum(), at.y, at.z)
}

/// Adds the skin of the fuselage between the stations `a` and `b`, the panel `i` in the tone
/// `tones[i]`. With `halves` the two roof faces are left open between the lips, which are that
/// far from the middle at `a` and at `b`.
fn band(
    solid: &mut Solid,
    (a, b): ([f32; 4], [f32; 4]),
    halves: Option<(f32, f32)>,
    tones: [u8; 8],
) {
    for i in 0..SIDES {
        solid.set_tone(tones[i as usize]);
        let out = rim([0.0, 1.0, 1.0, 0.0], i) + rim([0.0, 1.0, 1.0, 0.0], i + 1);
        let (from, to) = (rim(a, i), rim(b, i));
        match halves.filter(|_| i == 1 || i == 2) {
            None => solid.quad([from, rim(a, i + 1), rim(b, i + 1), to], out),
            Some((wa, wb)) => {
                let (edge, side) = if i == 1 { (1, 1.0) } else { (3, -1.0) };
                let (lips, ends) = (
                    (lip(a, wa * side), lip(b, wb * side)),
                    (rim(a, edge), rim(b, edge)),
                );
                solid.quad([ends.0, lips.0, lips.1, ends.1], out);
            }
        }
    }
}

/// Adds the fan that closes the fuselage from `end` to `station`, facing `out`, in the tones
/// of `tones`.
fn cap(solid: &mut Solid, end: Vec3, (station, out): ([f32; 4], Vec3), tones: [u8; 8]) {
    for i in 0..SIDES {
        solid.set_tone(tones[i as usize]);
        solid.tri([end, rim(station, i), rim(station, i + 1)], out);
    }
}

/// Returns the two halves of the fuselage as they are drawn, the body and the nose, with the
/// mouth of the cockpit open in their roof and nothing across the joint between them. Every
/// station is a ring of the same corners, so all the panels share their edges.
pub fn hull() -> [Solid; 2] {
    let (mut body, mut nose) = (Solid::default(), Solid::default());
    let [front, joint, back] = mouth();
    let (first, last) = (FUSELAGE[0], FUSELAGE[FUSELAGE.len() - 1]);
    cap(&mut nose, SPINE[0], (first, Vec3::NEG_Z), cone(0));
    for (k, pair) in FUSELAGE[..=PRE].windows(2).enumerate() {
        band(&mut nose, (pair[0], pair[1]), None, cone(k));
    }
    band(&mut nose, (FUSELAGE[PRE], front), None, plating());
    band(
        &mut nose,
        (front, joint),
        Some((HALF[0], HALF[1])),
        plating(),
    );
    band(
        &mut body,
        (joint, back),
        Some((HALF[1], HALF[2])),
        plating(),
    );
    band(&mut body, (back, FUSELAGE[POST]), None, plating());
    for pair in FUSELAGE[POST..].windows(2) {
        band(&mut body, (pair[0], pair[1]), None, plating());
    }
    cap(&mut body, SPINE[2], (last, Vec3::Z), plating());
    [body, nose]
}
