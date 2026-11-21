//! The lock of the body a course is aimed at, shrunk to its arrival shell.

use super::Skies;

impl Skies {
    /// Sets the lock of the well of the body a course is aimed at to the radius of its arrival shell,
    /// when the ship is in the system of that body.
    pub(super) fn shrink_target(&mut self) {
        let Some((body, lock)) = self.arrive else {
            return;
        };
        let Some((_, system)) = self.system.as_ref().filter(|(star, _)| *star == body.star) else {
            return;
        };
        let rank = system.planets.iter().position(|planet| planet.id == body);
        let well = rank.and_then(|rank| self.wells.get_mut(rank + 1));
        if let Some(well) = well {
            well.lock = lock;
        }
    }
}
