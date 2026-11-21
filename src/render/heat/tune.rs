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

/// Fastest the glow of the shock rises and falls, as a share of its full strength per second:
/// it follows the heat flux of this moment, but fades in from nothing over a few seconds as the
/// air thickens on the way in, and out over about two seconds on the way out, never stepping
/// more than a percent from one frame to the next.
pub const PLASMA_RATE: (f32, f32) = (0.3, 0.6);

/// Fastest the glow of the hot skin rises, as a share of its full glow per second: a faint
/// orange tinge at the nose first, growing over a few seconds. It falls as fast as the skin
/// cools.
pub const SKIN_RISE: f32 = 0.4;

/// Decades of red radiance under that of the full glow over which the glow of the skin fades in
/// from nothing to full: below them it is lost in daylight.
pub const SKIN_DECADES: f32 = 2.2;

/// Heat capacity of the skin of the nose and the leading edges per square metre, in joules per
/// kelvin: half a millimetre of nickel alloy, 8,000 kg/m3 times 450 J/(kg K) times 0.0005 m,
/// the thin skin of a leading edge. It sets how fast the skin heats up in an entry, over about a
/// second, and how it cools by radiation after: a skin at 2,100 K is under 1,370 K, where its
/// glow is lost in daylight, in about 4 s, under 1,110 K, where it is lost at night, in about 8 s,
/// and dark at the Draper point in about 23 s.
pub const SKIN_CAPACITY: f32 = 1800.0;

/// Temperature in kelvin of a skin that no air heats: where a skin starts warming from.
pub const AMBIENT: f32 = 280.0;

/// Temperature in kelvin at which a hot skin starts to glow at all, the Draper point of a dull
/// red seen in the dark, and at which its glow is full, a bright orange.
pub const SKIN_GLOW: (f32, f32) = (798.0, 1800.0);

/// Wavelength in metres of the red the glow of the skin is weighed at: the visible radiance of a
/// hot body falls as `1 / (exp(hc / (lambda k T)) - 1)`, by about a hundred times from 1,500 K
/// to 1,000 K.
pub const RED: f32 = 650.0e-9;

/// Longest step in seconds the temperature of the skin is integrated over at once.
pub const SKIN_STEP: f32 = 0.05;

/// Rate per second at which the colour of the glow follows the gas the ship flies through.
pub const HUE_RATE: f32 = 4.0;

/// Where the sphere that holds the sheath sits, in radii of the craft: how far behind the
/// centre of the craft along its flight, and its radius. It reaches from just ahead of the bow
/// shock to the end of the wake, ten radii behind.
pub const SHEATH_BOUND: (f32, f32) = (4.4, 5.8);

/// Speed in metres per second under which the nose of a craft does not glow in any air of the
/// game: it would take air of 680 kilograms per cubic metre to glow at this speed.
pub const MIN_SPEED: f64 = 250.0;
