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

impl Lid {
    /// Every lid, from open to shut.
    pub const ALL: [Lid; 3] = [Lid::Open, Lid::Half, Lid::Closed];
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

impl Frame {
    /// Returns the frame as the wire carries it: the rank of the expression, then the lid in
    /// bits 0 and 1, the gaze x plus one in bits 2 and 3, the gaze y plus one in bits 4 and 5
    /// and the beat in bit 6.
    pub fn pack(&self) -> (u8, u8) {
        let expr = Expression::ALL
            .iter()
            .position(|e| *e == self.expr)
            .unwrap_or(0) as u8;
        let lid = Lid::ALL.iter().position(|l| *l == self.lid).unwrap_or(0) as u8;
        let axis = |v: i32| (v.clamp(-1, 1) + 1) as u8;
        let beat = u8::from(self.beat) << 6;
        (
            expr,
            lid | axis(self.gaze.x) << 2 | axis(self.gaze.y) << 4 | beat,
        )
    }

    /// Returns the frame the wire values `expr` and `face` of [`Frame::pack`] stand for. A
    /// value out of range reads as the nearest one.
    pub fn unpack(expr: u8, face: u8) -> Self {
        let rank = |value: u8, len: usize| usize::from(value).min(len - 1);
        let axis = |bits: u8| i32::from((bits & 3).min(2)) - 1;
        Self {
            expr: Expression::ALL[rank(expr, Expression::ALL.len())],
            lid: Lid::ALL[rank(face & 3, Lid::ALL.len())],
            gaze: IVec2::new(axis(face >> 2), axis(face >> 4)),
            beat: face & 64 != 0,
        }
    }
}
