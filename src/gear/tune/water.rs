//! Tuning constants of the boot water jet: the liquids, the body and the nozzles.

/// Density of liquid water in kilograms per cubic metre.
pub const WATER_DENSITY: f32 = 1000.0;

/// Density of molten basalt in kilograms per cubic metre, the sea of a lava world.
pub const MAGMA_DENSITY: f32 = 2700.0;

/// Mass of the robot with its gear in kilograms.
///
/// It matches the drag of the air: [`super::AERO`] times [`super::BODY_DRAG`] gives a drag
/// area of about 0.24 square metres on this mass, the head down area of a skydiver.
pub const BODY_MASS: f32 = 90.0;

/// Area in square metres of the two boot nozzles together: two round nozzles of 5 cm radius.
pub const NOZZLE_AREA: f32 = 0.0157;

/// Airspeed in metres per second at which the boots deliver their power in the air: the middle
/// of the level top speed of powered flight at home. The power of the boots is the thrust
/// [`super::THRUST_G`] times this speed, in watts per kilogram.
pub const POWER_SPEED: f32 = 75.0;

/// Steps of the bisection that finds the speed of the jet: a millionth of the bracket.
pub const PUMP_STEPS: u32 = 20;
