//! The height of a standing jump under standard gravity.

use bevy::prelude::*;

use crate::player::controller::Motion;
use crate::player::intent::{Intent, land_vertical};
use crate::universe::STANDARD_GRAVITY;

/// Height in metres a standing jump rose in the first tuning: 7 m/s under 22 m/s squared.
const TODAY: f32 = 7.0 * 7.0 / (2.0 * 22.0);

/// Returns how high in metres a standing jump rises under `gravity`, stepped at 240 frames a second.
fn rise(gravity: f32) -> f32 {
    let dt = 1.0 / 240.0;
    let asks = Intent {
        wish: Vec3::ZERO,
        forward: 0.0,
        right: 0.0,
        sprint: false,
        jump: true,
        yaw: 0.0,
        steer: false,
    };
    let mut motion = Motion {
        grounded: true,
        ..default()
    };
    motion.velocity.y = land_vertical(&mut motion, &asks, gravity, dt);
    let (mut height, mut top) = (0.0f32, 0.0f32);
    while motion.velocity.y > 0.0 {
        height += motion.velocity.y * dt;
        top = top.max(height);
        let calm = Intent {
            jump: false,
            ..asks
        };
        motion.velocity.y = land_vertical(&mut motion, &calm, gravity, dt);
    }
    top
}

/// A standing jump on a planet of standard gravity rises within 5 percent of the height it
/// had under the 22 m/s squared the robot was first tuned for.
#[test]
fn a_standing_jump_rises_as_high_as_before() {
    let height = rise(STANDARD_GRAVITY as f32);
    assert!(
        (height / TODAY - 1.0).abs() < 0.05,
        "rose {height} m, was {TODAY} m"
    );
}
