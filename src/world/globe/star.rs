//! The star over a globe: where it stands over one day, and with the climate, which places are fair.

use super::Globe;
use crate::climate::Fair;

impl Globe {
    /// Returns the light and the warmth of the planet, for asking how fair a place is: whether the
    /// star climbs high over a day there and the mean air is mild. The shores a new game starts
    /// at and the home base are picked with it.
    pub fn fair(&self) -> Fair {
        Fair::of(&self.setting, self.star)
    }
}
