//! One-shot sounds other modules ask for.

use bevy::prelude::*;

use super::call::Call;
use super::hit::HitSound;

/// What the robot the player controls is walking on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StepSurface {
    /// Beach, desert or a shallow sandy floor.
    Sand,
    /// Meadow or forest floor.
    Grass,
    /// Bare rock.
    Rock,
    /// Planks: floors, piers and decks.
    Wood,
    /// Snow.
    Snow,
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
    /// One footstep on a surface.
    Footstep(StepSurface),
    /// The ship touched the ground. The strength runs from 0 (feather) to 1 (hard).
    Land(f32),
    /// The warp drive starts charging: a tone that rises for about three seconds.
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
}

/// Every surface, in code order.
const SURFACES: [StepSurface; 5] = [
    StepSurface::Sand,
    StepSurface::Grass,
    StepSurface::Rock,
    StepSurface::Wood,
    StepSurface::Snow,
];

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
    /// the weather and the pump from the boots; both are auditioned on their own. Used to audition and to test.
    pub fn all() -> impl Iterator<Item = AudioCue> {
        let plain = [
            AudioCue::UiMove,
            AudioCue::UiConfirm,
            AudioCue::UiCancel,
            AudioCue::Land(1.0),
            AudioCue::WarpSpool,
            AudioCue::WarpEngage,
            AudioCue::WarpExit,
        ];
        let steps = SURFACES.into_iter().map(AudioCue::Footstep);
        let chirps = CHIRPS.into_iter().map(AudioCue::Chirp);
        plain.into_iter().chain(steps).chain(chirps)
    }

    /// Packs the cue into one non-zero word, so that it can cross to the audio thread without a lock.
    pub(super) fn encode(self) -> u32 {
        let (kind, arg) = match self {
            AudioCue::UiMove => (1, 0),
            AudioCue::UiConfirm => (2, 0),
            AudioCue::UiCancel => (3, 0),
            AudioCue::Footstep(surface) => (4, surface as u32),
            AudioCue::Land(strength) => (5, (strength.clamp(0.0, 1.0) * STRENGTH_STEPS) as u32),
            AudioCue::WarpSpool => (6, 0),
            AudioCue::WarpEngage => (7, 0),
            AudioCue::WarpExit => (8, 0),
            AudioCue::Chirp(kind) => (9, kind as u32),
            AudioCue::Thunder(level) => (10, (level.clamp(0.0, 1.0) * STRENGTH_STEPS) as u32),
            AudioCue::Call(call) => (11, call.pack()),
            AudioCue::Pump(strength) => (12, (strength.clamp(0.0, 1.0) * STRENGTH_STEPS) as u32),
            AudioCue::Hit(sound) => (13, sound.pack()),
        };
        kind | (arg << 8)
    }

    /// Unpacks a word written by [`AudioCue::encode`], or nothing when the word is not a cue.
    pub(super) fn decode(word: u32) -> Option<Self> {
        let arg = (word >> 8) as usize;
        match word & 0xFF {
            1 => Some(AudioCue::UiMove),
            2 => Some(AudioCue::UiConfirm),
            3 => Some(AudioCue::UiCancel),
            4 => SURFACES.get(arg).copied().map(AudioCue::Footstep),
            5 => Some(AudioCue::Land(arg as f32 / STRENGTH_STEPS)),
            6 => Some(AudioCue::WarpSpool),
            7 => Some(AudioCue::WarpEngage),
            8 => Some(AudioCue::WarpExit),
            9 => CHIRPS.get(arg).copied().map(AudioCue::Chirp),
            10 => Some(AudioCue::Thunder(arg as f32 / STRENGTH_STEPS)),
            11 => Call::unpack(arg as u32).map(AudioCue::Call),
            12 => Some(AudioCue::Pump(arg as f32 / STRENGTH_STEPS)),
            13 => HitSound::unpack(arg as u32).map(AudioCue::Hit),
            _ => None,
        }
    }
}
