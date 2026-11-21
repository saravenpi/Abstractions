//! Tuning constants of the footsteps. Frequencies are in hertz, times in seconds, lengths in
//! metres, masses in kilograms. A level is a peak against a full scale of 1.

/// Force of a walk at home, the speed of the sole into the ground over the speed of a hard
/// landing: the levels of every ground are given for it.
pub const FORCE_REF: f32 = 0.13;

/// Exponent of the pressure of a strike against the speed of the sole: the ground gives way
/// and takes part of a harder strike in, so the sound grows slower than the speed.
pub const GROWTH: f32 = 0.7;

/// Weight a foot carries in a walk at home, in weights of the robot: the grains of every ground
/// are given for it.
pub const LOAD_REF: f32 = 2.9;

/// Mass of a leg of the robot, 16 percent of its 80 kilograms: the thud of every ground is
/// tuned for it. A heavier strike stays longer on the ground and sounds lower, as the square
/// root of its mass.
pub const MASS_REF: f32 = 12.9;

/// How far the pitch of a step strays from its ground, either way, and the quietest a step is
/// against the loudest of its force.
pub const STRAY: (f32, f32) = (0.06, 0.8);

/// How far the roll from heel to toe strays from what the gait asks, either way, as a share.
pub const ROLL_STRAY: f32 = 0.15;

/// How far apart the two feet of a body are tuned, how far to each side of the listener its
/// own feet are heard, and how far apart their levels are: no two legs are built alike.
pub const FEET: (f32, f32, f32) = (0.03, 0.06, 0.08);

/// Share of a step the heel strikes with, and the share the toe lands with after the roll.
pub const HEEL_TOE: (f32, f32) = (0.6, 0.4);

/// The armour of the robot on a hard landing: the force it starts to tick at, its tone, its
/// resonance, and its level against the strike of the landing at the hardest one. A harder landing
/// rings the plates harder: at the hardest the tick peaks about 9 dB under the thud of the landing,
/// faint but heard, for a robot is not a bell.
pub const TICK: (f32, f32, f32, f32) = (0.45, 3900.0, 60.0, 0.05);

/// The scrape of a slide, sent every 0.1 seconds while the body slides: the seconds one stretch
/// swells and dies over, twice the gap so that the stretches overlap into one even rustle; the
/// share of a step's grains it breaks; the level of the rustle at full speed, about the level of a
/// walk, a few decibels over it at most; and the force of the slide, from its speed, that counts
/// as full speed: 14 metres per second.
pub const SCRAPE: (f32, f32, f32, f32) = (0.2, 0.3, 0.03, 0.4);

/// Most a strike may push against a walk at home: 8 dB. A hard landing is a short, firm thud, not a
/// crash.
pub const PUSH_MAX: f32 = 2.5;

/// Depth of water, in metres, at which a foot wades to the knee: the splash, the thud and the drops
/// are gone and the slosh of the leg is a long, low swish.
pub const KNEE: f32 = 0.5;

/// Resonance of the band of the suck of mud.
pub const WET_Q: f32 = 2.5;

/// Resonance of the band of a splash, a slosh or a scrape: broad, so it hisses and never whistles.
pub const HISS_Q: f32 = 1.0;

/// Depth of water at which a splash has the level its ground gives it, and the least and the
/// most of that level a shallower or deeper splash has.
pub const WADE: (f32, f32, f32) = (0.25, 0.4, 1.6);

/// Shortest tail of a step after its last contact, so that its ring and grains die out.
pub const TAIL: f32 = 0.3;
