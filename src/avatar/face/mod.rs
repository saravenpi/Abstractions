//! The face on the screen: what it expresses, which state calls for which face, and where the eyes look.
//!
//! It is drawn like the companion's, on the same thirty-two by sixteen screen and in the same
//! colours, with calmer shapes: this is the protagonist.

pub mod machine;
pub mod paint;
mod sprites;

use bevy::prelude::*;

use super::tune::{FAST_FLIGHT, FREE_FALL, SLEEPY_AFTER, WINCE_ABOVE};
use super::{AvatarPose, Mode};
use crate::companion::Expression;

/// What the face expresses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Mien {
    /// Calm tall eyes.
    #[default]
    Neutral,
    /// Narrowed eyes under set brows: sprinting or flying fast.
    Focused,
    /// Round wide eyes: free fall.
    Wide,
    /// Arched eyes and a smile: a soft landing.
    Happy,
    /// Eyes screwed shut: a hard landing.
    Wince,
    /// Half-closed eyes: idle for long.
    Sleepy,
    /// Small eyes over a wobbling held breath: under water.
    Submerged,
}

impl Mien {
    /// Every face, in declaration order.
    #[cfg(test)]
    pub const ALL: [Mien; 7] = [
        Mien::Neutral,
        Mien::Focused,
        Mien::Wide,
        Mien::Happy,
        Mien::Wince,
        Mien::Sleepy,
        Mien::Submerged,
    ];

    /// Returns the companion's expression this face borrows its colour, glow and rhythm from.
    pub fn kin(self) -> Expression {
        match self {
            Mien::Neutral | Mien::Focused => Expression::Neutral,
            Mien::Wide => Expression::Surprised,
            Mien::Happy => Expression::Happy,
            Mien::Wince => Expression::Worried,
            Mien::Sleepy => Expression::Sleepy,
            Mien::Submerged => Expression::Curious,
        }
    }

    /// Returns the linear colour of the screen and of its light, times its brightness from 0 to 1.
    pub fn tint(self) -> Vec3 {
        self.kin().tint() * self.kin().glow()
    }
}

/// How far the eyelids are closed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Lid {
    /// Eyes open.
    #[default]
    Open,
    /// Halfway through a blink or a change of face.
    Half,
    /// Eyes shut.
    Closed,
}

/// One picture of the face. The screen is repainted only when this changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Frame {
    /// The face.
    pub mien: Mien,
    /// The eyelids.
    pub lid: Lid,
    /// Where the eyes look, each axis -1, 0 or 1: x toward the right of the picture, y up.
    pub gaze: IVec2,
    /// Which of the two frames of the face is shown.
    pub beat: bool,
}

/// Returns the face the state `input` calls for after `still` seconds of standing still, when no
/// reaction is running.
pub fn ambient(input: &AvatarPose, still: f32) -> Mien {
    let fast = input.airspeed > FAST_FLIGHT;
    match input.mode {
        Mode::Dive => Mien::Submerged,
        Mode::Fall if input.vertical_speed < -FREE_FALL => Mien::Wide,
        Mode::Run => Mien::Focused,
        Mode::Glide | Mode::Fly | Mode::BootFall if fast => Mien::Focused,
        Mode::Idle if still > SLEEPY_AFTER => Mien::Sleepy,
        _ => Mien::Neutral,
    }
}

/// Returns the face a landing of hardness `hard` calls for: a smile when soft, a wince when hard.
pub fn landing(hard: f32) -> Mien {
    match hard > WINCE_ABOVE {
        true => Mien::Wince,
        false => Mien::Happy,
    }
}

/// Returns where the eyes go for a head looking by `look`: a step toward where the head turns.
///
/// The picture is seen from the front, so a look to the body's left moves the eyes to the right of it.
pub fn gaze(look: Vec2) -> IVec2 {
    let step = |turn: f32| i32::from(turn > 0.25) - i32::from(turn < -0.25);
    IVec2::new(step(look.x), step(look.y))
}
