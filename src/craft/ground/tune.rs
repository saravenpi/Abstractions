//! Tuning constants of the ground: where a thing is put down, how near it is taken and drawn.

/// Distance ahead of the robot, past half the length of the thing, at which it is put down, in
/// metres: clear of the robot's body, at its feet.
pub const AHEAD: f32 = 0.6;

/// Turns about the robot's up, in radians, at which a tool is tried in turn when another thing
/// lies where it would go: ahead, then to each side, then further around the feet.
pub const TURNS: [f32; 7] = [0.0, 0.8, -0.8, 1.6, -1.6, 2.4, -2.4];

/// Seconds a place a thing was put down at stays taken before the ground tells of the thing.
pub const RECENT: f64 = 10.0;

/// Longest drop from the middle of the robot down to the ground under a thing put down, in metres.
pub const FALL: f32 = 3.0;

/// Farthest from the robot a thing can be and still be taken, in metres.
pub const REACH: f32 = 2.5;

/// Farthest from the line of sight a thing can lie and still be the one taken, in metres.
pub const AIM: f32 = 0.8;

/// Distance from the robot within which the things on the ground are drawn, in metres.
pub const DRAW: f64 = 150.0;

/// Seconds between two looks for things that came into or left the drawing distance.
pub const LOOK: f32 = 0.5;

/// Smallest and largest side of a drawn heap, in metres: a heap is a cube of its volume.
pub const HEAP_SIDE: (f32, f32) = (0.04, 0.8);

/// Thinnest a drawn tool is across, in metres.
pub const THIN: f32 = 0.03;

/// Mass of free room under which the pack takes nothing of a heap, in kilograms.
pub const CRUMB: f32 = 0.01;
