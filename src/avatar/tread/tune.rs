//! Tuning constants of the feet on the ground. Masses are in kilograms, speeds in metres per
//! second, lengths in metres and times in seconds.

/// Mass of the robot.
pub const BODY_KG: f32 = 80.0;

/// Share of the mass of a body in one leg: thigh, shank and foot, from Winter, "Biomechanics and
/// Motor Control of Human Movement" (Wiley, 4th edition 2009), table 4.1.
pub const LEG_SHARE: f32 = 0.161;

/// Speed into the ground at which a landing turns hard, as the filler reads it: the strike of a
/// foot is measured against it.
pub const HARD: f32 = 9.0;

/// Share of the speed of its foot a crouched body keeps as it sets the foot down: it creeps.
pub const CREEP: f32 = 0.5;

/// Share of the time a foot is down that the roll from heel to toe takes, and the shortest and
/// longest roll.
pub const ROLL: (f32, f32, f32) = (0.5, 0.02, 0.15);

/// Longest time a foot stays down, and how long the feet of a landing stay down before the
/// body moves on.
pub const STANCE: (f32, f32) = (0.6, 0.3);

/// Slowest fall that lands with a sound: a step down from a kerb.
pub const LAND_FROM: f32 = 1.0;

/// Seconds after a landing in which the gait sends no step: the feet are already down.
pub const HUSH: f32 = 0.2;

/// Seconds between two scrapes of a slide, and the share of the speed of a slide that drives
/// the scrape into the ground.
pub const SCRAPE: (f32, f32) = (0.1, 0.25);

/// Height over the ground under which a foot stands on the ground, not on something built.
pub const RAISED: f32 = 0.3;

/// Depth of water under which a foot only wets the sole, and over which the body swims.
pub const WADE: (f32, f32) = (0.03, 0.6);

/// Depth of a river: the world gives its bed, not its water.
pub const RIVER: f32 = 0.2;

/// Slope under which the rock of high land lies as loose stones.
pub const GRAVEL_SLOPE: f32 = 0.35;

/// Rain, from 0 to 1, over which soil and paths are mud.
pub const MUDDY: f32 = 0.4;
