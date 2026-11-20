//! Directions: which side a swing comes from, read from the mouse over the last moment.

use bevy::prelude::*;

use super::tune::{DEADZONE, STROKES, WINDOW};

/// The side a swing comes from, or the side a guard covers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Dir {
    /// From the right, across to the left.
    #[default]
    Right,
    /// From the left, a backhand across to the right.
    Left,
    /// From above: overhead.
    Up,
    /// From below: rising.
    Down,
}

impl Dir {
    /// Returns the direction of a mouse `motion` on the screen, +Y down, or nothing when it is under
    /// [`DEADZONE`] pixels. Moving right swings from the right, moving up swings overhead.
    pub fn of(motion: Vec2) -> Option<Dir> {
        if motion.length() < DEADZONE {
            return None;
        }
        Some(match motion.x.abs() > motion.y.abs() {
            true if motion.x > 0.0 => Dir::Right,
            true => Dir::Left,
            false if motion.y < 0.0 => Dir::Up,
            false => Dir::Down,
        })
    }

    /// Returns +1 for the right, -1 for the left and 0 above or below.
    pub fn side(self) -> f32 {
        match self {
            Dir::Right => 1.0,
            Dir::Left => -1.0,
            Dir::Up | Dir::Down => 0.0,
        }
    }
}

/// The mouse motion of the last frames, in a ring that never grows, and the direction it chose last.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Strokes {
    /// Game seconds and motion of each sample.
    at: [(f32, Vec2); STROKES],
    /// Index of the next sample to write.
    next: usize,
    /// The direction chosen last, kept while the mouse rests.
    pub last: Dir,
}

impl Default for Strokes {
    fn default() -> Self {
        Self {
            at: [(f32::NEG_INFINITY, Vec2::ZERO); STROKES],
            next: 0,
            last: Dir::default(),
        }
    }
}

impl Strokes {
    /// Records the mouse motion `delta` of the frame at `now` seconds.
    pub fn push(&mut self, now: f32, delta: Vec2) {
        self.at[self.next] = (now, delta);
        self.next = (self.next + 1) % STROKES;
    }

    /// Returns the sum of the motion of the last [`WINDOW`] seconds before `now`.
    pub fn sum(&self, now: f32) -> Vec2 {
        let recent = self.at.iter().filter(|(t, _)| now - *t <= WINDOW);
        recent.map(|(_, d)| *d).sum()
    }

    /// Returns the direction the motion of the window before `now` asks, and keeps it: the last one
    /// chosen when the mouse rested.
    pub fn dir(&mut self, now: f32) -> Dir {
        if let Some(dir) = Dir::of(self.sum(now)) {
            self.last = dir;
        }
        self.last
    }
}
