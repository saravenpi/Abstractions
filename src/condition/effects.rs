//! What a damaged leg or arm does to the system that drives it: each is a share from 0 to 1 the
//! system multiplies its own number by, 1 for a whole part.

use super::tune::{ARM_POWER, BROKEN, CRAWL, CRAWL_PACE, LIMP_JUMP, LIMP_PACE};
use super::{Condition, Part};

/// Returns `floor` for a destroyed part and 1 for a whole one, between them in a straight line.
pub(super) fn keep(health: f32, floor: f32) -> f32 {
    floor + (1.0 - floor) * health.clamp(0.0, 1.0)
}

impl Condition {
    /// Returns the mean health of the legs.
    fn legs(&self) -> f32 {
        (self.of(Part::LegL) + self.of(Part::LegR)) / 2.0
    }

    /// Returns true when both legs are badly hurt: the robot crawls.
    pub fn crawling(&self) -> bool {
        self.of(Part::LegL) < CRAWL && self.of(Part::LegR) < CRAWL
    }

    /// Returns the share of its walking speed the legs leave.
    pub fn pace(&self) -> f32 {
        match self.crawling() {
            true => CRAWL_PACE,
            false => keep(self.legs(), LIMP_PACE),
        }
    }

    /// Returns the share of its jump speed the legs leave.
    pub fn leap(&self) -> f32 {
        keep(self.legs(), LIMP_JUMP)
    }

    /// Returns the share of its swing speed the arm on `side` (0 left, 1 right) leaves.
    pub fn arm(&self, side: usize) -> f32 {
        keep(self.of([Part::ArmL, Part::ArmR][side]), ARM_POWER)
    }

    /// Returns true when the arm on `side` is broken and holds nothing.
    pub fn broken(&self, side: usize) -> bool {
        self.of([Part::ArmL, Part::ArmR][side]) < BROKEN
    }
}
