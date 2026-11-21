//! The slideshow of the title screen as numbers: which picture shows, when the next fades in
//! over it, and how each drifts while it shows.

use bevy::prelude::*;

use crate::settings::{Persistent, Settings};
use crate::ui::motion::{LEVEL_FADE, SLIDE, SLIDE_DRIFT, SLIDE_FADE, ease_out};

/// Where the slideshow is. It exists only while the title screen shows.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Slideshow {
    /// Index of the picture shown.
    pub slide: usize,
    /// Pictures shown before this one since the title screen opened. Its parity gives the way
    /// the picture drifts: in on even turns, out on odd ones.
    pub turn: u32,
    /// Seconds this picture has been the one shown, from 0 to [`SLIDE`].
    pub clock: f32,
    /// True with the "Reduce motion" setting: no drift, and a short fade.
    pub calm: bool,
}

impl Slideshow {
    /// Returns a slideshow of `count` pictures that opens on the one `seed` picks.
    pub fn starting(seed: u64, count: usize) -> Self {
        Self {
            slide: (seed % count.max(1) as u64) as usize,
            turn: 0,
            clock: 0.0,
            calm: false,
        }
    }

    /// Returns the seconds of the cross-fade at the end of a picture's time.
    pub fn fade(&self) -> f32 {
        match self.calm {
            true => LEVEL_FADE,
            false => SLIDE_FADE,
        }
    }

    /// Returns the index of the picture after the one shown, the first after the last.
    pub fn next(&self, count: usize) -> usize {
        (self.slide + 1) % count.max(1)
    }

    /// Returns whether the next picture is fading in over the one shown.
    pub fn fading(&self) -> bool {
        self.clock >= SLIDE - self.fade()
    }

    /// Moves the slideshow of `count` pictures on by `dt` seconds. After [`SLIDE`] seconds the
    /// next picture becomes the one shown.
    pub fn tick(&mut self, dt: f32, count: usize) {
        self.clock += dt.max(0.0);
        if self.clock >= SLIDE {
            *self = Self {
                slide: self.next(count),
                turn: self.turn.wrapping_add(1),
                clock: (self.clock - SLIDE).min(SLIDE - SLIDE_FADE),
                calm: self.calm,
            };
        }
    }

    /// Returns the share of the picture shown and of the next one in what the player sees. They
    /// always sum to 1: the next one is drawn over the shown one at its share, eased out.
    pub fn weights(&self) -> (f32, f32) {
        let incoming = ease_out((self.clock - (SLIDE - self.fade())) / self.fade());
        (1.0 - incoming, incoming)
    }

    /// Returns the scale of the picture shown and of the next one: 1 for both when calm. A
    /// picture goes from 1 to 1 + [`SLIDE_DRIFT`] over its whole time on screen on an even turn,
    /// and back on an odd one, so it never jumps where two meet and never uncovers the window.
    pub fn zooms(&self) -> (f32, f32) {
        if self.calm {
            return (1.0, 1.0);
        }
        let zoom = |turn: u32, age: f32| {
            let along = (age / (SLIDE + SLIDE_FADE)).clamp(0.0, 1.0);
            let grown = match turn.is_multiple_of(2) {
                true => along,
                false => 1.0 - along,
            };
            1.0 + SLIDE_DRIFT * grown
        };
        let begun = self.clock - (SLIDE - SLIDE_FADE);
        (
            zoom(self.turn, self.clock + SLIDE_FADE),
            zoom(self.turn.wrapping_add(1), begun),
        )
    }
}

/// Moves the slideshow on by the time of the frame, calm when the settings ask for less motion.
pub fn advance(
    time: Res<Time>,
    settings: Option<Res<Persistent<Settings>>>,
    mut show: ResMut<Slideshow>,
) {
    show.calm = settings.is_some_and(|settings| settings.gameplay.reduce_motion);
    show.tick(time.delta_secs(), super::title::SLIDES.len());
}
