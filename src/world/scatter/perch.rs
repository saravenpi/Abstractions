//! Where a stone can rest: on ground no steeper than its angle of repose, never perched on a crest, a
//! ridge or a peak, and more often in a hollow, at the foot of a slope, below a cliff or in a river bed.
//! A stone that rests lies tilted to the ground, and a plain rock of the tables is one of the hard
//! stones that lie on the planet, picked by the lode of the spot.

use std::f32::consts::TAU;

use bevy::prelude::*;
use rand::RngExt;
use rand_chacha::ChaCha8Rng;

use super::ctx::Ctx;
use crate::world::propmesh::Plant;
use crate::world::rocks::tune::{CREST, HOLLOW, LODE_MIX, LODES, REPOSE, RING, STEP};

/// Points on the ring a stone is weighed against.
const RING_POINTS: usize = 8;

/// The ground around a spot, as a stone resting there feels it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lie {
    /// Rise over run of the drawn ground under the stone along chart x and z, in true metres.
    pub grade: Vec2,
    /// Height of the spot over the mean of a ring of [`RING`] metres around it, in metres: positive on a
    /// crest, a ridge or a peak, negative in a hollow.
    pub crest: f32,
}

/// Returns the angle of repose of stone `plant` in degrees, or nothing for a prop that is no resting stone.
pub fn repose(plant: Plant) -> Option<f32> {
    REPOSE
        .iter()
        .find(|(p, _)| *p == plant)
        .map(|(_, deg)| *deg)
}

/// Returns how many of the table stones of kind `plant` rest on ground of lie `lie`, out of [`HOLLOW`]:
/// none on ground steeper than its angle of repose or on a crest of [`CREST`] or more, one on flat or
/// evenly sloping ground, and up to [`HOLLOW`] in a hollow.
pub fn welcome(plant: Plant, lie: &Lie) -> f32 {
    let steep = repose(plant).is_none_or(|deg| lie.grade.length() > deg.to_radians().tan());
    let share = (1.0 - lie.crest / CREST).clamp(0.0, HOLLOW);
    if steep { 0.0 } else { share }
}

impl Ctx<'_> {
    /// Returns the height of the drawn ground at chart position `w`: the mesh of the chunk at its point
    /// nearest `w` on a globe, the finest mesh on the legacy plane.
    fn floor(&self, w: Vec2) -> f32 {
        match &self.relief {
            Some(relief) => relief.near(w),
            None => self.terrain.mesh_height(w.x, w.y),
        }
    }

    /// Returns the grade of the drawn ground at chart position `w`: the triangle of the chunk mesh under
    /// it on a globe, in true metres, and a difference across [`STEP`] metres on the legacy plane.
    fn grade(&self, w: Vec2) -> Vec2 {
        if let Some(relief) = &self.relief {
            let (_, grade) = relief.at(w);
            return grade.normalize_or_zero() * self.metric.slope(grade);
        }
        let across = |d: Vec2| (self.floor(w + d * STEP) - self.floor(w - d * STEP)) / (2.0 * STEP);
        Vec2::new(across(Vec2::X), across(Vec2::Y))
    }

    /// Returns the lie of the ground at chart position `w`.
    pub fn lie(&self, w: Vec2) -> Lie {
        let turn = |k: usize| Vec2::from_angle(k as f32 / RING_POINTS as f32 * TAU) * RING;
        let ring: f32 = (0..RING_POINTS)
            .map(|k| self.floor(w + self.metric.chart(turn(k))))
            .sum();
        Lie {
            grade: self.grade(w),
            crest: self.floor(w) - ring / RING_POINTS as f32,
        }
    }

    /// Returns the plant a table pick of `plant` shows at chart position `w`: a plain rock is one of the
    /// hard stones that lie on the planet, by the lode there mixed with `mix`, a value in `[0, 1)`.
    /// Anything else is itself.
    fn lode(&self, w: Vec2, plant: Plant, mix: f32) -> Plant {
        if plant != Plant::Rock {
            return plant;
        }
        let record = self.terrain.globe().record();
        let lying = || LODES.into_iter().filter(move |(m, _)| m.lies_on(record));
        let field = self.fields.lode((self.terrain, w));
        let pick = (field * (1.0 - LODE_MIX) + mix * LODE_MIX).min(0.999);
        let mut left = pick * lying().map(|(_, share)| share).sum::<f32>();
        let found = lying().find(|(_, share)| {
            left -= share;
            left < 0.0
        });
        found.map_or(plant, |(stone, _)| stone.plant())
    }

    /// Returns the plant a table pick of `plant` lays at chart position `w` and the grade it is tilted
    /// to, or nothing where the stone could not rest: a draw from `rng` keeps it by its welcome there.
    /// Anything that is no resting stone passes unchanged and upright, without a draw.
    pub fn settle(&self, rng: &mut ChaCha8Rng, w: Vec2, plant: Plant) -> Option<(Plant, Vec2)> {
        if repose(plant).is_none() {
            return Some((plant, Vec2::ZERO));
        }
        let (keep, mix): (f32, f32) = (rng.random(), rng.random());
        let (stone, lie) = (self.lode(w, plant, mix), self.lie(w));
        (keep * HOLLOW < welcome(stone, &lie)).then_some((stone, lie.grade))
    }
}
