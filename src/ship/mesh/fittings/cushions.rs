//! The seat: the pan, the backrest and the headrest, with side bolsters and a harness.

use bevy::prelude::*;

use super::pair;
use crate::ship::cockpit::{BACK, PAN};
use crate::ship::solid::Solid;

/// Thickness of a strap of the harness, in metres: it lies on the face of the backrest.
const STRAP: f32 = 0.008;

/// Adds the bolsters on the sides of the pan and of the backrest.
fn bolsters(seat: &mut Solid) {
    pair(
        seat,
        Vec3::new(0.2, 0.0, -1.35),
        Vec3::new(0.225, 0.11, BACK),
    );
    pair(
        seat,
        Vec3::new(0.19, PAN, BACK - 0.06),
        Vec3::new(0.215, 0.5, BACK),
    );
}

/// Adds the harness: the two shoulder straps down the backrest, the lap belt across the pan and
/// its buckle.
fn harness(seat: &mut Solid) {
    pair(
        seat,
        Vec3::new(0.06, 0.14, BACK - STRAP),
        Vec3::new(0.09, 0.64, BACK),
    );
    seat.slab(
        Vec3::new(-0.2, PAN, -1.12),
        Vec3::new(0.2, PAN + STRAP, -1.08),
    );
    seat.slab(
        Vec3::new(-0.03, PAN + STRAP, -1.13),
        Vec3::new(0.03, PAN + 0.02, -1.07),
    );
}

/// Returns the cushions of the seat: the pan, the backrest and the headrest, the bolsters and
/// the harness. It is the same on both sides.
pub fn seat() -> Solid {
    let mut seat = Solid::default();
    seat.slab(Vec3::new(-0.2, 0.0, -1.4), Vec3::new(0.2, PAN, BACK));
    seat.slab(
        Vec3::new(-0.19, PAN, BACK),
        Vec3::new(0.19, 0.66, BACK + 0.045),
    );
    seat.slab(
        Vec3::new(-0.11, 0.66, BACK),
        Vec3::new(0.11, 1.0, BACK + 0.045),
    );
    bolsters(&mut seat);
    harness(&mut seat);
    seat
}
