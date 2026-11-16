//! Stairs on a [`Build`]: solid steps no taller than the player can climb.

use bevy::prelude::*;

use super::brush::{Build, Mat};

/// Tallest allowed step.
const MAX_RISE: f32 = 0.29;

/// Depth of one step.
const RUN: f32 = 0.36;

impl Build<'_> {
    /// Adds a stair whose top end is at `run.0`, descending along `run.1` from height `rise.1` to `rise.0`.
    pub fn stair(&mut self, run: (Vec2, Vec2), width: f32, rise: (f32, f32), mat: Mat) {
        let (from, dir) = run;
        let (y0, y1) = rise;
        let n = (((y1 - y0) / MAX_RISE).ceil() as i32).clamp(1, 16);
        let step = (y1 - y0) / n as f32;
        let side = Vec2::new(-dir.y, dir.x) * width * 0.5;
        for i in 0..n {
            let a = from + dir * RUN * i as f32;
            let corners = [
                a - side,
                a + side,
                a + dir * RUN - side,
                a + dir * RUN + side,
            ];
            let lo = corners.iter().fold(Vec2::MAX, |m, p| m.min(*p));
            let hi = corners.iter().fold(Vec2::MIN, |m, p| m.max(*p));
            let top = y1 - step * i as f32;
            self.cuboid(
                Vec3::new(lo.x, y0 - 0.6, lo.y),
                Vec3::new(hi.x, top, hi.y),
                mat,
            );
        }
    }
}
