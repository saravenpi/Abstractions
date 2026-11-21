//! The scene: what the companion sees around it on one frame, in render space.

use bevy::prelude::*;

use super::tune::FAST_SPEED;

/// What the companion sees this frame, in render space. The things are only there when close.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Scene {
    /// Position of the companion.
    pub here: Vec3,
    /// Unit direction its head faces.
    pub heading: Vec3,
    /// Position of the player's eyes, on the head of the player's robot.
    pub eye: Vec3,
    /// Unit local vertical.
    pub up: Vec3,
    /// Velocity of the player in meters per second.
    pub drift: Vec3,
    /// Unit direction a fast player travels in: its course on foot, the nose of the ship seated.
    pub course: Option<Vec3>,
    /// Position of the camera.
    pub camera: Vec3,
    /// Unit view direction of the camera.
    pub view: Vec3,
    /// The ship and the door of the home base, when each is close.
    pub sights: [Option<Vec3>; 2],
}

impl Scene {
    /// Returns the horizontal way its head faces, or any horizontal way before it is known.
    pub fn ahead(&self) -> Vec3 {
        self.level(self.heading)
            .unwrap_or_else(|| self.up.any_orthonormal_vector())
    }

    /// Returns `dir` flattened onto the horizontal plane, or nothing when it is vertical.
    pub fn level(&self, dir: Vec3) -> Option<Vec3> {
        (dir - self.up * dir.dot(self.up)).try_normalize()
    }

    /// Returns the unit direction of a look around: turned left by `turn.x` from the way
    /// away from the player and raised by `turn.y`.
    pub fn around(&self, turn: Vec2) -> Vec3 {
        let away = self
            .level(self.here - self.eye)
            .unwrap_or_else(|| self.ahead());
        let flat = Quat::from_axis_angle(self.up, turn.x) * away;
        flat * turn.y.cos() + self.up * turn.y.sin()
    }
}

/// Returns the unit direction along the ground of a player moving at `drift`, when it runs
/// faster than `FAST_SPEED`.
pub fn course(drift: Vec3, up: Vec3) -> Option<Vec3> {
    let flat = drift - up * drift.dot(up);
    flat.try_normalize().filter(|_| flat.length() > FAST_SPEED)
}
