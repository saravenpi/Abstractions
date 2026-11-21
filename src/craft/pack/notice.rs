//! The notice of what the robot just picked up, for the prompt line: "Flint 1.2 kg".

use bevy::prelude::*;

use super::tune::NOTICE;
use super::words::called;
use crate::craft::matter::Stuff;

/// What the robot just picked up, and the seconds the prompt line still shows it.
#[derive(Resource, Debug, Clone, PartialEq, Default)]
pub struct Notice {
    /// The words: the material and its mass.
    pub line: String,
    /// Seconds left on screen.
    pub left: f32,
}

impl Notice {
    /// Returns the notice of `kilograms` of `stuff` picked up, fresh: its name, capitalised, and
    /// its mass to a tenth of a kilogram.
    pub fn of(stuff: &Stuff, kilograms: f32) -> Self {
        let name = called(stuff);
        let mut chars = name.chars();
        let first = chars.next().map(|c| c.to_ascii_uppercase());
        let line = format!(
            "{}{} {kilograms:.1} kg",
            first.unwrap_or_default(),
            chars.as_str()
        );
        Self { line, left: NOTICE }
    }

    /// Returns the words while the notice shows.
    pub fn shown(&self) -> Option<&str> {
        (self.left > 0.0).then_some(self.line.as_str())
    }
}

/// Run condition: a notice shows.
pub fn fading(notice: Res<Notice>) -> bool {
    notice.left > 0.0
}

/// Counts the time a notice has left.
pub fn fade(time: Res<Time>, mut notice: ResMut<Notice>) {
    notice.left = (notice.left - time.delta_secs()).max(0.0);
}
