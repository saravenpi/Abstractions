//! Which markers show over the world: a three-step choice kept in two plain switches, so a
//! settings file of 0.10, which only has the on or off switch, loads and means what it meant.

use super::Gameplay;

/// Which markers show over the world.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldMarkers {
    /// None. The compass, the minimap and the map still show every marker.
    Off,
    /// Only what the player tracks and who plays with them: the course, the active pin and
    /// friends.
    Minimal,
    /// Every marker: the course, pins, friends, home and the ship.
    All,
}

impl WorldMarkers {
    /// Every step, from the fewest markers.
    pub const ALL: [WorldMarkers; 3] = [Self::Off, Self::Minimal, Self::All];

    /// Returns the name of the step, as the settings show it.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::Minimal => "Minimal",
            Self::All => "All",
        }
    }

    /// Returns the step `dir` places from this one, staying on the first and the last.
    pub fn moved(self, dir: i32) -> Self {
        let at = Self::ALL.iter().position(|step| *step == self).unwrap_or(2) as i32;
        Self::ALL[(at + dir).clamp(0, 2) as usize]
    }
}

impl Gameplay {
    /// Returns which markers show over the world.
    pub const fn markers(&self) -> WorldMarkers {
        match (self.world_markers, self.markers_minimal) {
            (false, _) => WorldMarkers::Off,
            (true, true) => WorldMarkers::Minimal,
            (true, false) => WorldMarkers::All,
        }
    }

    /// Sets which markers show over the world.
    pub fn set_markers(&mut self, level: WorldMarkers) {
        self.world_markers = level != WorldMarkers::Off;
        self.markers_minimal = level == WorldMarkers::Minimal;
    }
}
