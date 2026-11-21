//! Tuning of the voices of animals: levels, how many sound at once, the shapes of the glottal pulse,
//! the absorption of the air and the muffling of the water.

/// Voices of animals that can sound at once.
pub const THROATS: usize = 6;

/// Contact and young calls that may sound at once: past it a chorus becomes a wall, so a new one waits
/// for its turn, as neighbouring animals avoid calling over each other.
pub const CHORUS: usize = 3;

/// Share of its length a call of a species must have sounded before another of the same species may
/// start a contact call: they answer each other instead of calling in unison.
pub const TURN: f32 = 0.6;

/// Level of a call heard beside the head, before its cry and its distance.
pub const LEVEL: f32 = 0.12;

/// Root mean square a voice is brought to before its envelope, against a full scale of 1.
pub const BODY_RMS: f32 = 0.4;

/// Seconds the level follower of a voice takes to hear a change of its root mean square.
pub const FOLLOW: f32 = 0.03;

/// Share of the glottal period the folds take to open, and to close: the Rosenberg pulse.
pub const OPEN: (f32, f32) = (0.45, 0.18);

/// Value of the logistic map in its chaotic range that drives the irregular periods of a rough voice.
pub const CHAOS_R: f32 = 3.91;

/// Roughness, 0 to 1, between which the pulse train doubles its period: subharmonics.
pub const SUBHARMONIC: (f32, f32) = (0.25, 0.65);

/// Roughness between which the period-doubled train breaks into deterministic chaos.
pub const CHAOTIC: (f32, f32) = (0.55, 1.0);

/// Level of the second, unrelated tone of a biphonating voice against the first.
pub const BIPHONIC: f32 = 0.3;

/// Samples between two retunings of the filters that follow the pitch.
pub const RETUNE: u32 = 32;

/// Absorption of the air in decibels per metre at 1 kHz, and the exponent of the frequency in it:
/// about 0.005 dB/m at 1 kHz and 0.1 dB/m at 8 kHz at 20 degrees and half humidity (ISO 9613-1).
pub const AIR: (f32, f32) = (0.005, 1.5);

/// Decibels the air may take at the corner of the low-pass that stands for it.
pub const AIR_CORNER_DB: f32 = 3.0;

/// Highest corner in hertz of the low-pass of the air, beside the head.
pub const AIR_TOP: f32 = 12_000.0;

/// Corner in hertz of the low-pass of a call heard through the surface of the water, and the share
/// of the level it keeps.
pub const MUFFLE: (f32, f32) = (650.0, 0.5);

/// Share of the level the voice of the companion keeps while an animal calls: it ducks under.
pub const BUDDY_UNDER: f32 = 0.5;
