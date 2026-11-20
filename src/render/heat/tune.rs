//! Constants of the heat of an atmospheric entry.

/// Sutton-Graves constant of the convective heating of the nose, in kilograms to the half per
/// metre: 1.74e-4 for air, from <https://en.wikipedia.org/wiki/Atmospheric_entry>.
pub const SUTTON_GRAVES: f64 = 1.74e-4;

/// Radius of the curve of the nose in metres. A blunter nose heats less, as one over its root.
pub const NOSE_RADIUS: f64 = 0.5;

/// Heat flux at the nose in watts per square metre at which the glow starts and at which it is
/// full. The planets of the game are small, so a fast fall peaks in the low thousands of watts.
pub const HEATING: (f64, f64) = (100.0, 2500.0);

/// Dynamic pressure in pascals at which the roar and the shake of an entry start and at which
/// they are full.
pub const PRESSURE: (f64, f64) = (2.0e3, 6.0e4);

/// Speed toward the ground in metres per second at which a fall counts as an entry and at which
/// it is full. Level flight, a shallow descent under power and a landing approach stay under
/// the first, whatever the heating.
pub const SINK: (f32, f32) = (30.0, 110.0);

/// Rate per second at which the glow rises toward its value and at which it dies out. The heat
/// of a hull builds up and cools slowly, so it never steps.
pub const EASE: (f32, f32) = (3.0, 0.8);

/// Rate per second at which the colour of the glow follows the gas the ship flies through.
pub const HUE_RATE: f32 = 4.0;

/// Mean surface temperature in kelvin under which a frozen world has a cold methane air.
pub const METHANE_BELOW: f64 = 150.0;

/// Heat share that makes a craft show its sheath from outside; under it nothing is drawn.
pub const SHEATH_FROM: f32 = 0.02;

/// Radius of the sheath in radii of the craft.
pub const SHEATH_REACH: f32 = 2.4;

/// Speed in metres per second under which the nose of a craft does not glow in any air of the
/// game: the thickest air glows from about 60.
pub const MIN_SPEED: f64 = 50.0;
