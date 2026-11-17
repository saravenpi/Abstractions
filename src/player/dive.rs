//! Swimming movement: view-directed strokes, diving, buoyancy and the surface hop.

use bevy::prelude::*;

use super::intent::approach;
use super::swim::float_center;

/// Swimming speed in meters per second.
const SWIM_SPEED: f32 = 3.5;

/// Sprint stroke speed in meters per second.
const SWIM_SPRINT_SPEED: f32 = 5.5;

/// Horizontal acceleration rate while swimming.
const SWIM_ACCEL: f32 = 6.0;

/// Rate at which the vertical speed follows its target.
const VERTICAL_RATE: f32 = 8.0;

/// Strength of the spring pulling the body to its floating height.
const SPRING: f32 = 3.0;

/// Largest vertical speed of the buoyancy spring.
const MAX_SPRING_SPEED: f32 = 2.0;

/// Upward speed while the jump key is held under water.
const SWIM_UP_SPEED: f32 = 4.0;

/// Upward speed while the jump key is held near the surface.
const SURFACE_HOP_SPEED: f32 = 6.5;

/// Depth of the body center below the sea above which the surface hop starts.
const HOP_DEPTH: f32 = 0.6;

/// What the swimmer asks for this frame.
pub(super) struct SwimInput {
    pub(super) wish: Vec3,
    pub(super) sprint: bool,
    pub(super) up: bool,
}

/// Returns the unit stroke direction that follows the view, pitch included.
pub(super) fn swim_wish(forward: f32, right: f32, yaw: f32, pitch: f32) -> Vec3 {
    let view = Quat::from_rotation_y(yaw) * Quat::from_rotation_x(pitch);
    (view * Vec3::new(right, 0.0, -forward)).normalize_or_zero()
}

/// Returns the vertical speed the body aims for, from the altitude `height` of its center above the sea.
fn vertical_target(height: f32, stroke_y: f32, up: bool, time: f32) -> f32 {
    let target = float_center(time);
    let spring = ((target - height) * SPRING).clamp(-MAX_SPRING_SPEED, MAX_SPRING_SPEED);
    if up {
        let hop = if height > -HOP_DEPTH {
            SURFACE_HOP_SPEED
        } else {
            SWIM_UP_SPEED
        };
        return spring.max(hop);
    }
    if stroke_y < 0.0 {
        return stroke_y;
    }
    if stroke_y > 0.0 && height < target {
        return spring.max(stroke_y);
    }
    spring
}

/// Returns the swimming velocity for this step, in the upright frame of the body.
///
/// `height` is the altitude of the body center above the sea.
pub(super) fn swim_velocity(
    current: Vec3,
    input: &SwimInput,
    height: f32,
    time: f32,
    dt: f32,
) -> Vec3 {
    let speed = if input.sprint {
        SWIM_SPRINT_SPEED
    } else {
        SWIM_SPEED
    };
    let stroke = input.wish * speed;
    let flat = approach(
        Vec3::new(current.x, 0.0, current.z),
        Vec3::new(stroke.x, 0.0, stroke.z),
        SWIM_ACCEL,
        dt,
    );
    let wanted = vertical_target(height, stroke.y, input.up, time);
    let vertical = current.y + (wanted - current.y) * (1.0 - (-VERTICAL_RATE * dt).exp());
    Vec3::new(flat.x, vertical, flat.z)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Looking down while moving forward dives, looking up surfaces.
    #[test]
    fn stroke_follows_pitch() {
        let down = swim_wish(1.0, 0.0, 0.0, -0.8);
        let up = swim_wish(1.0, 0.0, 0.0, 0.8);
        assert!(down.y < -0.5 && up.y > 0.5);
        assert!((down.length() - 1.0).abs() < 1e-5);
        assert_eq!(swim_wish(0.0, 0.0, 1.0, 0.5), Vec3::ZERO);
    }

    /// An idle swimmer rises toward the float height and the jump key swims up.
    #[test]
    fn buoyancy_and_swim_up() {
        let deep = -3.0;
        assert!(vertical_target(deep, 0.0, false, 0.0) > 0.0);
        assert!(vertical_target(deep, 0.0, false, 0.0) <= MAX_SPRING_SPEED);
        assert_eq!(vertical_target(deep, 0.0, true, 0.0), SWIM_UP_SPEED);
        assert!(vertical_target(deep, -2.0, false, 0.0) < 0.0);
        let at_rest = vertical_target(float_center(0.0), 0.0, false, 0.0);
        assert!(at_rest.abs() < 1e-4);
    }
}
