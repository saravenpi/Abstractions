//! The pad: slow chords made of detuned sines and a soft triangle, crossfading between two layers.

use super::piece::Timbre;
use super::voicing::{Chord, VOICES};
use crate::audio::dsp::osc::{Phasor, note_hz, sine, triangle};
use crate::audio::dsp::rand::Rand;

/// Overall level of the pad, which leaves headroom for five voices and the reverb.
pub const PAD_LEVEL: f32 = 0.2;

/// Level of each voice: bass, three upper voices, colour tone.
const GAINS: [f32; VOICES] = [1.0, 0.62, 0.55, 0.5, 0.45];

/// Level of the octave partial each voice always has. Only the bass has one, so that it is
/// still heard on small speakers.
const OCTAVE: [f32; VOICES] = [0.3, 0.0, 0.0, 0.0, 0.0];

/// Turns per second of the stereo drift of each voice.
const DRIFT_HZ: [f32; VOICES] = [0.011, 0.047, 0.031, 0.059, 0.023];

/// Turns per second of the slow swell of each voice.
const BREATH_HZ: [f32; VOICES] = [0.043, 0.071, 0.089, 0.061, 0.103];

/// How far the stereo drift moves a voice from the centre, 0.5 being hard left and right.
const DRIFT_WIDTH: f32 = 0.34;

/// Depth of the slow swell, as a fraction of the voice level.
const BREATH_DEPTH: f32 = 0.22;

/// The oscillators of one voice.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct Tone {
    /// Sharp sine, flat sine, triangle and octave sine.
    partials: [Phasor; 4],
    /// Stereo drift.
    drift: Phasor,
    /// Slow swell.
    breath: Phasor,
}

/// One chord held by the pad.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct Layer {
    /// Oscillators of each voice.
    tones: [Tone; VOICES],
    /// Turns per sample of each voice.
    steps: [f32; VOICES],
    /// Fade position from 0 (silent) to 1 (full).
    level: f32,
    /// Fade position the layer moves toward.
    target: f32,
}

impl Layer {
    /// Renders one stereo frame. `fade` is the fade speed per sample and `gains` the voice levels.
    fn frame(&mut self, timbre: &Timbre, fade: f32, gains: &[f32; VOICES], rate: f32) -> [f32; 2] {
        if self.level <= 0.0 && self.target <= 0.0 {
            return [0.0; 2];
        }
        self.level = (self.level + fade.copysign(self.target - 0.5)).clamp(0.0, 1.0);
        let curve = self.level * self.level * (3.0 - 2.0 * self.level);
        let sharp = (timbre.detune / 1200.0).exp2();
        let mut out = [0.0; 2];
        for v in 0..VOICES {
            let (tone, step) = (&mut self.tones[v], self.steps[v]);
            let [a, b, c, d] = &mut tone.partials;
            let pair = 0.5 * (sine(a.tick(step * sharp)) + sine(b.tick(step / sharp)));
            let soft = timbre.triangle * triangle(c.tick(step));
            let high = (timbre.shimmer + OCTAVE[v]) * sine(d.tick(step * 2.0));
            let swell =
                1.0 - BREATH_DEPTH * (0.5 + 0.5 * sine(tone.breath.tick(BREATH_HZ[v] / rate)));
            let pan = 0.5 + DRIFT_WIDTH * sine(tone.drift.tick(DRIFT_HZ[v] / rate));
            let voice = (pair + soft + high) * gains[v] * swell * curve;
            out = [out[0] + voice * (1.0 - pan), out[1] + voice * pan];
        }
        out
    }
}

/// The pad of a piece: two layers, one fading in while the other fades out.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pad {
    /// The two layers.
    layers: [Layer; 2],
    /// Index of the layer holding the current chord.
    front: usize,
    /// Sample rate in hertz.
    rate: f32,
}

impl Pad {
    /// Builds a silent pad whose oscillators start at phases drawn from `seed`.
    pub fn new(rate: f32, seed: u64) -> Self {
        let mut rand = Rand::new(seed, 4);
        let mut tone = || Tone {
            partials: [0; 4].map(|_| Phasor(rand.unit())),
            drift: Phasor(rand.unit()),
            breath: Phasor(rand.unit()),
        };
        let mut layer = || Layer {
            tones: [0; VOICES].map(|_| tone()),
            ..Layer::default()
        };
        Self {
            layers: [layer(), layer()],
            front: 0,
            rate,
        }
    }

    /// Starts `chord` on the idle layer and lets the current chord fade out.
    ///
    /// A voice that keeps its note takes over the phases of the fading layer, so that a tone
    /// common to both chords is held steady instead of beating against itself.
    pub fn play(&mut self, chord: &Chord) {
        let [a, b] = &mut self.layers;
        let (old, new) = if self.front == 0 { (a, b) } else { (b, a) };
        self.front = 1 - self.front;
        new.steps = chord.notes.map(|note| note_hz(note as f32) / self.rate);
        for v in 0..VOICES {
            if new.steps[v] == old.steps[v] {
                new.tones[v] = old.tones[v];
            }
        }
        (old.target, new.target) = (0.0, 1.0);
    }

    /// Renders one stereo frame. `fade_seconds` is the crossfade time and `day` thins the top
    /// voices at night.
    pub fn frame(&mut self, timbre: &Timbre, fade_seconds: f32, day: f32) -> [f32; 2] {
        let fade = 1.0 / (fade_seconds * self.rate);
        let mut gains = GAINS;
        gains[3] *= 0.3 + 0.7 * day;
        gains[4] *= timbre.colour * (0.2 + 0.8 * day);
        let [a, b] = &mut self.layers;
        let (x, y) = (
            a.frame(timbre, fade, &gains, self.rate),
            b.frame(timbre, fade, &gains, self.rate),
        );
        [(x[0] + y[0]) * PAD_LEVEL, (x[1] + y[1]) * PAD_LEVEL]
    }
}
