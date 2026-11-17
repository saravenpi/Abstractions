//! The face state machine: quick changes of face, timed reactions and natural blinks.

use bevy::prelude::*;

use super::{Frame, Lid, Mien};
use crate::avatar::Mode;

/// Shortest and longest time between the starts of two blinks, in seconds.
pub const BLINK_EVERY: (f32, f32) = (2.6, 7.0);

/// Seconds a blink lasts: lids half shut, shut, then half shut again.
pub const BLINK_SECONDS: f32 = 0.16;

/// Seconds a change of face takes: the old face closes its eyes and the new one opens them.
pub const SHIFT_SECONDS: f32 = 0.14;

/// The face of the body: what it shows now and what it is changing to.
#[derive(Component, Debug, Clone, PartialEq)]
pub struct Visor {
    /// Face on the screen.
    shown: Mien,
    /// Face it is changing to.
    next: Mien,
    /// Seconds left of the change, zero when settled.
    shift: f32,
    /// Seconds until the next blink starts.
    blink_in: f32,
    /// Seconds left of the current blink, zero when the eyes are open.
    blink: f32,
    /// Seconds into the two-frame cycle of the face.
    beat: f32,
    /// A reaction that overrides the ambient face, with the seconds it still lasts.
    reaction: Option<(Mien, f32)>,
    /// Number of blinks so far, which picks the next interval.
    blinks: u32,
    /// The mode seen last, so a landing is noticed once.
    pub was: Mode,
}

/// Returns the seconds until blink number `count`: the same sequence on every run.
pub fn interval(count: u32) -> f32 {
    let mut x = count.wrapping_mul(0x9E37_79B9) ^ 0x1234_5678;
    x = (x ^ (x >> 16)).wrapping_mul(0x7feb_352d);
    x = (x ^ (x >> 15)).wrapping_mul(0x846c_a68b);
    let unit = ((x ^ (x >> 16)) >> 8) as f32 / 16_777_216.0;
    BLINK_EVERY.0 + (BLINK_EVERY.1 - BLINK_EVERY.0) * unit
}

impl Default for Visor {
    /// Returns a neutral face that has not blinked yet.
    fn default() -> Self {
        Self {
            shown: Mien::Neutral,
            next: Mien::Neutral,
            shift: 0.0,
            blink_in: interval(0),
            blink: 0.0,
            beat: 0.0,
            reaction: None,
            blinks: 0,
            was: Mode::Idle,
        }
    }
}

impl Visor {
    /// Returns the face on the screen.
    pub fn shown(&self) -> Mien {
        self.shown
    }

    /// Shows `mien` for `seconds`, whatever the ambient face is, then lets the face fall back.
    pub fn react(&mut self, mien: Mien, seconds: f32) {
        self.reaction = Some((mien, seconds));
    }

    /// Moves the change of face forward: the faces swap halfway, while the eyes are shut.
    fn change(&mut self, want: Mien, dt: f32) {
        if self.shift > 0.0 {
            let before = self.shift;
            self.shift = (self.shift - dt).max(0.0);
            let half = SHIFT_SECONDS * 0.5;
            if before > half && self.shift <= half {
                self.shown = self.next;
            }
        } else if want != self.shown {
            self.next = want;
            self.shift = SHIFT_SECONDS;
        }
    }

    /// Advances the face by `dt` seconds toward the `ambient` face or the current reaction.
    pub fn advance(&mut self, ambient: Mien, dt: f32) {
        self.reaction = self
            .reaction
            .map(|(mien, left)| (mien, left - dt))
            .filter(|(_, left)| *left > 0.0);
        let want = self.reaction.map_or(ambient, |(mien, _)| mien);
        self.beat = (self.beat + dt) % (2.0 * self.shown.kin().beat());
        self.blink = (self.blink - dt).max(0.0);
        self.blink_in -= dt;
        if self.blink_in <= 0.0 {
            self.blinks += 1;
            self.blink = BLINK_SECONDS;
            self.blink_in = interval(self.blinks);
        }
        self.change(want, dt);
    }

    /// Returns how far the eyelids are closed by the blink or the change of face in progress.
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
            mien: self.shown,
            lid: self.lid(),
            gaze,
            beat: self.beat >= self.shown.kin().beat(),
        }
    }
}
