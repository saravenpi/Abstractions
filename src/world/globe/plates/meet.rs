//! Where plates meet: the ground every pair of plates near a direction shapes along its edge.

use bevy::math::DVec3;

use super::Plates;
use super::edge::{Meet, Uplift, profile};
use super::near::Near;
use super::tune::{JUNCTION, RIDGE};
use crate::world::islands::smooth;

impl Plates {
    /// Returns how fast plates `i` and `j` close in at warped unit direction `w`, as a share of the fastest
    /// plate: the speed of one against the other across their edge, negative when they part. Swapping
    /// the plates gives the same value.
    pub(super) fn closing(&self, w: DVec3, i: usize, j: usize) -> f64 {
        let across = self.centres[i] - self.centres[j];
        let normal = (across - w * across.dot(w)).normalize_or_zero();
        (self.spins[j] - self.spins[i]).cross(w).dot(normal)
    }

    /// Returns what the edges between the plates of `near` add at warped unit direction `w`: the sum over
    /// every pair, so that a third plate close to an edge shapes both of its sides alike. Where ranges
    /// cross, the ground stops at the height of the highest mountains.
    pub(super) fn uplift(&self, w: DVec3, near: &Near) -> Uplift {
        let mut sum = Uplift::default();
        for a in 0..near.count {
            for b in a + 1..near.count {
                let (add, fade) = self.pair(w, near, [a, b]);
                sum.rise += add.rise * fade;
                sum.floor += add.floor * fade;
            }
        }
        sum.rise = sum.rise.min(1.0);
        sum
    }

    /// Returns what the edge between entries `a` and `b` of `near` adds at warped unit direction `w`, and
    /// how much of it counts: all of it inside the cells of the two plates, none [`JUNCTION`] past them.
    ///
    /// The pair is read with the lower plate index first, so both sides of the edge see the same plate
    /// sink.
    fn pair(&self, w: DVec3, near: &Near, [a, b]: [usize; 2]) -> (Uplift, f32) {
        let [a, b] = if near.ids[a] < near.ids[b] {
            [a, b]
        } else {
            [b, a]
        };
        let (i, j) = (near.ids[a], near.ids[b]);
        let t = (near.dots[a] - near.dots[b]) / self.chord(i, j);
        let out = (near.dots[0] - near.dots[a].max(near.dots[b])) / self.spacing;
        let fade = 1.0 - smooth(0.0, JUNCTION as f32, out as f32);
        if t.abs() >= RIDGE || fade <= 0.0 {
            return (Uplift::default(), 0.0);
        }
        let meet = Meet {
            crust: [self.crust[i], self.crust[j]],
            t: t as f32,
            closing: self.closing(w, i, j) as f32,
            mobility: self.interior.mobility,
        };
        (profile(&meet), fade)
    }
}
