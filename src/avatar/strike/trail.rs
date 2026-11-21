//! Trail: the path of the edge of a tool through the air, kept for a moment so the swing can be seen.
//!
//! One row per frame while the tip is fast: the two ends of the edge, in the frame of the body. The
//! ring is fixed in size, so nothing is allocated; a row older than [`TRAIL_LIFE`] is dropped. It is
//! the same idea as the ring of the gear trails, but laid by time and not by distance, since a swing
//! covers a metre in a tenth of a second and a ring spaced by 1.75 m would hold two points of it.

use bevy::prelude::*;

use super::tune::{TRAIL_LIFE, TRAIL_SPEED};

/// Rows a trail keeps: more than the frames of [`TRAIL_LIFE`] at any frame rate a player plays at.
pub const ROWS: usize = 32;

/// One row of the trail.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Row {
    /// The two ends of the edge, the one nearer the grip first, in the frame of the body.
    pub edge: [Vec3; 2],
    /// Seconds since it was laid.
    pub age: f32,
    /// How brightly it glows, 0 to 1.
    pub glow: f32,
}

/// The ring of rows behind one tool.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Trail {
    /// The ring.
    rows: [Row; ROWS],
    /// Index of the newest row.
    head: usize,
    /// Rows laid and still alive.
    len: usize,
    /// Seconds a row lasts.
    life: f32,
    /// Least seconds between two rows.
    every: f32,
}

impl Default for Trail {
    /// An empty trail of a swing: rows of [`TRAIL_LIFE`], one every frame.
    fn default() -> Self {
        Self::lasting(TRAIL_LIFE, 0.0)
    }
}

/// Returns how brightly a tip moving at `speed` metres per second glows: nothing at [`TRAIL_SPEED`],
/// full at twice it.
pub fn glow(speed: f32) -> f32 {
    ((speed - TRAIL_SPEED) / TRAIL_SPEED).clamp(0.0, 1.0)
}

impl Trail {
    /// Returns an empty trail whose rows last `life` seconds, laid no closer than `every` seconds.
    pub fn lasting(life: f32, every: f32) -> Self {
        Self {
            rows: [Row::default(); ROWS],
            head: 0,
            len: 0,
            life,
            every,
        }
    }

    /// Returns the seconds a row lasts.
    pub fn life(&self) -> f32 {
        self.life
    }

    /// Returns the number of rows alive.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true when no row is alive.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns row `index`, 0 for the newest, or nothing past the oldest.
    pub fn row(&self, index: usize) -> Option<Row> {
        (index < self.len).then(|| self.rows[(self.head + ROWS - index) % ROWS])
    }

    /// Ages every row by `dt` seconds, drops the ones past the life of the trail, and lays a row of
    /// `edge` when the tip moves at `speed` metres per second, fast enough to glow, and the newest
    /// row is old enough.
    pub fn feed(&mut self, edge: [Vec3; 2], speed: f32, dt: f32) {
        for row in &mut self.rows {
            row.age += dt;
        }
        while self
            .row(self.len.saturating_sub(1))
            .is_some_and(|r| r.age > self.life)
        {
            self.len -= 1;
        }
        let glow = glow(speed);
        let spaced = self.row(0).is_none_or(|r| r.age >= self.every);
        if glow > 0.0 && spaced {
            self.head = (self.head + 1) % ROWS;
            self.rows[self.head] = Row {
                edge,
                age: 0.0,
                glow,
            };
            self.len = (self.len + 1).min(ROWS);
        }
    }
}
