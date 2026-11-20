//! The numbers of the field acts: how a sound travels in two bytes, how long an echo waits for its
//! animal, how big a robot is to a swing.

/// Seconds an echo of a wound waits for its animal to be drawn here. After that the blood it lost is
/// lost for good, and the wound is dropped.
pub const WOUND_KEEP: f64 = 600.0;

/// Seconds an attack of an animal may be late when it arrives. A later one is not played.
pub const ATTACK_LATE: f64 = 2.0;

/// Radius in metres of the sphere around the middle of a robot that a swing must enter to hit it.
pub const BODY_RADIUS: f32 = 0.7;

/// The power of two, in gigapascals, of the stiffness code 0 of a tool: a code is the exponent plus
/// this bias.
pub const STIFFNESS_BIAS: f32 = 4.0;

/// The power of two, in kilograms, of the mass code 0 of a tool: a code is the exponent plus this
/// bias.
pub const MASS_BIAS: f32 = 6.0;
