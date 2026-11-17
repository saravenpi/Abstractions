//! The audio asset Bevy plays: one endless stereo stream rendered by the mixer.

use std::sync::Arc;
use std::time::Duration;

use bevy::audio::{ChannelCount, Decodable, Sample, SampleRate, Source};
use bevy::prelude::*;

use super::mix::Mixer;
use super::shared::Shared;

/// Sample rate the synthesiser renders at. The output device converts it to its own rate.
pub const SYNTH_RATE: u32 = 48_000;

/// The stream is stereo.
const STEREO: ChannelCount = ChannelCount::new(2).unwrap();

/// The stream rate as the type the audio backend expects.
const RATE: SampleRate = SampleRate::new(SYNTH_RATE).unwrap();

/// The game's whole sound, as an asset. Playing it starts the synthesiser.
#[derive(Asset, TypePath, Debug, Clone)]
pub struct Synth {
    /// Values the game writes and the stream reads.
    shared: Arc<Shared>,
}

impl Synth {
    /// Builds the asset around the shared values.
    pub fn new(shared: Arc<Shared>) -> Self {
        Self { shared }
    }
}

/// The running synthesiser: an endless iterator of interleaved left and right samples.
#[derive(Debug)]
pub struct Stream {
    /// The synthesiser.
    mixer: Mixer,
    /// Right sample of the frame whose left sample was just returned.
    right: Option<f32>,
}

impl Iterator for Stream {
    type Item = Sample;

    /// Returns the next sample: left, then right, then the next frame.
    fn next(&mut self) -> Option<Sample> {
        if let Some(right) = self.right.take() {
            return Some(right);
        }
        let [left, right] = self.mixer.frame();
        self.right = Some(right);
        Some(left)
    }
}

impl Source for Stream {
    /// The stream never changes format.
    fn current_span_len(&self) -> Option<usize> {
        None
    }

    /// Two channels.
    fn channels(&self) -> ChannelCount {
        STEREO
    }

    /// The synthesiser rate.
    fn sample_rate(&self) -> SampleRate {
        RATE
    }

    /// The stream never ends.
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

impl Decodable for Synth {
    type Decoder = Stream;

    /// Starts a synthesiser, silent, that fades in.
    fn decoder(&self) -> Stream {
        Stream {
            mixer: Mixer::new(SYNTH_RATE as f32, self.shared.clone()),
            right: None,
        }
    }
}
