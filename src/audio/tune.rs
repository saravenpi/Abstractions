//! The targets and the constants of the whole mix. Loudness is measured as ITU-R BS.1770 says,
//! in LUFS, and peaks as true peaks in dBTP. Levels are linear against a full scale of 1.
//!
//! The game is played on a computer: on headphones, on desktop speakers and on laptop speakers.
//! Console guidance asks for -24 LUFS over a long stretch of play on a home system and -18 on
//! a portable one, with true peaks under -1 dBTP. The targets here sit between the two: calm
//! play is the quiet end and a ship at full throttle the loud end of one mix, so that the player
//! sets the volume once.

/// Highest true peak of the output, in dBTP.
pub const TRUE_PEAK: f32 = -1.0;

/// Integrated loudness of the title screen, in LUFS: music alone. It is the first thing a new
/// player hears, at whatever volume the computer was left at, so it is not the loudest scene.
pub const TITLE: f32 = -23.0;

/// Integrated loudness of calm play, in LUFS: standing or walking on a planet in fair weather.
pub const CALM: f32 = -26.0;

/// Integrated loudness of busy play, in LUFS: a storm, a fight, a ship idling or cruising.
pub const BUSY: f32 = -21.0;

/// Integrated loudness of the loudest sustained play, in LUFS: a lift-off, full throttle.
pub const LOUD: f32 = -17.0;

/// How far over the integrated loudness of a calm day the loudest moment of a ship passing
/// 100 metres away must rise, in LU: the pass is an event in a calm scene, and it must be one.
pub const PASS_MARGIN: f32 = 4.0;

/// How far the integrated loudness of a scene may sit from its target, in LU, either way.
pub const WINDOW: f32 = 3.0;

/// Highest momentary loudness any event may reach, in LUFS: a boom, a crash, a blow.
pub const LOUDEST_MOMENT: f32 = -10.0;

/// Level above which the master limiter turns the output down: -3 dBFS. Below it the limiter
/// does nothing; it is a guard against sums nobody planned, not a way to be loud.
pub const LIMIT_AT: f32 = 0.708;

/// Seconds the limiter takes to turn down, which is how far it looks ahead, and seconds it
/// takes to let go.
pub const LIMIT_TIMES: (f32, f32) = (0.0004, 0.25);

/// Level of the output the reduced dynamic range pulls toward: -24 dBFS root mean square on
/// each channel, which is what a mix at -22 LUFS reads, the middle of the targets. Scenes
/// louder than that come down and quieter ones come up, so the average stays where it is.
pub const PIVOT: f32 = 0.063;

/// Ratio of the reduced dynamic range: 2 decibels in give 1 out, around the pivot.
pub const RATIO: f32 = 2.0;

/// Most the reduced dynamic range raises a quiet moment and lowers a loud one, as gains: 8
/// decibels either way.
pub const REACH: (f32, f32) = (2.5, 0.4);

/// Seconds the reduced dynamic range takes to hear a rise and a fall of the level, and seconds
/// its gain takes to follow. Slow, so that it rides the level like a hand on a fader and is
/// not heard pumping, and the same both ways, so that it reads the mean level and not the peaks:
/// a fast rise read every scene 4 decibels hot and turned all of them down.
pub const RIDE: (f32, f32, f32) = (0.3, 0.3, 0.6);

/// Levels of the effects, root mean square, between which the music gives way to them, and
/// the share of its level the music keeps under the loudest: 6 decibels down. An engine at
/// full throttle masks the music anyway; this takes the masked music out of the sum.
pub const DUCK: (f32, f32, f32) = (0.05, 0.25, 0.5);

/// Seconds the music takes to give way and to come back.
pub const DUCK_TIMES: (f32, f32) = (0.4, 2.0);

/// Level of the music in play against the title screen: 3 decibels down. On the title screen the
/// music is all there is; in play it is a bed under the world.
pub const BED_MUSIC: f32 = 0.708;

/// Gain of the one-shot sounds, the interface, the steps, the blows and the voices of the
/// companion and the animals: 2 decibels up, so that they read over the beds.
pub const CUES_GAIN: f32 = 1.25;

/// Level of the voice of the companion against its phrases as written: 4.4 decibels down, so it
/// speaks under the world rather than over it.
pub const CHIRP_LEVEL: f32 = 0.6;
