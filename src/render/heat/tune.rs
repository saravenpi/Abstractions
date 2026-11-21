//! Constants of the heat of an atmospheric entry.

/// Sutton-Graves constant of the convective heating of the nose, in kilograms to the half per
/// metre: 1.74e-4 for air, from <https://en.wikipedia.org/wiki/Atmospheric_entry>.
pub const SUTTON_GRAVES: f64 = 1.74e-4;

/// Radius of the curve of the nose in metres. A blunter nose heats less, as one over its root.
pub const NOSE_RADIUS: f64 = 0.5;

/// Heat flux at the nose in watts per square metre at which the glow starts and at which it is
/// full. At the first a nose that radiates its heat away sits near 1,200 K and shows a dull red;
/// at the second, near the peak of an entry from a low orbit, it sits near 2,550 K. The share
/// in between follows the logarithm of the flux, which spans a factor 20.
pub const HEATING: (f64, f64) = (1.0e5, 2.0e6);

/// Emissivity of the hot skin of the nose, from 0 to 1: about 0.85 for the tiles of a heat
/// shield.
pub const EMISSIVITY: f64 = 0.85;

/// Stefan-Boltzmann constant in watts per square metre per kelvin to the fourth.
pub const STEFAN_BOLTZMANN: f64 = 5.670_374e-8;

/// Dynamic pressure in pascals at which the roar and the shake of an entry start and at which
/// they are full.
pub const PRESSURE: (f64, f64) = (2.0e3, 6.0e4);

/// Rate per second at which the glow rises toward its value and at which it dies out. The heat
/// of a hull builds up and cools slowly, so it never steps.
pub const EASE: (f32, f32) = (3.0, 0.8);

/// Rate per second at which the colour of the glow follows the gas the ship flies through.
pub const HUE_RATE: f32 = 4.0;

/// Heat share that makes a craft show its sheath; under it nothing is drawn.
pub const SHEATH_FROM: f32 = 0.02;

/// Where the sphere that holds the sheath sits, in radii of the craft: how far behind the
/// centre of the craft along its flight, and its radius. It reaches from just ahead of the bow
/// shock to the end of the wake, ten radii behind.
pub const SHEATH_BOUND: (f32, f32) = (4.4, 5.8);

/// Luminous power in lumens of the light the hot shock casts on the hull at full heat: about
/// 2,000 lux on the hull three metres from the nose. It casts no shadow.
pub const HULL_LUMENS: f32 = 2.0e5;

/// Speed in metres per second under which the nose of a craft does not glow in any air of the
/// game: it would take air of 680 kilograms per cubic metre to glow at this speed.
pub const MIN_SPEED: f64 = 250.0;
