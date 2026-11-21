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

/// The whoosh of a hard roll or yaw, in radians per second and seconds: the rate under which it
/// re-arms, the rate whose crossing sounds it, the time it dies away in, the refractory time before
/// it may sound again, and how much it adds to the level of the wind at its fullest. The small
/// corrections of the stick in steady flight stay under the onset.
pub const WHIRL: (f32, f32, f32, f32, f32) = (0.6, 1.5, 0.5, 1.5, 0.5);

/// Loads of a turn in g between which a pull starts and finishes shaking the airframe: from 3 g,
/// past what the stabiliser's own corrections give at full speed.
pub const PULL: (f32, f32) = (3.0, 6.0);

/// Dynamic pressure in pascals at which the wind over the canopy is at its level.
pub const FULL_PRESSURE: f32 = 7.5e4;

/// Corner of the thuds of the buffet and their level in the roughest air.
pub const BUFFET: (f32, f32) = (45.0, 0.22);

/// Centre and resonance of the rattle of the airframe, the share of the samples that carry a
/// knock, and the peak of a knock in the roughest air.
pub const RATTLE: (f32, f32, f32, f32) = (1700.0, 6.0, 0.002, 0.05);

/// Corner in hertz of the deep roar of an entry through the hull, and its level at full heat.
pub const ROAR: (f32, f32) = (70.0, 0.3);

/// Rate of the gusts on the canopy, and their depth in the roughest air. In calm air there is no
/// gust and steady flight is steady: the gusts follow how rough the air is.
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

/// Speeds in metres per second between which the flow starts and finishes tearing at a high angle:
/// in a hover the path points anywhere and nothing tears.
pub const SLOW: (f32, f32) = (20.0, 60.0);
