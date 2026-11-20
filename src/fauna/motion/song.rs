//! The call layer: a body that calls moves to the rhythm of its notes. The head lifts and reaches,
//! the jaw drops, the chest or the bell pumps on every note, and a threat holds a display: the body
//! rises and spreads and the head drops to show its horns.

use std::f32::consts::PI;

use bevy::math::Vec3;

use crate::life::signs::calls::{Cry, Rhythm};

/// Radians the head lifts at the height of a note.
const LIFT: f32 = 0.35;

/// Radians the head drops through a threat display.
const BOW: f32 = 0.45;

/// Radians the jaw drops at the height of a note.
pub const JAW: f32 = 0.5;

/// Share the chest or bell swells at the height of a note.
const PUMP: f32 = 0.08;

/// Share of its hip height a body rises through a threat display.
const RAISE: f32 = 0.12;

/// Share a body spreads sideways through a threat display.
const SPREAD: f32 = 0.12;

/// Seconds a display takes to rise and to fall.
const DISPLAY_EASE: f32 = 0.35;

/// A call the body is making.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Song {
    /// Seconds since the call started.
    pub age: f32,
    /// Its timing, the one its sound follows.
    pub rhythm: Rhythm,
    /// Why it calls.
    pub cry: Cry,
}

impl Song {
    /// Returns true while the call still moves the body.
    pub fn live(&self) -> bool {
        (0.0..self.length()).contains(&self.age)
    }

    /// Returns the seconds the call moves the body, the display easing out included.
    fn length(&self) -> f32 {
        let ease = if self.cry == Cry::Threat {
            DISPLAY_EASE
        } else {
            0.0
        };
        self.rhythm.seconds() + ease
    }

    /// Returns how open the call is now, 0 between notes to 1 at the height of a note.
    pub fn open(&self) -> f32 {
        let period = self.rhythm.period().max(1e-3);
        let k = (self.age / period).floor();
        let within = self.age - k * period;
        let sounding =
            self.age >= 0.0 && k < f32::from(self.rhythm.notes) && within < self.rhythm.note;
        match sounding {
            true => (PI * within / self.rhythm.note.max(1e-3)).sin(),
            false => 0.0,
        }
    }

    /// Returns how far a threat display is held now, 0 to 1, easing in and out.
    pub fn display(&self) -> f32 {
        if self.cry != Cry::Threat || !self.live() {
            return 0.0;
        }
        let ease = |t: f32| (t / DISPLAY_EASE).clamp(0.0, 1.0);
        ease(self.age).min(ease(self.length() - self.age))
    }

    /// Returns the radians the head pitches up: it reaches on each note and bows in a display.
    pub fn pitch(&self) -> f32 {
        LIFT * self.open() - BOW * self.display()
    }

    /// Returns the scale of a chest that pumps and spreads.
    pub fn chest(&self) -> Vec3 {
        let pump = PUMP * self.open();
        let spread = SPREAD * self.display();
        Vec3::new(1.0 + pump + spread, 1.0 + pump, 1.0)
    }

    /// Returns the scale a squashing bell or blob takes on top of its own, its volume kept.
    pub fn squash(&self) -> Vec3 {
        let s = 1.0 + PUMP * self.open();
        Vec3::new(s, 1.0 / (s * s), s)
    }

    /// Returns how far a body of hip height `height` metres rises through a display.
    pub fn raise(&self, height: f32) -> f32 {
        RAISE * height * self.display()
    }
}
