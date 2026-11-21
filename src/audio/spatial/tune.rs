//! Tuning of the spatial path. Lengths are in metres, times in seconds, frequencies in hertz. A
//! reach is the distance at which a source is heard at the level its recipe is written for: past
//! it, its pressure falls as `1 / r`.

/// Radius of the head of the robot: the far ear hears a sound fully to one side 0.66 ms later in
/// the air of home, by Woodworth's law `a / c (theta + sin theta)`.
pub const HEAD: f32 = 0.0875;

/// How far a sound fully to one side leans the pan, from 0 (both ears equal) to 1 (the far ear
/// silent): at 0.5 the near ear is 7.7 dB over the far one before the shadow of the head.
pub const LEAN: f32 = 0.5;

/// Corner of the shadow of the head on the far ear for a sound fully to one side. It opens as the
/// square of the sine of the bearing, so a sound ahead is not shadowed at all.
pub const SHADOW_HZ: f32 = 1800.0;

/// Corner of the shelf of the outer ear, and the share of the highs over it that a sound straight
/// behind loses: 4.4 dB, the cue that tells ahead from behind.
pub const REAR: (f32, f32) = (3000.0, 0.4);

/// Samples the line to the far ear holds: more than the widest gap in the slowest gas.
pub const GAP: usize = 64;

/// Height of the ears over the centre of the body of the robot. The soles are 0.9 metres under
/// it, so the ears are 1.7 metres over the ground.
pub const EARS: f32 = 0.8;

/// Level under which a sound is not played: 54 dB under its level at its reach, under the floor
/// of every scene. This is where a far sound ends, not at a fixed distance.
pub const FLOOR: f32 = 0.002;

/// Densest air, as a share of the home sea level, whose pressure still makes a sound louder.
pub const DENSE_CAP: f32 = 2.0;

/// Decibels the air takes at the corner of the low-pass that stands for it: a one-pole low-pass
/// is 3 dB down at its corner.
pub const CORNER_DB: f64 = 3.0;

/// Lowest and highest frequency the corner of the air is looked for between.
pub const BAND: (f64, f64) = (20.0, 40_000.0);

/// Halvings of the band in the search of a corner: a step of 0.05 percent.
pub const HALVINGS: usize = 24;

/// The two distances the corner of the air is worked out at: the corner follows a power law of
/// the distance through both, and past them.
pub const FIT: (f32, f32) = (30.0, 1000.0);

/// Lowest corner of the air, and a corner far above hearing that stands for no absorption.
pub const CORNERS: (f32, f32) = (40.0, 1.0e6);

/// Speed of sound in the water of a sea at 15 degrees.
pub const WATER_SOUND: f32 = 1480.0;

/// Depth under the surface at which a listener or a source counts as under water.
pub const DEEP: f32 = 0.3;

/// Depth of the sea past which sound in it spreads over a cylinder and not a sphere: farther
/// than that it falls as `1 / sqrt(r)`.
pub const SEA_DEPTH: f32 = 10.0;

/// Corner of what an ear under water hears, through the bones of the head and not the ear drum,
/// and the share of the bearing it keeps: under water a head tells direction poorly.
pub const WET_EAR: (f32, f32) = (1200.0, 0.3);

/// Share of the pressure that crosses the surface of the water, and the corner it leaves. A real
/// crossing keeps a thousandth of the energy, 30 dB; the game keeps 20 dB so that the world above
/// stays faintly heard.
pub const CROSS: (f32, f32) = (0.1, 650.0);

/// Share of the level a hill between a source and the listener leaves, and the corner of what
/// bends over it.
pub const HIDDEN: (f32, f32) = (0.5, 700.0);

/// Points read along the line from the listener to a source to see whether the ground hides it,
/// and the metres at either end left unread: the ground under the feet hides nothing.
pub const SIGHT: (usize, f32) = (8, 1.5);

/// Reach of a footstep, the height of the ears over the soles, and the share of the thud and the
/// ring of the listener's own step that the body carries, which is all of it a vacuum leaves.
pub const STEP: (f32, f32) = (1.7, 0.5);

/// Reach of a blow and of a swing, and the share of a blow the body carries when it lands on the
/// listener or on what it holds. At 1 metre a blow is at most 0.8 dB louder than the old law of
/// half the level at 10 metres, and quieter past 1.1 metres.
pub const HIT: (f32, f32) = (1.0, 0.25);

/// Metres from the centre of the listener's body within which a blow touches it or what it holds:
/// the length of a spear thrust.
pub const TOUCH: f32 = 2.2;

/// Reach of the voice of the companion: at most 0.8 dB louder than the old law of half the level
/// at 6 metres, and quieter past 0.8 metres.
pub const BUDDY: f32 = 0.7;

/// Share of the half-level distance of an animal call that is its reach: at most 0.8 dB louder
/// than the old law beside the animal, and quieter past a ninth of it.
pub const CALL: f32 = 0.1;

/// Distance and reach of far thunder, so that the level of a clap is its level 1000 metres off.
pub const THUNDER: f32 = 1000.0;

/// Height of a thunder clap over the ground, as a share of its distance.
pub const THUNDER_RISE: f32 = 0.3;

/// Reach of the boot pump, and the share of it the body carries: the pump is strapped to it.
pub const PUMP: (f32, f32) = (1.7, 0.6);

/// Radius in metres of the cabin around the seat, the share of the pressure of a sound outside it
/// that comes through the closed hull to ears in the seat, 10 dB down, and the corner the hull
/// leaves: glass and plating pass the low end and stop the rest.
pub const HULL: (f32, f32, f32) = (2.5, 0.3, 500.0);

/// Reach of the sounds of the cabin, its relays and its chimes, heard from outside through the
/// hull: 32 dB down for a chase view 12 metres back.
pub const CABIN_REACH: f32 = 0.3;

/// Level of what the ships make in the seat against the chase view: 0.63, 4 dB down. The cabin
/// gives about a decibel of it back as reflections.
pub const SEAT: f32 = 0.63;

/// Where the panel of the cockpit is for ears in the seat, in their own axes, and the reach of its
/// relays and chimes there: heard at their level, a little below and ahead.
pub const PANEL: ([f32; 3], f32) = ([0.0, -0.3, -0.5], 0.6);

/// Samples the lines of the cabin hold: 10.7 milliseconds at 48 kHz, a power of two.
pub const ROOM_LINE: usize = 512;

/// The early reflections of the cabin: the delay in milliseconds, the gain and the ear of each.
/// The canopy and the panels of a cabin a metre or so across send them back within 2 to 6 ms.
pub const TAPS: [(f32, f32, usize); 4] = [
    (2.1, 0.42, 0),
    (3.3, 0.36, 1),
    (4.6, 0.3, 0),
    (5.7, 0.26, 1),
];

/// The tail of the cabin: the delay of one pass in milliseconds, the seconds it takes to die by
/// 60 dB, and the corner in hertz of the damping of the walls on each pass.
pub const COMB: (f32, f32, f32) = (7.1, 0.15, 3500.0);

/// The modes of the cabin: the centre of their rise in hertz, its resonance and how much it adds:
/// 3.5 dB at 350 Hz, the box of a small room.
pub const BOX: (f32, f32, f32) = (350.0, 1.2, 0.5);

/// Share of the reflections of the cabin in what the ears hear.
pub const WET: f32 = 0.35;
