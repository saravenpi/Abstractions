//! What a damaged torso, head, wing or boot does to the system that drives it.

use super::effects::keep;
use super::tune::{BOOT_THRUST, HEAD_SCAN, TORSO_CARRY, TORSO_RECOVERY, WING_LIFT, WING_PULL};
use super::{Condition, Part};

impl Condition {
    /// Returns the share of its carrying capacity the torso leaves.
    pub fn carry(&self) -> f32 {
        keep(self.of(Part::Torso), TORSO_CARRY)
    }

    /// Returns the share of its swing recovery speed the torso leaves.
    pub fn recovery(&self) -> f32 {
        keep(self.of(Part::Torso), TORSO_RECOVERY)
    }

    /// Returns the share of its companion scan reach the head leaves, which is also how clean the
    /// picture of the view and the HUD stays.
    pub fn scan(&self) -> f32 {
        keep(self.of(Part::Head), HEAD_SCAN)
    }

    /// Returns the share of its lift each wing keeps, left then right.
    pub fn wings(&self) -> [f32; 2] {
        [Part::WingL, Part::WingR].map(|wing| keep(self.of(wing), WING_LIFT))
    }

    /// Returns the share of its lift the wings keep together.
    pub fn lift(&self) -> f32 {
        let [left, right] = self.wings();
        (left + right) / 2.0
    }

    /// Returns the share of its thrust the boots keep.
    pub fn boots(&self) -> f32 {
        keep(
            (self.of(Part::BootL) + self.of(Part::BootR)) / 2.0,
            BOOT_THRUST,
        )
    }

    /// Returns the roll stick the weaker wing pulls the body toward, from -1 (left) to 1 (right).
    pub fn pull(&self) -> f32 {
        let [left, right] = self.wings();
        (left - right) * WING_PULL
    }
}
