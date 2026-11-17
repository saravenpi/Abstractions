//! A small hand-written reverb: two diffusing all-pass stages into four cross-fed delay lines.

use super::filter::Lag;

/// Longest delay line in seconds, at the largest room size.
const MAX_SECONDS: f32 = 0.36;

/// Delay line lengths in seconds for a room size of 1, mutually prime once in samples.
const LINES: [f32; 4] = [0.0731, 0.0893, 0.1069, 0.1277];

/// All-pass lengths in seconds, two per channel.
const DIFFUSE: [f32; 4] = [0.0047, 0.0127, 0.0053, 0.0113];

/// Feedback of the all-pass stages.
const DIFFUSION: f32 = 0.55;

/// One delay line with a fixed buffer and an adjustable length.
#[derive(Debug, Clone)]
struct Line {
    /// Sample memory, allocated once.
    buf: Vec<f32>,
    /// Samples in use, at most the buffer length.
    len: usize,
    /// Read and write position.
    pos: usize,
}

impl Line {
    /// Builds a silent line that can hold `seconds` and currently spans `span` seconds.
    fn new(seconds: f32, span: f32, rate: f32) -> Self {
        let buf = vec![0.0; (seconds * rate) as usize + 2];
        let len = ((span * rate) as usize).clamp(1, buf.len());
        Self { buf, len, pos: 0 }
    }

    /// Changes the span to `seconds` and clears the memory.
    fn resize(&mut self, seconds: f32, rate: f32) {
        self.len = ((seconds * rate) as usize).clamp(1, self.buf.len());
        self.pos = 0;
        self.buf.fill(0.0);
    }

    /// Returns the oldest sample.
    fn read(&self) -> f32 {
        self.buf[self.pos]
    }

    /// Replaces the oldest sample with `x` and moves on.
    fn write(&mut self, x: f32) {
        self.buf[self.pos] = x;
        self.pos = (self.pos + 1) % self.len;
    }

    /// Runs one sample through the line as an all-pass stage.
    fn allpass(&mut self, x: f32) -> f32 {
        let old = self.read();
        let fed = x + old * DIFFUSION;
        self.write(fed);
        old - fed * DIFFUSION
    }
}

/// A stereo reverb with a long, dark, even tail.
#[derive(Debug, Clone)]
pub struct Reverb {
    /// The four cross-fed delay lines.
    lines: [Line; 4],
    /// The input all-pass stages, two per channel.
    diffusers: [Line; 4],
    /// High damping inside each loop.
    damp: [Lag; 4],
    /// Loop gain, below 1.
    feedback: f32,
    /// Damping coefficient of the loop low-pass.
    dark: f32,
    /// Sample rate in hertz.
    rate: f32,
}

impl Reverb {
    /// Builds a silent reverb for a sample rate.
    pub fn new(rate: f32) -> Self {
        Self {
            lines: LINES.map(|s| Line::new(MAX_SECONDS, s, rate)),
            diffusers: DIFFUSE.map(|s| Line::new(s, s, rate)),
            damp: [Lag::default(); 4],
            feedback: 0.8,
            dark: 0.5,
            rate,
        }
    }

    /// Sets the room: `size` scales the delays (1 to 2.6), `tail` is the time in seconds to fall
    /// by 60 dB and `dark` the damping coefficient.
    ///
    /// The memory is cleared, so call it only while the input is silent.
    pub fn shape(&mut self, size: f32, tail: f32, dark: f32) {
        let mean = LINES.iter().sum::<f32>() / 4.0 * size;
        let feedback = 10.0_f32.powf(-3.0 * mean / tail.max(0.1));
        for (line, seconds) in self.lines.iter_mut().zip(LINES) {
            line.resize(seconds * size, self.rate);
        }
        self.damp = [Lag::default(); 4];
        self.feedback = feedback.clamp(0.0, 0.97);
        self.dark = dark.clamp(0.02, 1.0);
    }

    /// Runs one stereo frame and returns the wet signal only, level matched to the input.
    pub fn run(&mut self, left: f32, right: f32) -> [f32; 2] {
        let [a, b, c, d] = &mut self.diffusers;
        let l = b.allpass(a.allpass(left));
        let r = d.allpass(c.allpass(right));
        let out = [0, 1, 2, 3].map(|i| self.lines[i].read());
        let half = 0.5 * (out[0] + out[1] + out[2] + out[3]);
        for (i, tap) in out.iter().enumerate() {
            let back = self.damp[i].to(tap - half, self.dark) * self.feedback;
            self.lines[i].write(back + if i % 2 == 0 { l } else { r });
        }
        let norm = (1.0 - self.feedback * self.feedback).sqrt() * 0.7;
        [(out[0] + out[2]) * norm, (out[1] + out[3]) * norm]
    }
}
