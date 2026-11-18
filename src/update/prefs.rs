//! The update options saved with the settings.

use serde::{Deserialize, Serialize};

/// Returns true: what an option that is on by default reads from a file written before it existed.
fn on() -> bool {
    true
}

/// Update options, saved under `[updates]` in `settings.toml`.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(default)]
pub struct UpdatePrefs {
    /// Whether the game looks for a newer release once, when it reaches the title screen.
    #[serde(default = "on")]
    pub check_at_launch: bool,
}

impl Default for UpdatePrefs {
    fn default() -> Self {
        Self {
            check_at_launch: on(),
        }
    }
}
