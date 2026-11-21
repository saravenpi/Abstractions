//! The numbers a player compares tools by, and the four bars a tool card draws from them.

use super::tune::{FULL_DEPTH, FULL_MASS, FULL_REACH, FULL_SPEED};
use crate::craft::build::{Build, Intent, Place, blow, lives, mass, reference, strike, swing};
use crate::ui::units::fixed;

/// What a tool does here, in the units the workshop shows.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sheet {
    /// Mass in kilograms.
    pub mass: f32,
    /// From the hand to the far end of the head, in metres.
    pub reach: f32,
    /// Speed of the head at the end of a swing, in metres per second.
    pub speed: f32,
    /// Layers of the reference target it goes through, with the share of the next.
    pub depth: f32,
    /// Blows its weakest joint lasts.
    pub life: f32,
}

/// One bar of a tool card: what it measures, how full it is, 0 to 1, and the number in words.
#[derive(Debug, Clone, PartialEq)]
pub struct Gauge {
    /// What the bar measures.
    pub label: &'static str,
    /// How full the bar is, 0 to 1.
    pub share: f32,
    /// The number with its unit.
    pub value: String,
}

/// Returns the numbers of `build` used for `intent` at `place`.
pub fn sheet(build: &Build, intent: Intent, place: &Place) -> Sheet {
    let swung = swing(build, place);
    let hit = strike(&blow(build, intent, place), &reference());
    Sheet {
        mass: mass(&build.pieces()),
        reach: build.reach(),
        speed: swung.speed,
        depth: hit.score(),
        life: lives(build, &swung)
            .into_iter()
            .fold(f32::INFINITY, f32::min),
    }
}

/// Returns a count of blows as the workshop shows it: whole, and no more than 9999.
pub fn blows(life: f32) -> f32 {
    life.min(9999.0).floor()
}

/// Returns the name of what the head does for `intent`: its edge, its point or its impact.
fn bite(intent: Intent) -> &'static str {
    match intent {
        Intent::Cut => "Edge",
        Intent::Pierce | Intent::Throw => "Point",
        Intent::Crush => "Impact",
    }
}

/// Returns the share of `value` against `full`, held within 0 to 1.
fn share(value: f32, full: f32) -> f32 {
    (value / full).clamp(0.0, 1.0)
}

/// Returns the four bars of a tool for `intent`: reach, mass, how deep its edge, point or impact
/// goes into the reference target, and the speed of its swing.
pub fn gauges(sheet: &Sheet, intent: Intent) -> [Gauge; 4] {
    let depth = share(sheet.depth, FULL_DEPTH);
    [
        Gauge {
            label: "Reach",
            share: share(sheet.reach, FULL_REACH),
            value: format!("{} m", fixed(sheet.reach, 2)),
        },
        Gauge {
            label: "Mass",
            share: share(sheet.mass, FULL_MASS),
            value: format!("{} kg", fixed(sheet.mass, 2)),
        },
        Gauge {
            label: bite(intent),
            share: depth,
            value: format!("{} %", fixed(depth * 100.0, 0)),
        },
        Gauge {
            label: "Swing",
            share: share(sheet.speed, FULL_SPEED),
            value: format!("{} m/s", fixed(sheet.speed, 1)),
        },
    ]
}
