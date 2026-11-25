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

/// Linear RGB of each mineral, in the order of [`crate::craft::matter::Mineral::ALL`].
pub const STONE_TONES: [[f32; 3]; 24] = [
    [0.10, 0.10, 0.11],
    [0.03, 0.03, 0.04],
    [0.30, 0.27, 0.23],
    [0.70, 0.80, 0.88],
    [0.88, 0.80, 0.82],
    [0.14, 0.46, 0.26],
    [0.32, 0.14, 0.11],
    [0.12, 0.56, 0.36],
    [0.86, 0.78, 0.12],
    [0.22, 0.24, 0.28],
    [0.72, 0.34, 0.18],
    [0.78, 0.64, 0.20],
    [0.44, 0.46, 0.50],
    [0.16, 0.32, 0.72],
    [0.78, 0.14, 0.14],
    [0.54, 0.24, 0.74],
    [0.16, 0.15, 0.19],
    [0.64, 0.86, 0.92],
    [0.36, 0.38, 0.44],
    [0.24, 0.22, 0.26],
    [0.86, 0.12, 0.28],
    [0.48, 0.64, 0.84],
    [0.16, 0.78, 0.68],
    [0.92, 0.82, 0.34],
];

/// Seconds a notice of what the robot picked up shows on the prompt line.
pub const NOTICE: f32 = 2.5;

/// Energy of a blow of the bare fist on a rock, in joules: a hard punch.
pub const FIST_JOULES: f32 = 30.0;

/// Young's modulus of the plate of the robot's hand, in gigapascals: an aluminium alloy.
pub const FIST_STIFFNESS: f32 = 70.0;

/// Mass of the robot's fist that rings against a rock, in kilograms.
pub const FIST_MASS: f32 = 0.5;

/// Speed a piece broken off a rock flies off at, in metres per second.
pub const PIECE_SPEED: f32 = 2.5;

/// Kilograms of each stone underfoot the workshop may draw on to make the kit, more than any of its
/// tools weighs. The kit takes nothing from the pack: the robot wakes with its tools made.
pub const KIT_STONE: f32 = 10.0;
