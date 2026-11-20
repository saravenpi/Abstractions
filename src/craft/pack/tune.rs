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

/// Mass of stone one pick from the ground takes, in kilograms.
pub const HANDFUL: f32 = 0.5;

/// Seconds between two picks from the ground.
pub const PICK_GAP: f32 = 0.6;

/// The key that picks a stone up off the ground.
pub const PICK_KEY: bevy::input::keyboard::KeyCode = bevy::input::keyboard::KeyCode::KeyH;
