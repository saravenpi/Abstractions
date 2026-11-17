//! Door frames: a casing that wraps the wall around a doorway.

use super::brush::{Build, Mat};
use super::tags::Tag;

/// How far the casing covers the sides and the top of the hole, so that none of its faces lies in the plane
/// of the wall faces it covers.
const PROUD: f32 = 0.007;

/// Width of the casing boards.
const BOARD: f32 = 0.12;

/// How far the casing stands out of the inner and the outer wall face.
const REACH: (f32, f32) = (0.03, 0.06);

/// A doorway through a wall that runs along local x, its threshold at `y = 0`.
#[derive(Debug, Clone, Copy)]
pub struct Doorway {
    /// Left and right side of the hole along x.
    pub x: (f32, f32),
    /// Height of the hole.
    pub top: f32,
    /// Inner and outer face of the wall along z.
    pub z: (f32, f32),
}

impl Build<'_> {
    /// Adds the casing of `door`: two jambs standing on the threshold and a head across them. It wraps the
    /// wall, standing out of both faces, and covers the sides and the top of the hole by a few millimeters.
    pub fn door_frame(&mut self, door: &Doorway, rgb: [f32; 3]) {
        let (x0, x1) = (door.x.0 + PROUD, door.x.1 - PROUD);
        let (z0, z1) = (door.z.0 - REACH.0, door.z.1 + REACH.1);
        let top = door.top - PROUD;
        let list = [
            ([x0 - BOARD, 0.0, z0], [x0, top, z1]),
            ([x1, 0.0, z0], [x1 + BOARD, top, z1]),
            ([x0 - BOARD, top, z0], [x1 + BOARD, top + BOARD, z1]),
        ];
        self.tagged(Tag::Frame, |b| b.boxes(&list, Mat::deco(rgb)));
    }
}
