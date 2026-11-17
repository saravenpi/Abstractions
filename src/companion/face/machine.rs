//! The expression state machine: quick changes of expression, timed reactions and natural blinks.

use bevy::prelude::*;

use super::{Expression, Frame, Lid};
use crate::companion::rng::Rng;

/// Shortest and longest time between the starts of two blinks, in seconds.
pub const BLINK_EVERY: (f32, f32) = (2.2, 6.5);

/// Seconds a blink lasts: lids half shut, shut, then half shut again.
pub const BLINK_SECONDS: f32 = 0.16;

/// Seconds a change of expression takes: the old face closes its eyes and the new one opens them.
pub const SHIFT_SECONDS: f32 = 0.14;

/// The face of the companion: what it shows now and what it is changing to.
#[derive(Component, Debug, Clone)]
pub struct Face {
    /// Expression on the screen.
    shown: Expression,
    /// Expression it is changing to.
    next: Expression,
    /// Seconds left of the change, zero when settled.
    shift: f32,
    /// Seconds until the next blink starts.
    blink_in: f32,
    /// Seconds left of the current blink, zero when the eyes are open.
    blink: f32,
    /// Seconds into the two-frame cycle of the expression.
    beat: f32,
    /// A reaction that overrides the ambient expression, with the seconds it still lasts.
    reaction: Option<(Expression, f32)>,
    /// Random numbers for the blink intervals.
    rng: Rng,
}

impl Face {
    /// Returns a neutral face whose blinks follow `seed`.
    pub fn new(seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        Self {
            shown: Expression::Neutral,
            next: Expression::Neutral,
            shift: 0.0,
            blink_in: rng.range(BLINK_EVERY.0, BLINK_EVERY.1),
            blink: 0.0,
            beat: 0.0,
            reaction: None,
            rng,
        }
    }

    /// Returns the expression on the screen.
    pub fn shown(&self) -> Expression {
        self.shown
    }

    /// Shows `expr` for `seconds`, whatever the ambient expression is, then lets the face fall back.
    pub fn react(&mut self, expr: Expression, seconds: f32) {
        self.reaction = Some((expr, seconds));
    }

    /// Moves the change of expression forward and returns the new expression on the frame it appears.
    fn change(&mut self, want: Expression, dt: f32) -> Option<Expression> {
        if self.shift > 0.0 {
            let before = self.shift;
            self.shift = (self.shift - dt).max(0.0);
            let half = SHIFT_SECONDS * 0.5;
            let swapped = before > half && self.shift <= half && self.shown != self.next;
            self.shown = if swapped { self.next } else { self.shown };
            return swapped.then_some(self.shown);
        }
        if want != self.shown {
            self.next = want;
            self.shift = SHIFT_SECONDS;
        }
        None
    }

    /// Advances the face by `dt` seconds toward the `ambient` expression or the current reaction.
    ///
    /// Returns the new expression on the frame the screen changes to it. Once every reaction has
    /// run out, the face settles on `ambient`.
    pub fn advance(&mut self, ambient: Expression, dt: f32) -> Option<Expression> {
        self.reaction = self
            .reaction
            .map(|(expr, left)| (expr, left - dt))
            .filter(|(_, left)| *left > 0.0);
        let want = self.reaction.map_or(ambient, |(expr, _)| expr);
        self.beat = (self.beat + dt) % (2.0 * self.shown.beat());
        self.blink = (self.blink - dt).max(0.0);
        self.blink_in -= dt;
        if self.blink_in <= 0.0 {
            self.blink = BLINK_SECONDS;
            self.blink_in = self.rng.range(BLINK_EVERY.0, BLINK_EVERY.1);
        }
        self.change(want, dt)
    }

    /// Returns how far the eyelids are closed by the blink or the change of expression in progress.
    fn lid(&self) -> Lid {
        let blink = (self.blink / BLINK_SECONDS - 0.5).abs();
        let shift = (self.shift / SHIFT_SECONDS - 0.5).abs();
        let busy = [(self.blink, blink), (self.shift, shift)];
        let middle = busy.iter().any(|(left, mid)| *left > 0.0 && *mid < 0.2);
        let moving = busy.iter().any(|(left, _)| *left > 0.0);
        match (middle, moving) {
            (true, _) => Lid::Closed,
            (false, true) => Lid::Half,
            (false, false) => Lid::Open,
        }
    }

    /// Returns the picture to show now, with the eyes turned by `gaze`.
    pub fn frame(&self, gaze: IVec2) -> Frame {
        Frame {
            expr: self.shown,
            lid: self.lid(),
            gaze,
            beat: self.beat >= self.shown.beat(),
        }
    }
}
