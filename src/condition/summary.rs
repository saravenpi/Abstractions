//! Reading the condition as a whole: its integrity, the parts that are hurt, and its saved form.

use super::tune::DAMAGED;
use super::{Condition, PARTS, Part};

impl Condition {
    /// Returns the integrity of the whole frame, from 0 to 1: the mean health of its parts.
    pub fn integrity(&self) -> f32 {
        self.whole.iter().sum::<f32>() / PARTS as f32
    }

    /// Returns the parts that carry damage, worst first.
    pub fn damaged(&self) -> Vec<Part> {
        let mut hurt: Vec<Part> = Part::ALL
            .into_iter()
            .filter(|part| self.of(*part) < DAMAGED)
            .collect();
        hurt.sort_by(|a, b| self.of(*a).total_cmp(&self.of(*b)));
        hurt
    }

    /// Returns true when the frame is destroyed: the head or the torso is gone.
    pub fn ruined(&self) -> bool {
        self.of(Part::Head) <= 0.0 || self.of(Part::Torso) <= 0.0
    }

    /// Returns true when every part is whole.
    pub fn intact(&self) -> bool {
        self.whole.iter().all(|health| *health >= 1.0 - 1e-4)
    }

    /// Returns the health of every part in the order of [`Part::ALL`], to keep in a save.
    pub fn saved(&self) -> [f32; PARTS] {
        self.whole
    }

    /// Returns the condition a save kept, each health held from 0 to 1 and a value that is not a
    /// number read as whole.
    pub fn restored(saved: [f32; PARTS]) -> Self {
        let kept = |health: f32| match health.is_finite() {
            true => health.clamp(0.0, 1.0),
            false => 1.0,
        };
        Self {
            whole: saved.map(kept),
        }
    }
}
