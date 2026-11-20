//! Tuning constants of mending the robot. Health is a share from 0 to 1.

use bevy::input::keyboard::KeyCode;

/// The key that mends the worst part with what the robot carries.
pub const MEND_KEY: KeyCode = KeyCode::KeyU;

/// Distance from the home base within which the robot mends itself, in metres.
pub const BAY: f32 = 18.0;

/// Health the base gives back to every part each second.
pub const BAY_RATE: f32 = 0.15;

/// Health one patch in the field gives back.
pub const PATCH: f32 = 0.25;

/// Health a patch in the field cannot lift a part above: only the base makes it whole.
pub const PATCH_CAP: f32 = 0.8;

/// Mass of hard material a plate patch takes, in kilograms.
pub const PLATE_MASS: f32 = 0.15;

/// Mass of binding a joint patch takes, in kilograms.
pub const BIND_MASS: f32 = 0.05;

/// Least Mohs hardness of a material that patches a plate: bone and horn hold, hide and wood do not.
pub const HARD: f32 = 2.5;
