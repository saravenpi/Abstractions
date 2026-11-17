//! The face on the screen: expressions, the frame shown and its colours.

pub mod machine;
pub mod paint;
mod sprites;

use bevy::prelude::*;

/// What the face expresses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Expression {
    /// Calm round eyes.
    #[default]
    Neutral,
    /// Arched eyes and a smile.
    Happy,
    /// One eye bigger than the other.
    Curious,
    /// Round wide eyes and a small mouth.
    Surprised,
    /// Half-closed eyes.
    Sleepy,
    /// Small eyes under raised brows.
    Worried,
    /// Heart eyes, for discoveries.
    Love,
}

impl Expression {
    /// Every expression, in declaration order.
    pub const ALL: [Expression; 7] = [
        Expression::Neutral,
        Expression::Happy,
        Expression::Curious,
        Expression::Surprised,
        Expression::Sleepy,
        Expression::Worried,
        Expression::Love,
    ];

    /// Returns the linear colour of the screen and of its light: warm when happy, cool when curious.
    pub fn tint(self) -> Vec3 {
        match self {
            Expression::Neutral => Vec3::new(0.5, 0.92, 1.0),
            Expression::Happy => Vec3::new(1.0, 0.78, 0.36),
            Expression::Curious => Vec3::new(0.36, 0.7, 1.0),
            Expression::Surprised => Vec3::new(1.0, 1.0, 1.0),
            Expression::Sleepy => Vec3::new(0.5, 0.48, 0.9),
            Expression::Worried => Vec3::new(0.72, 0.62, 1.0),
            Expression::Love => Vec3::new(1.0, 0.4, 0.62),
        }
    }

    /// Returns how brightly the screen glows, from 0 to 1: dim when sleepy.
    pub fn glow(self) -> f32 {
        match self {
            Expression::Sleepy => 0.5,
            Expression::Worried => 0.7,
            Expression::Surprised | Expression::Love => 1.0,
            _ => 0.85,
        }
    }

    /// Returns the seconds each of the two frames of the expression is shown.
    pub fn beat(self) -> f32 {
        match self {
            Expression::Sleepy => 1.6,
            Expression::Neutral | Expression::Curious => 0.9,
            Expression::Happy | Expression::Love => 0.45,
            Expression::Surprised | Expression::Worried => 0.3,
        }
    }
}

/// How far the eyelids are closed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Lid {
    /// Eyes open.
    #[default]
    Open,
    /// Halfway through a blink or a change of expression.
    Half,
    /// Eyes shut.
    Closed,
}

/// One picture of the face. The screen is repainted only when this changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Frame {
    /// The expression.
    pub expr: Expression,
    /// The eyelids.
    pub lid: Lid,
    /// Where the eyes look, each axis -1, 0 or 1: x toward the right of the picture, y up.
    pub gaze: IVec2,
    /// Which of the two frames of the expression is shown.
    pub beat: bool,
}
