//! What is bolted into the cockpit tub: the seat, the side consoles, the dashboard with its
//! two hooded instrument pods, the column of the stick and the faces of the instruments.

use bevy::prelude::*;

use crate::ship::cockpit::{BACK, FLOOR, PAN, PANEL_Z, STICK};
use crate::ship::solid::Solid;

/// Height of the top of the side consoles: the throttle hinges on the left one.
pub const CONSOLE: f32 = 0.22;

/// Inner and outer edge of the instrument pods across the hull, and their height: low enough
/// to leave the horizon free, and apart, so that the nose shows between them.
const POD: (f32, f32, f32) = (0.1, 0.3, 0.6);

/// Adds the box between `lo` and `hi` and its mirror image on the left of the hull.
fn pair(solid: &mut Solid, lo: Vec3, hi: Vec3) {
    solid.slab(lo, hi);
    solid.slab(Vec3::new(-hi.x, lo.y, lo.z), Vec3::new(-lo.x, hi.y, hi.z));
}

/// Returns the cushions of the seat: the pan, the backrest and the headrest.
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
    seat
}

/// Returns the dark furniture of the tub: the foot of the seat, the column of the stick, the
/// two side consoles, the dashboard, and the two instrument pods with their hoods.
pub fn furniture() -> Solid {
    let mut dark = Solid::default();
    dark.slab(Vec3::new(-0.14, FLOOR, -1.3), Vec3::new(0.14, 0.0, -1.0));
    dark.slab(
        Vec3::new(-0.025, FLOOR, STICK.z - 0.025),
        Vec3::new(0.025, STICK.y, STICK.z + 0.025),
    );
    pair(
        &mut dark,
        Vec3::new(0.22, FLOOR, -1.8),
        Vec3::new(0.36, CONSOLE, BACK),
    );
    dark.slab(Vec3::new(-0.25, 0.3, -2.2), Vec3::new(0.25, 0.48, -1.99));
    pair(
        &mut dark,
        Vec3::new(POD.0, 0.36, -2.12),
        Vec3::new(POD.1, POD.2, PANEL_Z),
    );
    pair(
        &mut dark,
        Vec3::new(POD.0 - 0.01, POD.2, -2.12),
        Vec3::new(POD.1 + 0.01, POD.2 + 0.02, PANEL_Z + 0.05),
    );
    dark
}

/// Returns the black faces of the two instrument pods, which the lit instruments stand on.
pub fn faces() -> Solid {
    let mut faces = Solid::default();
    let z = PANEL_Z + 0.003;
    for side in [-1.0, 1.0] {
        let (near, far) = (side * (POD.0 + 0.02), side * (POD.1 - 0.005));
        let face = [
            Vec3::new(near, 0.465, z),
            Vec3::new(far, 0.465, z),
            Vec3::new(far, POD.2 - 0.012, z),
            Vec3::new(near, POD.2 - 0.012, z),
        ];
        faces.quad(face, Vec3::Z);
    }
    faces
}
