//! Pure steering helpers: angles, turning toward a heading and stepping toward a point.

use std::f32::consts::{FRAC_PI_2, PI, TAU};

use bevy::prelude::*;

/// Wraps an angle into `[-PI, PI)`.
pub fn wrap_angle(angle: f32) -> f32 {
    (angle + PI).rem_euclid(TAU) - PI
}

/// Turns `heading` toward `desired` by at most `max_turn` radians along the shortest way.
pub fn turn_toward(heading: f32, desired: f32, max_turn: f32) -> f32 {
    let delta = wrap_angle(desired - heading);
    wrap_angle(heading + delta.clamp(-max_turn, max_turn))
}

/// Returns the heading from `from` to `to` on the XZ plane, where `(cos, sin)` maps to `(x, z)`.
pub fn bearing(from: Vec3, to: Vec3) -> f32 {
    (to.z - from.z).atan2(to.x - from.x)
}

/// Returns the rotation about Y that makes an object's forward axis follow `heading`.
pub fn yaw(heading: f32) -> f32 {
    -heading - FRAC_PI_2
}

/// Moves `current` toward `target` by at most `step` without overshooting.
pub fn approach(current: f32, target: f32, step: f32) -> f32 {
    current + (target - current).clamp(-step, step)
}

/// Moves `from` toward `to` by at most `step` meters without overshooting.
pub fn step_toward(from: Vec3, to: Vec3, step: f32) -> Vec3 {
    from + (to - from).clamp_length_max(step)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Turning takes the short way, respects the rate and the yaw faces the heading.
    #[test]
    fn steering_follows_the_shortest_turn() {
        let h = turn_toward(3.0, -3.0, 0.1);
        assert!(h > 3.0 || h < -3.0);
        assert!((wrap_angle(h - 3.1)).abs() < 1e-4);
        assert!((turn_toward(0.0, 0.05, 0.5) - 0.05).abs() < 1e-6);
        let from = Vec3::ZERO;
        let to = Vec3::new(3.0, 5.0, 4.0);
        let facing = Quat::from_rotation_y(yaw(bearing(from, to))) * Vec3::NEG_Z;
        assert!((facing - Vec3::new(0.6, 0.0, 0.8)).length() < 1e-4);
        assert_eq!(step_toward(from, to, 100.0), to);
        assert!((step_toward(from, to, 1.0).length() - 1.0).abs() < 1e-5);
        assert_eq!(approach(1.0, 5.0, 0.5), 1.5);
    }
}
