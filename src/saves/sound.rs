//! A saved place that is somewhere: the check a place read from a file or sent by a server
//! goes through before the world starts there. The world itself checks the height, which
//! needs the ground of the body.

use bevy::math::DVec3;

use super::model::Place;

/// Shortest squared direction that still points somewhere.
const POINTED: f64 = 1.0e-12;

impl Place {
    /// Returns true when the direction is made of numbers and has a length, so the place is
    /// somewhere on its body.
    pub fn pointed(&self) -> bool {
        let dir = DVec3::from(self.dir);
        dir.is_finite() && dir.length_squared() > POINTED
    }
}
