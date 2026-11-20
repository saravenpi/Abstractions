//! Units: the physical constants the universe is measured against.

/// Standard gravity in metres per second squared: the pull at the surface of the Earth, the
/// unit of "1 g" for displays and conversions. No planet is pinned to it.
pub const STANDARD_GRAVITY: f64 = 9.806_65;

/// Mean radius of the Earth in metres, the unit planet sizes are compared in.
pub const EARTH_RADIUS: f64 = 6.371e6;

/// Metres in one astronomical unit, the distance at which a star of luminosity 1 gives a flux of 1.
pub const AU: f64 = 1.495_978_707e11;

/// Mass of the Sun in kilograms, the unit of the star class table.
pub const SUN_MASS: f64 = 1.988_47e30;

/// Radius of the Sun in metres, the unit star radii are scaled from.
pub const SUN_RADIUS: f64 = 6.957e8;

/// Surface temperature of the Sun in kelvin, the unit star temperatures are scaled from.
pub const SUN_TEMPERATURE: f64 = 5780.0;

/// Seconds in one year, the unit star ages are told in.
pub const YEAR: f64 = 3.155_76e7;

/// Watts per square metre in a flux of 1: the sunlight the Earth receives.
pub const STARLIGHT: f64 = 1361.0;
