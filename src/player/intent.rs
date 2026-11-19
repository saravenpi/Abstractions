//! Pure movement rules: what a body is asked, its direction, smoothing and vertical speed on land.

use bevy::prelude::*;

use super::controller::Motion;

/// Upward speed of a jump.
const JUMP_SPEED: f32 = 7.0;

/// Seconds after leaving the ground during which a jump still works.
const COYOTE_TIME: f32 = 0.12;

/// Fastest allowed fall speed.
const TERMINAL_SPEED: f32 = 50.0;

/// What the player asks for this frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Intent {
    pub(super) wish: Vec3,
    pub(super) forward: f32,
    pub(super) right: f32,
    pub(super) sprint: bool,
    pub(super) jump: bool,
    pub(super) yaw: f32,
    pub(super) steer: bool,
}

/// Returns the unit horizontal direction for forward and right axes at a yaw angle.
pub(super) fn wish_direction(forward: f32, right: f32, yaw: f32) -> Vec3 {
    (Quat::from_rotation_y(yaw) * Vec3::new(right, 0.0, -forward)).normalize_or_zero()
}

/// Returns -1, 0 or 1 from a pair of held keys.
pub(super) fn axis(keys: &ButtonInput<KeyCode>, positive: KeyCode, negative: KeyCode) -> f32 {
    f32::from(keys.pressed(positive)) - f32::from(keys.pressed(negative))
}

/// Moves a horizontal velocity toward a target with exponential smoothing.
pub(super) fn approach(current: Vec3, target: Vec3, rate: f32, dt: f32) -> Vec3 {
    current.lerp(target, 1.0 - (-rate * dt).exp())
}

/// Returns the vertical speed on land under `gravity`, in meters per second squared, and updates
/// the coyote timer.
pub(super) fn land_vertical(motion: &mut Motion, intent: &Intent, gravity: f32, dt: f32) -> f32 {
    motion.coyote = if motion.grounded {
        COYOTE_TIME
    } else {
        motion.coyote - dt
    };
    if intent.jump && motion.coyote > 0.0 {
        motion.coyote = 0.0;
        motion.grounded = false;
        return JUMP_SPEED;
    }
    if motion.grounded {
        return 0.0;
    }
    (motion.velocity.y - gravity * dt).max(-TERMINAL_SPEED)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Forward at zero yaw points down -Z, and diagonals stay unit length.
    #[test]
    fn wish_direction_follows_yaw() {
        assert!((wish_direction(1.0, 0.0, 0.0) - Vec3::NEG_Z).length() < 1e-5);
        let turned = wish_direction(1.0, 0.0, std::f32::consts::FRAC_PI_2);
        assert!((turned - Vec3::NEG_X).length() < 1e-5);
        assert!((wish_direction(1.0, 1.0, 0.3).length() - 1.0).abs() < 1e-5);
        assert_eq!(wish_direction(0.0, 0.0, 1.0), Vec3::ZERO);
    }

    /// Jumping needs ground or coyote time.
    #[test]
    fn jump_requires_ground() {
        let intent = Intent {
            wish: Vec3::ZERO,
            forward: 0.0,
            right: 0.0,
            sprint: false,
            jump: true,
            yaw: 0.0,
            steer: false,
        };
        let mut airborne = Motion {
            coyote: -1.0,
            ..default()
        };
        assert!(land_vertical(&mut airborne, &intent, 22.0, 0.016) < JUMP_SPEED);
        let mut grounded = Motion {
            grounded: true,
            ..default()
        };
        assert_eq!(
            land_vertical(&mut grounded, &intent, 22.0, 0.016),
            JUMP_SPEED
        );
    }
}
