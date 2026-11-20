//! Tuning constants of strikes: the reading of the mouse, the guard, the hitstop and the flight of a
//! thrown spear. Lengths are metres in the frame of the torso: +X right, +Y up, -Z ahead.

/// Seconds of mouse motion that choose the direction of a swing (Mount & Blade reads it just before
/// the click).
pub const WINDOW: f32 = 0.12;

/// Mouse samples kept, more than the frames of [`WINDOW`] at any frame rate a player plays at.
pub const STROKES: usize = 32;

/// Least mouse motion over the window that picks a new direction, in pixels.
pub const DEADZONE: f32 = 3.0;

/// Seconds a raised guard parries a blow.
pub const PARRY: f32 = 0.15;

/// Seconds a raised guard parries a blow with a very light tool.
pub const PARRY_LIGHT: f32 = 0.10;

/// Mass under which a tool counts as very light, in kilograms.
pub const LIGHT: f32 = 0.5;

/// Share of the mass in the head above which a build chops or smashes overhead, unitless.
pub const HEAVY_HEAD: f32 = 0.6;

/// Seconds from the end of a swing back to carrying the tool.
pub const RECOVER: f32 = 0.25;

/// Seconds to raise the guard.
pub const GUARD_IN: f32 = 0.1;

/// Seconds to draw a spear back fully before a throw.
pub const DRAW: f32 = 0.8;

/// Share of a full draw under which a release throws nothing, unitless.
pub const LEAST_DRAW: f32 = 0.25;

/// Longest kickback stun after a blow that gave none of its energy, in seconds.
pub const KICK: f32 = 0.1;

/// Distance from the shoulder line to the circle the grip sweeps in a swing, in metres.
pub const GRIP_RADIUS: f32 = 0.36;

/// Distance from the wrist to the middle of the grip in the hand, in metres.
pub const HAND_GRIP: f32 = 0.05;

/// Distance along the shaft from the leading hand to the second hand of a two-handed grip, in metres.
pub const SECOND_HAND: f32 = 0.12;

/// Turn of the chest over the hips at the ends of a sideways swing, in radians.
pub const TWIST: f32 = 0.45;

/// Bend of the chest at the ends of an overhead swing, in radians.
pub const SPINE: f32 = 0.35;

/// Share of the energy a blow keeps when it lands at the grip, unitless: the sweet spot is the head.
pub const SWEET_FLOOR: f32 = 0.3;

/// Hitstop frames: the least, the gain per square root joule, and the most (Smash scales it by
/// damage, https://www.ssbwiki.com/Hitlag).
pub const STOP: [f32; 3] = [3.0, 0.4, 12.0];

/// Frame rate the hitstop frames count at, in frames per second.
pub const STOP_FPS: f32 = 60.0;

/// Shake of the struck animal at the start of a hitstop, in metres; it falls to 0 by its end.
pub const SHAKE: f32 = 0.03;

/// Shake of the camera at the start of a hitstop that involves the local robot, in metres.
pub const CAMERA_SHAKE: f32 = 0.012;

/// Pressure soil puts against a point driven into it, in pascals.
pub const SOIL: f32 = 1.0e6;

/// Least depth a spear must reach in the ground to stand in it, in metres.
pub const STICK: f32 = 0.05;

/// Share of its speed a spear keeps when it bounces, unitless.
pub const RESTITUTION: f32 = 0.3;

/// Speed under which a bouncing spear comes to rest, in metres per second.
pub const REST_SPEED: f32 = 1.0;

/// Distance within which the robot picks a lying spear up, in metres.
pub const PICK: f32 = 1.5;

/// Upward tilt of a throw over the view, in radians.
pub const LOFT: f32 = 0.08;
