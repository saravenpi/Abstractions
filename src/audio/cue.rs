//! One-shot sounds other modules ask for.

use bevy::prelude::*;

/// What the player is walking on.
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

/// A mood of the robot companion, said as a short phrase of blips.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChirpKind {
    /// Three quick rising blips.
    Happy,
    /// Two blips, the second gliding up like a question.
    Curious,
    /// Three falling, hurried blips.
    Worried,
    /// Two slow, low, falling notes.
    Sleepy,
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
const CHIRPS: [ChirpKind; 4] = [
    ChirpKind::Happy,
    ChirpKind::Curious,
    ChirpKind::Worried,
    ChirpKind::Sleepy,
];

/// Steps a landing strength is stored in.
const STRENGTH_STEPS: f32 = 1000.0;

impl AudioCue {
    /// Returns every cue once, landing at full strength. Used to audition and to test.
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
            _ => None,
        }
    }
}
