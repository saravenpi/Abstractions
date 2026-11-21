//! Tuning constants of the pack: what the robot's frame lifts and how a load slows it.

/// Mass the robot's frame carries under home gravity, in kilograms.
pub const CARRY: f32 = 25.0;

/// Gravity the carried mass is measured under, in metres per second squared.
pub const HOME_GRAVITY: f32 = 9.81;

/// Share of the carried mass above which walking slows, unitless.
pub const SLOW_FROM: f32 = 0.6;

/// Share of the walking speed left at a full load, unitless.
pub const SLOWEST: f32 = 0.5;

/// Mass of sinew or resin one binding takes, in kilograms.
pub const BINDING: f32 = 0.02;

/// Mass under which a stack is used up, in kilograms.
pub const CRUMB: f32 = 1.0e-4;

/// Most mass of rock one pick breaks off, in kilograms: what the robot's hands and its tool part from
/// the stone in one go. It is a tenth of what the frame carries, so a full pack takes several picks.
pub const HANDFUL: f32 = 2.5;

/// Farthest from the robot a rock can be and still be broken up, in metres.
pub const REACH: f32 = 2.5;

/// Longest ray cast from the camera to find the rock under the crosshair, in metres. It is longer
/// than [`REACH`] because the camera may stand behind the robot.
pub const SIGHT: f32 = 12.0;

/// The key that picks a stone up off the ground.
pub const PICK_KEY: bevy::input::keyboard::KeyCode = bevy::input::keyboard::KeyCode::KeyH;

/// Kilograms of each stone underfoot the workshop may draw on to make the kit, more than any of its
/// tools weighs. The kit takes nothing from the pack: the robot wakes with its tools made.
pub const KIT_STONE: f32 = 10.0;
