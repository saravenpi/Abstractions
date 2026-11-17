//! The lock-free bridge between the game and the audio thread: atomics only, no lock, no allocation.

use std::sync::atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering};

use super::cue::AudioCue;
use super::scene::AudioScene;
use crate::settings::Audio as Volumes;

/// A live value read by the audio thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Param {
    /// Master volume.
    Master,
    /// Music volume.
    Music,
    /// Effects volume.
    Effects,
    /// Music level asked by the scene.
    MusicLevel,
    /// Daylight.
    Day,
    /// Depth under water.
    Submerged,
    /// Wind strength, already thinned by altitude.
    Wind,
    /// Shore closeness.
    Shore,
    /// Engine throttle.
    Throttle,
    /// Dynamic pressure.
    Pressure,
    /// Depth inside the clouds.
    Cloud,
    /// Whether the listener is in the cabin.
    Cabin,
    /// Whether the listener is in space.
    Space,
    /// Whether the planet is the home planet.
    Home,
    /// Rush of air past a flying body.
    Rush,
    /// Push of the boot boosters.
    Boost,
    /// Flutter of the wingsuit cloth.
    Flutter,
}

/// Number of [`Param`] values.
pub const PARAMS: usize = 17;

impl Param {
    /// Every parameter, in index order.
    pub const ALL: [Param; PARAMS] = [
        Param::Master,
        Param::Music,
        Param::Effects,
        Param::MusicLevel,
        Param::Day,
        Param::Submerged,
        Param::Wind,
        Param::Shore,
        Param::Throttle,
        Param::Pressure,
        Param::Cloud,
        Param::Cabin,
        Param::Space,
        Param::Home,
        Param::Rush,
        Param::Boost,
        Param::Flutter,
    ];
}

/// Cues that can wait for the audio thread. More than that are dropped.
const QUEUE: usize = 64;

/// Everything the game tells the audio thread.
#[derive(Debug)]
pub struct Shared {
    /// Bits of the `f32` value of each [`Param`].
    params: [AtomicU32; PARAMS],
    /// Seed of the planet whose piece plays.
    seed: AtomicU64,
    /// Ring of encoded cues, 0 meaning empty.
    cues: [AtomicU32; QUEUE],
    /// Slots written so far.
    head: AtomicUsize,
    /// Slots read so far.
    tail: AtomicUsize,
}

impl Default for Shared {
    /// Starts on the title scene at the default volumes, with no cue waiting.
    fn default() -> Self {
        let shared = Self {
            params: std::array::from_fn(|_| AtomicU32::new(0)),
            seed: AtomicU64::new(0),
            cues: std::array::from_fn(|_| AtomicU32::new(0)),
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
        };
        shared.publish(&AudioScene::default(), &Volumes::default());
        shared
    }
}

impl Shared {
    /// Stores one value.
    pub fn set(&self, param: Param, value: f32) {
        self.params[param as usize].store(value.to_bits(), Ordering::Relaxed);
    }

    /// Returns one value.
    pub fn get(&self, param: Param) -> f32 {
        f32::from_bits(self.params[param as usize].load(Ordering::Relaxed))
    }

    /// Returns the seed of the planet whose piece should play.
    pub fn seed(&self) -> u64 {
        self.seed.load(Ordering::Relaxed)
    }

    /// Stores the whole scene and the volume settings.
    pub fn publish(&self, scene: &AudioScene, volumes: &Volumes) {
        let flag = |on: bool| f32::from(u8::from(on));
        let unit = |value: f32| {
            if value.is_finite() {
                value.clamp(0.0, 1.0)
            } else {
                0.0
            }
        };
        self.seed.store(scene.planet_seed, Ordering::Relaxed);
        self.set(Param::Master, unit(volumes.master));
        self.set(Param::Music, unit(volumes.music));
        self.set(Param::Effects, unit(volumes.effects));
        self.set(Param::MusicLevel, unit(scene.music_level()));
        self.set(Param::Day, unit(scene.daylight));
        self.set(Param::Submerged, unit(scene.submerged));
        self.set(Param::Wind, unit(scene.wind * scene.altitude_fade));
        self.set(Param::Shore, unit(scene.shore));
        self.set(Param::Throttle, unit(scene.throttle));
        self.set(Param::Pressure, unit(scene.dynamic_pressure));
        self.set(Param::Cloud, unit(scene.cloud_depth));
        self.set(Param::Cabin, unit(scene.cabin));
        self.set(Param::Space, flag(scene.in_space || scene.warping));
        self.set(Param::Home, flag(scene.home));
        let outside = 1.0 - flag(scene.in_space || scene.warping);
        self.set(Param::Rush, unit(scene.rush * outside));
        self.set(Param::Boost, unit(scene.boost));
        self.set(Param::Flutter, unit(scene.flutter * outside));
    }

    /// Queues a cue for the audio thread. Returns false when the queue is full and the cue is lost.
    ///
    /// Only one thread may push.
    pub fn push(&self, cue: AudioCue) -> bool {
        let head = self.head.load(Ordering::Relaxed);
        if head.wrapping_sub(self.tail.load(Ordering::Acquire)) >= QUEUE {
            return false;
        }
        self.cues[head % QUEUE].store(cue.encode(), Ordering::Relaxed);
        self.head.store(head.wrapping_add(1), Ordering::Release);
        true
    }

    /// Takes the oldest waiting cue. Only one thread may pop.
    pub fn pop(&self) -> Option<AudioCue> {
        let tail = self.tail.load(Ordering::Relaxed);
        if tail == self.head.load(Ordering::Acquire) {
            return None;
        }
        let word = self.cues[tail % QUEUE].load(Ordering::Relaxed);
        self.tail.store(tail.wrapping_add(1), Ordering::Release);
        AudioCue::decode(word)
    }
}
