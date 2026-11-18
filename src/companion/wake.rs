//! The wake: the points the companion flew through, kept in the planet frame.
//!
//! This is the ring of `gear::fx::trail`, which is private to the gear, cut down to a third of
//! its length. The points are 64-bit positions in the frame of the anchor, so an anchor rebase
//! moves none of them, and a point never moves once it is laid: it only ages.

use bevy::math::DVec3;

/// Number of points a wake keeps. The ribbon has one row more, for the live tip.
pub const POINTS: usize = 24;

/// Distance in metres between two points: 36 m of wake, a third of a wingtip trail.
pub const SPACING: f64 = 1.5;

/// Jump in metres above which the wake starts over instead of streaking across the world.
pub const BREAK: f64 = 30.0;

/// One point of a wake.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Sample {
    /// Position in the planet frame, in metres.
    pub at: DVec3,
    /// Seconds since it was laid.
    pub age: f32,
    /// How strongly it glowed when laid, 0 for a gap.
    pub heat: f32,
}

/// A fixed ring of points behind the companion. No heap.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Wake {
    /// The ring of points.
    ring: [Sample; POINTS],
    /// Index of the newest point.
    head: usize,
    /// Number of points laid, at most [`POINTS`].
    len: usize,
    /// The emitter right now: the start of the ribbon, ahead of the newest point.
    tip: Sample,
}

impl Default for Wake {
    /// An empty wake.
    fn default() -> Self {
        Self {
            ring: [Sample::default(); POINTS],
            head: 0,
            len: 0,
            tip: Sample::default(),
        }
    }
}

impl Wake {
    /// Returns the number of points laid, at most [`POINTS`].
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns the emitter position and heat of the last feed.
    pub fn tip(&self) -> Sample {
        self.tip
    }

    /// Returns point `index`, 0 for the newest. Past the end it repeats the oldest, or the tip
    /// of an empty wake.
    pub fn point(&self, index: usize) -> Sample {
        if self.len == 0 {
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
    /// from the newest point to the emitter, however long the frame was. A jump longer than
    /// [`BREAK`] starts the wake over.
    pub fn feed(&mut self, at: DVec3, heat: f32, dt: f32) {
        for sample in &mut self.ring {
            sample.age += dt;
        }
        let from = self.point(0);
        self.tip = Sample { at, age: 0.0, heat };
        let gap = from.at.distance(at);
        if self.len == 0 || gap > BREAK {
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
