//! The numbers a player compares tools by, and how a number reads against the held tool's.

use crate::craft::build::{Build, Intent, Place, blow, lives, mass, reference, strike, swing};

/// What a tool does here, in the units the workshop shows.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sheet {
    /// Mass in kilograms.
    pub mass: f32,
    /// Speed of the head at the end of a swing, in metres per second.
    pub speed: f32,
    /// Layers of the reference target it goes through, with the share of the next.
    pub depth: f32,
    /// Blows its weakest joint lasts.
    pub life: f32,
    /// Seconds from rest to the blow.
    pub windup: f32,
}

/// Returns the numbers of `build` used for `intent` at `place`.
pub fn sheet(build: &Build, intent: Intent, place: &Place) -> Sheet {
    let swung = swing(build, place);
    let hit = strike(&blow(build, intent, place), &reference());
    Sheet {
        mass: mass(&build.pieces()),
        speed: swung.speed,
        depth: hit.score(),
        life: lives(build, &swung)
            .into_iter()
            .fold(f32::INFINITY, f32::min),
        windup: swung.windup,
    }
}

/// Returns `value` with `digits` decimals and its `unit`, followed by its difference from `held`
/// in brackets when there is a held value.
pub fn versus(value: f32, held: Option<f32>, unit: &str, digits: usize) -> String {
    let shown = format!("{value:.digits$}{unit}");
    match held {
        Some(was) => {
            let by = value - was;
            let sign = if by >= 0.0 { "+" } else { "-" };
            format!("{shown} ({sign}{:.digits$})", by.abs())
        }
        None => shown,
    }
}

/// Returns a count of blows as the workshop shows it: whole, and no more than 9999.
pub fn blows(life: f32) -> f32 {
    life.min(9999.0).floor()
}
