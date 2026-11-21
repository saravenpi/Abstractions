//! What the companion may look at: the scene around it, the things in it, and where each one is.
//!
//! None of it follows the camera: the player's view only counts for the short moment it looks
//! back at a player who has been watching it.

use bevy::prelude::*;

use super::scene::Scene;
use super::spot::Stance;
use super::tune::{GAZE_WEIGHT, YIELD};

/// Angle in radians from straight ahead beyond which the eyes turn toward a target.
const GLANCE: f32 = 0.3;

/// Distance in meters of the point it looks at when it looks where the player is heading.
const HORIZON: f32 = 12.0;

/// Distance in meters of the point it looks at when it looks around at nothing.
const WANDER_REACH: f32 = 8.0;

impl Stance {
    /// Returns `pos` pushed sideways out of the cone around the line of sight, so the companion
    /// never floats in front of what the player looks at. Spots behind the eyes stay put.
    pub fn aside(&self, pos: Vec3) -> Vec3 {
        let offset = pos - self.eye;
        let along = offset.dot(self.forward);
        let across = offset - self.forward * along;
        let least = along * YIELD;
        if along <= 0.0 || across.length() >= least {
            return pos;
        }
        let side = across.try_normalize().unwrap_or_else(|| self.right());
        self.eye + self.forward * along + side * least
    }
}

/// What the companion looks at.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Focus {
    /// The player's robot.
    #[default]
    Player,
    /// The way a fast player travels.
    Travel,
    /// The ship parked nearby.
    Ship,
    /// The door of the home base.
    Home,
    /// A look around at nothing in particular.
    Wander,
    /// The camera, for a moment, when the player has been watching it.
    Camera,
}

impl Focus {
    /// What it chooses among when its eyes move on, in the order of `GAZE_WEIGHT`.
    pub const CHOICES: [Focus; 5] = [
        Focus::Player,
        Focus::Travel,
        Focus::Ship,
        Focus::Home,
        Focus::Wander,
    ];

    /// Returns the sight this focus looks at in [`Scene::sights`], if it is one.
    fn sight(self) -> Option<usize> {
        match self {
            Focus::Ship => Some(0),
            Focus::Home => Some(1),
            _ => None,
        }
    }

    /// Returns true while there is something to look at for this focus in `scene`.
    pub fn open(self, scene: &Scene) -> bool {
        match (self, self.sight()) {
            (_, Some(index)) => scene.sights[index].is_some(),
            (Focus::Travel, _) => scene.course.is_some(),
            (Focus::Wander | Focus::Camera, _) => scene.course.is_none(),
            _ => true,
        }
    }

    /// Returns how likely this focus is to be chosen in `scene` after `now`. It never picks the
    /// same thing twice in a row, but the way of a fast run.
    pub fn weight(self, scene: &Scene, now: Focus) -> f32 {
        let index = Self::CHOICES.iter().position(|focus| *focus == self);
        let again = self == now && self != Focus::Travel;
        let open = self.open(scene) && !again;
        index.map_or(0.0, |index| GAZE_WEIGHT[index] * f32::from(u8::from(open)))
    }

    /// Returns the shortest and longest time in seconds the eyes stay on this focus.
    pub fn dwell(self) -> (f32, f32) {
        match self {
            Focus::Player => (1.6, 3.6),
            Focus::Travel => (2.5, 5.0),
            Focus::Wander => (2.0, 4.2),
            Focus::Camera => (1.2, 2.2),
            _ => (2.2, 4.5),
        }
    }

    /// Returns the render position to look at in `scene`. `wander` is the turn to the left and
    /// the rise, in radians, of a look around, from the way away from the player.
    pub fn point(self, scene: &Scene, wander: Vec2) -> Vec3 {
        let thing = self.sight().and_then(|index| scene.sights[index]);
        match self {
            Focus::Travel => scene.eye + scene.course.unwrap_or(Vec3::ZERO) * HORIZON,
            Focus::Wander => scene.here + scene.around(wander) * WANDER_REACH,
            Focus::Camera => scene.camera,
            _ => thing.unwrap_or(scene.eye),
        }
    }
}

/// Returns where the eyes turn for a target at `local`, a direction in the body's own axes.
///
/// The screen faces -Z. The result is in picture axes: x toward the right of the picture, which
/// is the body's -X, and y up. Each axis is -1, 0 or 1.
pub fn eye_shift(local: Vec3) -> IVec2 {
    let side = local.x.atan2(-local.z);
    let rise = local.y.atan2(Vec2::new(local.x, local.z).length());
    let step = |angle: f32| i32::from(angle > GLANCE) - i32::from(angle < -GLANCE);
    IVec2::new(-step(side), step(rise))
}
