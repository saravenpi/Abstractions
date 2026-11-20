//! The plates near a direction: the one that holds it and those whose edge with it is close enough to
//! shape the ground there.

use super::tune::RIDGE;

/// Most plates kept near one direction: the owner and nine others. Eight is the most a model of these
/// plates has shown within reach of one direction.
pub const KEPT: usize = 10;

/// The plates near one direction, the owner first.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Near {
    /// Index of each plate kept, the owner first.
    pub ids: [usize; KEPT],
    /// Cosine of the angle from the direction to the centre of each plate kept.
    pub dots: [f64; KEPT],
    /// How many plates are kept.
    pub count: usize,
    /// Radians to the nearest edge of the owner's cell.
    pub edge: f64,
    /// Index of the plate across that edge.
    pub next: usize,
}

impl Near {
    /// Returns the plates near a direction held by plate `owner`, whose centre is at cosine `dot`, before
    /// any other plate is taken.
    pub fn of(owner: usize, dot: f64) -> Self {
        Self {
            ids: [owner; KEPT],
            dots: [dot; KEPT],
            count: 1,
            edge: f64::MAX,
            next: owner,
        }
    }

    /// Takes plate `k`, whose centre is at cosine `dot`, into account: it becomes the plate across the
    /// nearest edge when its edge is the closest so far, `chord` being the distance between its centre and
    /// the owner's, and it is kept when that edge lies within reach of a ridge for seeds `spacing` apart.
    ///
    /// The second test reads only the cosines, so both sides of an edge keep the same plates.
    pub fn take(&mut self, k: usize, dot: f64, [chord, spacing]: [f64; 2]) {
        let gap = self.dots[0] - dot;
        if gap / chord < self.edge {
            (self.edge, self.next) = (gap / chord, k);
        }
        if gap < 2.0 * RIDGE * spacing && self.count < KEPT {
            (self.ids[self.count], self.dots[self.count]) = (k, dot);
            self.count += 1;
        }
    }
}
