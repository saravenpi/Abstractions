//! Where the planets are at a given time.

use bevy::math::DVec3;

use super::{BodyId, Universe};

impl Universe {
    /// Returns where a planet is at `time` seconds, in metres from its star in the system frame.
    pub fn planet_position(&self, id: BodyId, time: f64) -> DVec3 {
        self.planet(id).position(time)
    }
}
