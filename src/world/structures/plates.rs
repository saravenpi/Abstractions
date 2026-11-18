//! Colliders of roofs: a thin box just under each drawn slope, so that whoever lands on a roof stands on what
//! they see, and level steps that fill the hipped ends no box can follow.

use bevy::prelude::*;

use super::brush::{Build, Part, Shape};

/// Thickness of the box under a roof slope, in meters: thin enough to stay out of the room under the lowest
/// pitch, the shed.
const THICK: f32 = 0.1;

/// Heights of the level steps that fill a hipped roof, as shares of its rise. The deepest dip between the
/// drawn roof and its collider is the widest gap between two of them.
const STEPS: [f32; 5] = [0.12, 0.3, 0.5, 0.68, 0.85];

/// Returns the share of a slope of pitch `rise / run` that its box covers from the eave up. On a steep gable
/// the box under one slope would stick out of the other near the ridge unless it stops short of it.
fn ridge_keep(rise: f32, run: f32) -> f32 {
    let s = rise / run;
    let short = THICK * ((s * s - 1.0) / (2.0 * s)).max(0.0);
    1.0 - short / rise.hypot(run)
}

impl Build<'_> {
    /// Adds a box whose top face is the local rectangle on the edge `from..to` that rises by `up`, which is
    /// square to the edge, and that reaches [`THICK`] under it.
    pub fn plate(&mut self, from: Vec3, to: Vec3, up: Vec3) {
        let along = to - from;
        let rise = up.normalize();
        let mut x = along.normalize();
        let mut n = rise.cross(x);
        if n.y < 0.0 {
            (x, n) = (-x, -n);
        }
        let local = Quat::from_mat3(&Mat3::from_cols(x, n, rise));
        let center = from + along * 0.5 + up * 0.5 - n * THICK * 0.5;
        self.parts.push(Part {
            pos: self.pt(center),
            rot: Quat::from_rotation_y(self.yaw) * local,
            shape: Shape::Cuboid(Vec3::new(along.length(), THICK, up.length())),
        });
    }

    /// Adds the boxes under the two slopes of a gabled roof over `lo..hi`, its ridge `rise` above the eaves
    /// at `lo.y` along local x.
    pub fn pitch_plates(&mut self, lo: Vec3, hi: Vec3, rise: f32) {
        let half = (hi.z - lo.z) * 0.5;
        let keep = ridge_keep(rise, half);
        for (z, run) in [(lo.z, half), (hi.z, -half)] {
            let (a, b) = (Vec3::new(lo.x, lo.y, z), Vec3::new(hi.x, lo.y, z));
            self.plate(a, b, Vec3::new(0.0, rise, run) * keep);
        }
    }

    /// Adds the collider of a hipped roof over `lo..hi`: a box under the lower half of each of its four
    /// slopes, as wide as the slope is halfway up, and level steps inside the roof, each one inset to stay
    /// under all four slopes, for the corners and the upper half that no box can follow.
    pub fn hip_plates(&mut self, lo: Vec3, hi: Vec3, rise: f32) {
        let (mid, run) = ((lo + hi) * 0.5, (hi.z - lo.z) * 0.5);
        let end = run.min((hi.x - lo.x) * 0.5);
        for side in [-1.0, 1.0] {
            let (x, z) = (mid.x + side * (hi.x - mid.x), mid.z + side * run);
            let long = [lo.x + end * 0.5, hi.x - end * 0.5].map(|x| Vec3::new(x, lo.y, z));
            self.plate(long[0], long[1], Vec3::new(0.0, rise, -side * run) * 0.5);
            let short = [-0.5, 0.5].map(|k| Vec3::new(x, lo.y, mid.z + k * run));
            self.plate(short[0], short[1], Vec3::new(-side * end, rise, 0.0) * 0.5);
        }
        for k in STEPS {
            let (dx, dz) = (end * k, run * k);
            let top = Vec3::new(hi.x - dx, lo.y + rise * k, hi.z - dz);
            self.collider(Vec3::new(lo.x + dx, lo.y, lo.z + dz), top);
        }
    }
}
