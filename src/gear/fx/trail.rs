//! A trail: a ring buffer of points laid down by distance, kept in the planet frame.
//!
//! The points are 64-bit positions in the frame of the anchor, so an anchor rebase moves none
//! of them: only their render positions change, and the ribbon is rebuilt from them each frame.

use bevy::math::DVec3;

/// Number of points a trail keeps. The ribbon mesh has one row more, for the live tip.
pub const POINTS: usize = 64;

/// Distance in metres between two points.
pub const SPACING: f64 = 1.75;

/// Jump in metres above which the trail starts over instead of streaking across the world.
pub const BREAK: f64 = 60.0;

/// One point of a trail.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Sample {
    /// Position in the planet frame, in metres.
    pub at: DVec3,
    /// Seconds since it was laid.
    pub age: f32,
    /// How strongly it glowed when laid, 0 for a gap.
    pub heat: f32,
}

/// A fixed ring of trail points behind one emitter. No heap.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Trail {
    /// The ring of points.
    ring: [Sample; POINTS],
    /// Index of the newest point.
    head: usize,
    /// Number of points laid, at most [`POINTS`].
    len: usize,
    /// The emitter right now: the start of the ribbon, ahead of the newest point.
    tip: Sample,
}

impl Default for Trail {
    /// An empty trail.
    fn default() -> Self {
        Self {
            ring: [Sample::default(); POINTS],
            head: 0,
            len: 0,
            tip: Sample::default(),
        }
    }
}

impl Trail {
    /// Returns the number of points laid, at most [`POINTS`].
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true before the first point is laid.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the emitter position and heat of the last feed.
    pub fn tip(&self) -> Sample {
        self.tip
    }

    /// Returns point `index`, 0 for the newest. Past the end it repeats the oldest, or the tip
    /// of an empty trail.
    pub fn point(&self, index: usize) -> Sample {
        if self.is_empty() {
            return self.tip;
        }
        self.ring[(self.head + POINTS - index.min(self.len - 1)) % POINTS]
    }

    /// Returns true while any point younger than `life` seconds still glows.
    pub fn live(&self, life: f32) -> bool {
        let glows = |sample: &Sample| sample.heat > 0.0 && sample.age < life;
        glows(&self.tip) || (0..self.len).any(|index| glows(&self.point(index)))
    }

    /// Lays one point at the head of the ring.
    fn push(&mut self, sample: Sample) {
        self.head = (self.head + 1) % POINTS;
        self.ring[self.head] = sample;
        self.len = (self.len + 1).min(POINTS);
    }

    /// Moves the emitter to `at` after `dt` seconds, glowing at `heat`.
    ///
    /// Every point ages. New points are laid every [`SPACING`] metres along the straight line
    /// from the newest point to the emitter, however long the frame was, so the spacing does
    /// not depend on the frame rate. A jump longer than [`BREAK`] starts the trail over.
    pub fn feed(&mut self, at: DVec3, heat: f32, dt: f32) {
        for sample in &mut self.ring {
            sample.age += dt;
        }
        let from = self.point(0);
        self.tip = Sample { at, age: 0.0, heat };
        let gap = from.at.distance(at);
        if self.is_empty() || gap > BREAK {
            self.len = 0;
            self.push(self.tip);
            return;
        }
        for step in 1..=(gap / SPACING) as usize {
            let along = (step as f64 * SPACING / gap) as f32;
            self.push(Sample {
                at: from.at.lerp(at, f64::from(along)),
                age: dt * (1.0 - along),
                heat: from.heat + (heat - from.heat) * along,
            });
        }
    }
}
