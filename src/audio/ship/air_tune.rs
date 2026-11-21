//! Tuning constants of the air on the airframe of a ship: the wind, the buffet, the rattle, the
//! roar of an entry, how the torn flow, a roll and a pull change them, and what a chase view hears
//! of it. Frequencies are in hertz, times in seconds, angles in radians, pressures in pascals.

/// Centre of the wind over the canopy at rest, what each metre per second adds, and its top.
pub const WIND_HZ: (f32, f32, f32) = (220.0, 5.0, 2600.0);

/// Resonance of the wind band and its level at the full dynamic pressure: 0.085, 3 dB under the
/// 0.12 of 0.11.0, which the owner heard as a little too loud.
pub const WIND: (f32, f32) = (0.6, 0.085);

/// Seconds the levels of the air on the airframe take to cover 63 percent of a change: they follow
/// the stick within 0.2 seconds and never click.
pub const FLOW_EASE: f32 = 0.1;

/// The flow torn off the canopy and the wings: the angle of attack or the sideslip in radians at
/// which it starts, 8 degrees, and at which it is torn through, 25 degrees; the share of the
/// resonance of the wind band it takes, which widens it; how rough the air on the airframe gets;
/// and how much it adds to the level of the wind.
pub const TORN: (f32, f32, f32, f32, f32) = (0.14, 0.44, 0.6, 0.5, 0.8);

/// Rates of roll and yaw in radians per second between which they start and finish adding their
/// whoosh, and how much the whoosh adds to the level of the wind at its fullest.
pub const WHIRL: (f32, f32, f32) = (0.5, 2.5, 0.5);

/// Loads of a turn in g between which a pull starts and finishes shaking the airframe.
pub const PULL: (f32, f32) = (2.0, 5.0);

/// Dynamic pressure in pascals at which the wind over the canopy is at its level.
pub const FULL_PRESSURE: f32 = 7.5e4;

/// Corner of the thuds of the buffet and their level in the roughest air.
pub const BUFFET: (f32, f32) = (45.0, 0.22);

/// Centre and resonance of the rattle of the airframe, the share of the samples that carry a
/// knock, and the peak of a knock in the roughest air.
pub const RATTLE: (f32, f32, f32, f32) = (1700.0, 6.0, 0.002, 0.05);

/// Corner in hertz of the deep roar of an entry through the hull, and its level at full heat.
pub const ROAR: (f32, f32) = (70.0, 0.3);

/// Rate of the gusts on the canopy and their depth.
pub const GUST: (f32, f32) = (0.8, 0.35);

/// Share of the wind on the airframe that a chase view riding along with the ship hears rushing
/// past its own ears.
pub const CHASE_WIND: f32 = 0.6;

/// Corner in hertz of the low-pass the hull makes of the wind over it, for ears in the seat: the
/// same changes of the flow come through, muffled.
pub const HUSH: f32 = 900.0;

/// How far the wind a chase view hears leans to the side the air comes from, at a sideslip of 90
/// degrees: the side of the ship facing the wind is the loud one.
pub const SLIP_LEAN: f32 = 0.6;
