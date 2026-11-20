//! The view preferences: first or third person, on foot and in the ship, and the comfort options.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::io::Kind;
use super::tune::{ZOOM_RANGE, ZOOM_STEP};

/// Where the camera sits.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ViewMode {
    /// Through the eyes of the body, or from the pilot seat.
    #[default]
    First,
    /// Behind the body, or behind the ship.
    Third,
}

impl ViewMode {
    /// Returns the other mode.
    pub fn toggled(self) -> Self {
        match self {
            Self::First => Self::Third,
            Self::Third => Self::First,
        }
    }
}

/// What the player chose for the camera. It is meant to be saved with the settings.
#[derive(Resource, Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(default)]
pub struct ViewPrefs {
    /// View while walking, swimming and flying.
    pub on_foot: ViewMode,
    /// View while seated in the ship.
    pub ship: ViewMode,
    /// Whether the first person camera bobs with each step.
    pub head_bob: bool,
    /// Whether the third person camera drifts behind the direction of travel when left alone.
    pub auto_follow: bool,
    /// Third person zoom, as a multiple of the boom of the rig. The wheel changes it.
    pub zoom: f32,
    /// Strength of the speed effects, from 0 (off) to 1 (full): the widening of the view, the
    /// shake of the air and the streaks of the wind. At 0 the view keeps the field of view that
    /// was set and nothing shakes.
    pub speed_fx: f32,
}

impl Default for ViewPrefs {
    fn default() -> Self {
        Self {
            on_foot: ViewMode::Third,
            ship: ViewMode::First,
            head_bob: true,
            auto_follow: true,
            zoom: 1.0,
            speed_fx: 1.0,
        }
    }
}

impl ViewPrefs {
    /// Returns the mode chosen for a target of `kind`.
    pub fn mode(&self, kind: Kind) -> ViewMode {
        match kind {
            Kind::Ship => self.ship,
            Kind::OnFoot | Kind::Flying => self.on_foot,
        }
    }

    /// Switches between first and third person for a target of `kind`, and leaves the other preference alone.
    pub fn toggle(&mut self, kind: Kind) {
        match kind {
            Kind::Ship => self.ship = self.ship.toggled(),
            Kind::OnFoot | Kind::Flying => self.on_foot = self.on_foot.toggled(),
        }
    }

    /// Zooms by `notches` wheel notches, positive toward the body, within the limits.
    pub fn zoom_by(&mut self, notches: f32) {
        let zoom = self.zoom * ZOOM_STEP.powf(notches);
        self.zoom = zoom.clamp(ZOOM_RANGE.0, ZOOM_RANGE.1);
    }
}
