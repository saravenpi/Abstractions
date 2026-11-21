//! The lock-free bridge between the game and the audio thread: atomics only, no lock, no allocation.

use std::sync::atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering};

use super::cue::AudioCue;
use super::planet::code;
use super::scene::AudioScene;
use super::ship::feed::Deck;
use super::spatial::{SPOT, Spot};
use crate::settings::{Audio as Volumes, Range};

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
    /// Speed of the air past a flying body, which sets the pitch of its rush.
    Speed,
    /// Buffeting of the air on a flying body.
    Buffet,
    /// Side of the shoreline: 0 left, 0.5 ahead, 1 right.
    ShoreSide,
    /// Corner in hertz of the low-pass a kilometre of the air around the listener makes.
    AirFar,
    /// Exponent of the distance in the law of that corner.
    AirSlope,
    /// Class of the planet, as [`code`] writes it.
    Class,
    /// How hard it rains around the listener.
    Rain,
    /// How much dust blows past the listener.
    Dust,
    /// Whether the dynamic range is reduced.
    Reduced,
    /// Whether both ears get the same sound.
    Mono,
}

/// Number of [`Param`] values.
pub const PARAMS: usize = 25;

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
        Param::Cloud,
        Param::Cabin,
        Param::Space,
        Param::Home,
        Param::Rush,
        Param::Boost,
        Param::Flutter,
        Param::Speed,
        Param::Buffet,
        Param::ShoreSide,
        Param::AirFar,
        Param::AirSlope,
        Param::Class,
        Param::Rain,
        Param::Dust,
        Param::Reduced,
        Param::Mono,
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
    cues: [AtomicU64; QUEUE],
    /// Bits of the fields of the spot of each cue of the ring.
    spots: [[AtomicU32; SPOT]; QUEUE],
    /// Slots written so far.
    head: AtomicUsize,
    /// Slots read so far.
    tail: AtomicUsize,
    /// What every ship in earshot sounds like.
    pub hulls: Deck,
}

impl Default for Shared {
    /// Starts on the title scene at the default volumes, with no cue waiting.
    fn default() -> Self {
        let shared = Self {
            params: std::array::from_fn(|_| AtomicU32::new(0)),
            seed: AtomicU64::new(0),
            cues: std::array::from_fn(|_| AtomicU64::new(0)),
            spots: std::array::from_fn(|_| std::array::from_fn(|_| AtomicU32::new(0))),
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            hulls: Deck::default(),
        };
        shared.publish(&AudioScene::default(), &Volumes::default());
        shared
    }
}

/// Returns `value` held between 0 and 1, and 0 for a value that is not a number.
pub(super) fn unit(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
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
        self.seed.store(scene.planet_seed, Ordering::Relaxed);
        self.set(Param::Class, code(scene.class));
        self.set(Param::Master, unit(volumes.master));
        self.set(Param::Music, unit(volumes.music));
        self.set(Param::Effects, unit(volumes.effects));
        self.set(Param::Reduced, flag(volumes.range == Range::Reduced));
        self.set(Param::Mono, flag(volumes.mono));
        self.set(Param::MusicLevel, unit(scene.music_level()));
        self.set(Param::Day, unit(scene.daylight));
        self.set(Param::Submerged, unit(scene.submerged));
        self.hulls.publish(&scene.hulls);
        self.set(Param::Cloud, unit(scene.cloud_depth));
        self.set(Param::Cabin, unit(scene.cabin));
        self.set(Param::Space, flag(scene.in_space || scene.warping));
        self.set(Param::Home, flag(scene.home));
        let outside = 1.0 - flag(scene.in_space || scene.warping);
        self.publish_flight(scene, outside);
        self.publish_air(scene, outside);
    }

    /// Queues a cue for the audio thread, heard as `spot` says. Returns false when the queue is
    /// full and the cue is lost.
    ///
    /// Only one thread may push.
    pub fn push(&self, cue: AudioCue, spot: &Spot) -> bool {
        let head = self.head.load(Ordering::Relaxed);
        if head.wrapping_sub(self.tail.load(Ordering::Acquire)) >= QUEUE {
            return false;
        }
        let cells = &self.spots[head % QUEUE];
        for (cell, value) in cells.iter().zip(spot.to_array()) {
            cell.store(value.to_bits(), Ordering::Relaxed);
        }
        self.cues[head % QUEUE].store(cue.encode(), Ordering::Relaxed);
        self.head.store(head.wrapping_add(1), Ordering::Release);
        true
    }

    /// Takes the oldest waiting cue and its spot. Only one thread may pop.
    pub fn pop(&self) -> Option<(AudioCue, Spot)> {
        let tail = self.tail.load(Ordering::Relaxed);
        if tail == self.head.load(Ordering::Acquire) {
            return None;
        }
        let word = self.cues[tail % QUEUE].load(Ordering::Relaxed);
        let cells = &self.spots[tail % QUEUE];
        let spot = Spot::from_array(std::array::from_fn(|i| {
            f32::from_bits(cells[i].load(Ordering::Relaxed))
        }));
        self.tail.store(tail.wrapping_add(1), Ordering::Release);
        AudioCue::decode(word).map(|cue| (cue, spot))
    }
}
