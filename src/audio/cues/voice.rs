//! One blip: a short pitched or noisy sound with a click-free envelope.

use crate::audio::dsp::filter::Svf;
use crate::audio::dsp::osc::{Phasor, sine};
use crate::audio::dsp::rand::Noise;
use crate::audio::spatial::Ear;

/// What a blip is made of.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Wave {
    /// A pure sine.
    Sine,
    /// A sine with a little second harmonic: rounder, easier to hear on small speakers.
    Soft,
    /// Noise through a band-pass at the `from` frequency.
    Noise,
}

/// The recipe of one blip.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Blip {
    /// Waveform.
    pub wave: Wave,
    /// Frequency in hertz at the start.
    pub from: f32,
    /// Frequency in hertz at the end, reached by an even glide. Noise ignores it.
    pub to: f32,
    /// Length in seconds.
    pub seconds: f32,
    /// Fade-in time in seconds.
    pub attack: f32,
    /// Seconds to wait before starting.
    pub delay: f32,
    /// Peak level.
    pub gain: f32,
    /// True for a sound that grows until its end instead of dying away.
    pub swell: bool,
}

impl Blip {
    /// A soft 440 Hz tone of a tenth of a second, the base other blips override.
    pub const BASE: Blip = Blip {
        wave: Wave::Soft,
        from: 440.0,
        to: 440.0,
        seconds: 0.1,
        attack: 0.004,
        delay: 0.0,
        gain: 0.15,
        swell: false,
    };
}

/// Resonance of the noise band-pass.
const NOISE_Q: f32 = 1.4;

/// Share of the length a swelling blip takes to fall silent at its end.
const SWELL_RELEASE: f32 = 0.08;

/// One playing blip.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Voice {
    /// What is playing.
    blip: Blip,
    /// Oscillator phase.
    phase: Phasor,
    /// Turns per sample now.
    step: f32,
    /// Factor applied to the step every sample.
    bend: f32,
    /// Samples since the start, negative while waiting.
    at: f32,
    /// Length in samples.
    len: f32,
    /// Fade-in length in samples.
    attack: f32,
    /// Noise colour.
    filter: Svf,
    /// Noise source.
    noise: Noise,
    /// Whether the voice is sounding or waiting to.
    live: bool,
    /// The two ears: where the sound comes from.
    pub ear: Ear,
    /// Whether it is a sound of the interface, which plays on while the game is paused.
    pub ui: bool,
}

impl Voice {
    /// Builds an idle voice with its own noise seed.
    pub fn new(seed: u32) -> Self {
        Self {
            blip: Blip::BASE,
            phase: Phasor(0.0),
            step: 0.0,
            bend: 1.0,
            at: 0.0,
            len: 1.0,
            attack: 1.0,
            filter: Svf::default(),
            noise: Noise::new(seed),
            live: false,
            ear: Ear::new(1.0),
            ui: false,
        }
    }

    /// Returns true when the voice is free.
    pub fn idle(&self) -> bool {
        !self.live
    }

    /// Starts `blip`, transposed by the factor `pitch` and scaled by `gain`.
    pub fn start(&mut self, blip: Blip, pitch: f32, gain: f32, rate: f32) {
        let len = (blip.seconds * rate).max(2.0);
        let from = (blip.from * pitch).clamp(20.0, 0.4 * rate);
        let to = (blip.to * pitch).clamp(20.0, 0.4 * rate);
        self.blip = Blip {
            gain: blip.gain * gain,
            ..blip
        };
        self.phase = Phasor(0.0);
        self.step = from / rate;
        self.bend = (to / from).powf(1.0 / len);
        self.at = -blip.delay * rate;
        self.len = len;
        self.attack = (blip.attack * rate).max(1.0);
        self.filter = Svf::default();
        self.filter.tune(from, NOISE_Q, rate);
        if blip.wave == Wave::Noise {
            self.blip.gain *= 2.5 * (1000.0 / from).sqrt().min(4.0);
        }
        self.live = true;
    }

    /// Returns the envelope at the current position: a smooth rise, then a fall that ends on zero.
    fn envelope(&self) -> f32 {
        let t = self.at / self.len;
        let rise = (self.at / self.attack).min(1.0);
        let body = if self.blip.swell {
            t * t * ((1.0 - t) / SWELL_RELEASE).min(1.0)
        } else {
            (1.0 - t) * (1.0 - t) * (1.0 - t)
        };
        rise * rise * (3.0 - 2.0 * rise) * body
    }

    /// Renders one sample.
    pub fn frame(&mut self) -> f32 {
        if !self.live {
            return 0.0;
        }
        self.at += 1.0;
        if self.at < 0.0 || self.at >= self.len {
            self.live = self.at < 0.0;
            return 0.0;
        }
        let turn = self.phase.tick(self.step);
        self.step *= self.bend;
        let raw = match self.blip.wave {
            Wave::Sine => sine(turn),
            Wave::Soft => 0.82 * sine(turn) + 0.18 * sine((2.0 * turn).fract()),
            Wave::Noise => self.filter.band(self.noise.tick()),
        };
        raw * self.envelope() * self.blip.gain
    }
}
