//! One-shot sounds other modules ask for.

use bevy::prelude::*;

use super::call::Call;
use super::hit::HitSound;
use super::step::Step;

/// What a foot lands on: one class of ground per kind of sound it makes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StepSurface {
    /// Meadow: a soft thud and the swish of the blades.
    Grass,
    /// Forest floor, packed earth and paths: a soft thud and a few crumbs.
    Soil,
    /// Loose stones: a crunch of grains.
    Gravel,
    /// Bare rock and masonry: a hard click and a little grit.
    Rock,
    /// Beach and desert: a soft hiss.
    Sand,
    /// Snow: a crunch and a squeak.
    Snow,
    /// Mud and the floor of a pond: a wet slap, then a suck as the foot pulls out.
    Mud,
    /// Shallow water: a splash, and drops.
    Water,
    /// The metal deck and the landing pad of the base: a hollow ring that dies fast.
    Metal,
    /// The wing and the hull of a ship: a thin panel that drums.
    Hull,
    /// Planks: floors, piers and decks.
    Wood,
}

impl StepSurface {
    /// Every surface, in code order.
    pub const ALL: [StepSurface; 11] = [
        StepSurface::Grass,
        StepSurface::Soil,
        StepSurface::Gravel,
        StepSurface::Rock,
        StepSurface::Sand,
        StepSurface::Snow,
        StepSurface::Mud,
        StepSurface::Water,
        StepSurface::Metal,
        StepSurface::Hull,
        StepSurface::Wood,
    ];
}

/// What the robot companion says, as a short phrase of blips. The companion picks the kind
/// and the audio gives each one its sound: the two modules share this one vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChirpKind {
    /// Pleased: three quick rising blips.
    Happy,
    /// Questioning: two blips, the second gliding up.
    Curious,
    /// Startled: one fast blip that jumps up.
    Surprised,
    /// Drowsy: two slow, low, falling notes.
    Sleepy,
    /// Uneasy: three falling, hurried blips.
    Worried,
    /// A discovery: a bright rising trill.
    Delight,
    /// The start of a hummed tune: four soft notes.
    Hum,
    /// Waking up: a low blip, then a quick rise.
    Wake,
}

/// A one-shot sound. Write it with `MessageWriter<AudioCue>` from any system.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub enum AudioCue {
    /// The menu selection moved.
    UiMove,
    /// A menu entry was accepted.
    UiConfirm,
    /// A menu was closed or an action refused.
    UiCancel,
    /// One foot meets the ground, or the body lands or slides on it.
    Footstep(Step),
    /// The ship touched the ground. The strength runs from 0 (feather) to 1 (hard).
    Land(f32),
    /// The warp drive starts charging: its contactor closes. The rising tone of the spool
    /// itself is part of the voice of the ship.
    WarpSpool,
    /// The warp drive fires: a soft boom.
    WarpEngage,
    /// The ship drops out of warp.
    WarpExit,
    /// The robot companion speaks.
    Chirp(ChirpKind),
    /// Thunder rolls far away. The level runs from 0 (barely heard) to 1 (a near storm).
    Thunder(f32),
    /// An animal calls, from where it is for the listener.
    Call(Call),
    /// One pulse of the boot water pump. The strength runs from 0 (idle) to 1 (full spool).
    Pump(f32),
    /// A blow, a glance or a swing, placed for the listener.
    Hit(HitSound),
    /// A switch of the cockpit is thrown: a relay clicks.
    Switch,
    /// The shock wave of a ship faster than sound passes the listener. The strength runs from 0
    /// (far off) to 1 (overhead).
    Boom(f32),
}

/// Every chirp, in code order.
const CHIRPS: [ChirpKind; 8] = [
    ChirpKind::Happy,
    ChirpKind::Curious,
    ChirpKind::Surprised,
    ChirpKind::Sleepy,
    ChirpKind::Worried,
    ChirpKind::Delight,
    ChirpKind::Hum,
    ChirpKind::Wake,
];

/// Steps a landing strength or a thunder level is stored in.
const STRENGTH_STEPS: f32 = 1000.0;

impl AudioCue {
    /// Returns every cue the player sets off once, landing at full strength. Thunder comes from
    /// the weather, the pump from the boots, the boom from a ship and the steps from the feet;
    /// they are auditioned on their own. Used to audition and to test.
    pub fn all() -> impl Iterator<Item = AudioCue> {
        let plain = [
            AudioCue::UiMove,
            AudioCue::UiConfirm,
            AudioCue::UiCancel,
            AudioCue::Land(1.0),
            AudioCue::WarpSpool,
            AudioCue::WarpEngage,
            AudioCue::WarpExit,
            AudioCue::Switch,
        ];
        let chirps = CHIRPS.into_iter().map(AudioCue::Chirp);
        plain.into_iter().chain(chirps)
    }

    /// Packs the cue into one non-zero word, so that it can cross to the audio thread without a lock.
    pub(super) fn encode(self) -> u64 {
        let strength = |level: f32| (level.clamp(0.0, 1.0) * STRENGTH_STEPS) as u64;
        let (kind, arg): (u64, u64) = match self {
            AudioCue::UiMove => (1, 0),
            AudioCue::UiConfirm => (2, 0),
            AudioCue::UiCancel => (3, 0),
            AudioCue::Footstep(step) => (4, step.pack()),
            AudioCue::Land(level) => (5, strength(level)),
            AudioCue::WarpSpool => (6, 0),
            AudioCue::WarpEngage => (7, 0),
            AudioCue::WarpExit => (8, 0),
            AudioCue::Chirp(kind) => (9, kind as u64),
            AudioCue::Thunder(level) => (10, strength(level)),
            AudioCue::Call(call) => (11, u64::from(call.pack())),
            AudioCue::Pump(level) => (12, strength(level)),
            AudioCue::Hit(sound) => (13, u64::from(sound.pack())),
            AudioCue::Switch => (14, 0),
            AudioCue::Boom(level) => (15, strength(level)),
        };
        kind | (arg << 8)
    }

    /// Unpacks a word written by [`AudioCue::encode`], or nothing when the word is not a cue.
    pub(super) fn decode(word: u64) -> Option<Self> {
        let bits = word >> 8;
        let arg = bits as usize;
        match word & 0xFF {
            1 => Some(AudioCue::UiMove),
            2 => Some(AudioCue::UiConfirm),
            3 => Some(AudioCue::UiCancel),
            4 => Step::unpack(bits).map(AudioCue::Footstep),
            5 => Some(AudioCue::Land(arg as f32 / STRENGTH_STEPS)),
            6 => Some(AudioCue::WarpSpool),
            7 => Some(AudioCue::WarpEngage),
            8 => Some(AudioCue::WarpExit),
            9 => CHIRPS.get(arg).copied().map(AudioCue::Chirp),
            10 => Some(AudioCue::Thunder(arg as f32 / STRENGTH_STEPS)),
            11 => Call::unpack(bits as u32).map(AudioCue::Call),
            12 => Some(AudioCue::Pump(arg as f32 / STRENGTH_STEPS)),
            13 => HitSound::unpack(bits as u32).map(AudioCue::Hit),
            14 => Some(AudioCue::Switch),
            15 => Some(AudioCue::Boom(arg as f32 / STRENGTH_STEPS)),
            _ => None,
        }
    }
}
