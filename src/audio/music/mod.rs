//! Procedural ambient music: one slow piece per planet, composed from the planet seed.

pub(super) mod bells;
pub(super) mod harmony;
pub(super) mod mode;
pub(super) mod pad;
pub(super) mod piece;
pub(super) mod roots;
pub(super) mod voicing;

use bells::Bells;
use harmony::Harmony;
use pad::Pad;
use piece::Piece;

use super::dsp::filter::{Lag, Svf, glide, pole};
use super::dsp::reverb::Reverb;

/// Seconds for the music level to cover 63 percent of a change: a fade lasts about four times this.
pub const FADE_SECONDS: f32 = 0.9;

/// Level under which a fading piece is switched off entirely.
const SILENT: f32 = 1e-3;

/// Low-pass cutoff in hertz when the listener is fully under water.
const DEEP_HZ: f32 = 320.0;

/// Fraction of the midday cutoff left at midnight.
const NIGHT_BRIGHT: f32 = 0.55;

/// Fraction of the midday level left at midnight.
const NIGHT_LEVEL: f32 = 0.7;

/// How many times longer the bells wait at midnight.
const NIGHT_BELLS: f32 = 2.4;

/// Seconds for the day, night and water changes to settle.
const MOOD_SECONDS: f32 = 2.5;

/// What the world asks of the music.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MusicCtl {
    /// Seed of the planet whose piece should play.
    pub seed: u64,
    /// Whether that planet is the home planet.
    pub home: bool,
    /// Level from 0 (silent) to 1 (full).
    pub level: f32,
    /// Daylight from 0 (night) to 1 (day).
    pub day: f32,
    /// How far the listener is under water, from 0 to 1.
    pub submerged: f32,
}

/// The music engine. Rendering a frame never allocates.
#[derive(Debug, Clone)]
pub struct Music {
    /// Sample rate in hertz.
    rate: f32,
    /// The piece being played.
    piece: Piece,
    /// Its chord progression.
    harmony: Harmony,
    /// Its pad.
    pad: Pad,
    /// Its bells.
    bells: Bells,
    /// Low-pass on each channel.
    tone: [Svf; 2],
    /// Reverb.
    reverb: Reverb,
    /// Current level, following `want`.
    level: Lag,
    /// Level asked for, 0 while another piece is waiting.
    want: f32,
    /// Smoothed daylight.
    day: Lag,
    /// Smoothed depth under water.
    deep: Lag,
    /// Samples until the next chord.
    until: f32,
    /// Per-sample coefficient of the level fade.
    fade: f32,
}

impl Music {
    /// Builds the engine, silent, holding the piece of planet `seed`.
    pub fn new(rate: f32, seed: u64, home: bool) -> Self {
        let piece = Piece::new(seed, home);
        let mut music = Self {
            rate,
            piece,
            harmony: Harmony::new(piece.scale, piece.lift, seed),
            pad: Pad::new(rate, seed),
            bells: Bells::new(rate, seed, piece.pace.bell_gap),
            tone: [Svf::default(); 2],
            reverb: Reverb::new(rate),
            level: Lag(0.0),
            want: 0.0,
            day: Lag(1.0),
            deep: Lag(0.0),
            until: 0.0,
            fade: glide(FADE_SECONDS, rate),
        };
        music.load(seed, home);
        music
    }

    /// Replaces the piece in place. Only called while the music is silent.
    fn load(&mut self, seed: u64, home: bool) {
        self.piece = Piece::new(seed, home);
        self.harmony = Harmony::new(self.piece.scale, self.piece.lift, seed);
        self.pad = Pad::new(self.rate, seed);
        self.pad.play(&self.harmony.chord());
        self.bells = Bells::new(self.rate, seed, self.piece.pace.bell_gap);
        let room = self.piece.room;
        self.reverb
            .shape(room.size, room.tail, pole(room.dark, self.rate));
        self.tone = [Svf::default(); 2];
        self.until = self.piece.pace.chord * self.rate;
    }

    /// Takes new orders. Call it every `dt` seconds, a few hundred times per second.
    ///
    /// A different planet fades the current piece out, swaps it while silent and fades back in.
    pub fn steer(&mut self, ctl: &MusicCtl, dt: f32) {
        let other = ctl.seed != self.piece.seed || ctl.home != self.piece.home;
        if other && self.level.0 == 0.0 {
            self.load(ctl.seed, ctl.home);
        }
        self.want = if other {
            0.0
        } else {
            ctl.level.clamp(0.0, 1.0)
        };
        let k = glide(MOOD_SECONDS, 1.0 / dt);
        let day = self.day.to(ctl.day.clamp(0.0, 1.0), k);
        let deep = self.deep.to(ctl.submerged.clamp(0.0, 1.0), k);
        let open = self.piece.timbre.bright * (NIGHT_BRIGHT + (1.0 - NIGHT_BRIGHT) * day);
        let hz = open * (DEEP_HZ / open).powf(deep);
        self.tone[0].tune(hz, 0.6, self.rate);
        self.tone[1].tune(hz, 0.6, self.rate);
    }

    /// Renders one stereo frame.
    pub fn frame(&mut self) -> [f32; 2] {
        let level = self.level.to(self.want, self.fade);
        if self.want == 0.0 && level < SILENT {
            self.level.0 = 0.0;
            return [0.0; 2];
        }
        self.until -= 1.0;
        if self.until <= 0.0 {
            self.pad.play(&self.harmony.advance());
            self.until += self.piece.pace.chord * self.rate;
        }
        let (day, pace) = (self.day.0, self.piece.pace);
        let pad = self.pad.frame(&self.piece.timbre, pace.fade, day);
        let gap = pace.bell_gap * (NIGHT_BELLS - (NIGHT_BELLS - 1.0) * day);
        let bell = self
            .bells
            .frame(&self.harmony.chord(), gap, pace.bell_level);
        let dry = [
            self.tone[0].low(pad[0] + bell[0]),
            self.tone[1].low(pad[1] + bell[1]),
        ];
        let wet = self.reverb.run(dry[0], dry[1]);
        let gain = level * (NIGHT_LEVEL + (1.0 - NIGHT_LEVEL) * day);
        let mix = self.piece.room.wet;
        [0, 1].map(|i| (dry[i] * (1.0 - 0.4 * mix) + wet[i] * mix) * gain)
    }

    /// Returns the piece being played.
    #[cfg(test)]
    pub fn piece(&self) -> &Piece {
        &self.piece
    }
}
