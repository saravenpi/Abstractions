//! Tests that the fades of the streamer hand Bevy few distinct visibility ranges, however long the
//! player flies: Bevy never frees the GPU slot of a range, and past 65,535 the ground blinks.

use bevy::camera::visibility::VisibilityRange;

use super::dither::{LEVELS, SPAN, Way, ranged, span};
use crate::render::ranges::{RANGE_BOUND, Seen};

/// Sea level radius of the Earth in metres.
const EARTH: f64 = 6.371e6;

/// Returns every range a fade can write with the margin `wide`: each level, each way.
fn all_levels(wide: f32) -> impl Iterator<Item = VisibilityRange> {
    let ways = [Way::In, Way::Hide, Way::Leave];
    ways.into_iter()
        .flat_map(move |way| (0..=LEVELS).map(move |level| ranged(way, level, wide)))
}

/// Returns the altitudes of a long flight: every metre of the first 20 km, every 10 m to
/// 2,000 km, and a climb by one percent at a time out to 100,000 km, far past the hand-over.
fn flight() -> impl Iterator<Item = f64> {
    let low = (0..20_000).map(f64::from);
    let mid = (2_000..200_000).map(|step| f64::from(step) * 10.0);
    let high = (0..400).map(|step| 2.0e6 * 1.01_f64.powi(step));
    low.chain(mid).chain(high)
}

/// Flying from the ground of an Earth to 100,000 km up, at every metre low down, the
/// fades use fewer distinct ranges than the bound the dev check warns at.
#[test]
fn a_long_flight_uses_few_visibility_ranges() {
    let mut seen = Seen::default();
    let mut last = None;
    for altitude in flight() {
        let wide = span(altitude, EARTH);
        if last == Some(wide) {
            continue;
        }
        last = Some(wide);
        for range in all_levels(wide).chain(all_levels(SPAN)) {
            assert!(
                !seen.note(&range),
                "{} ranges at {altitude} m",
                seen.count()
            );
        }
    }
    assert!(seen.count() <= RANGE_BOUND, "{}", seen.count());
}

/// Every planet from a moon of 100 km to a giant of 100,000 km, seen from any height, needs
/// margins that together stay far under the 65,535 slots of Bevy.
#[test]
fn every_planet_and_height_fit_in_bevy_slots() {
    let mut margins = Vec::new();
    for radius in (0..=60).map(|step| 1.0e5 * 1.122_f64.powi(step)) {
        for altitude in (0..=90).map(|step| 1.25_f64.powi(step) - 1.0) {
            margins.push(span(altitude, radius).to_bits());
        }
    }
    margins.sort_unstable();
    margins.dedup();
    let levels = 3 * (usize::from(LEVELS) + 1);
    assert!(margins.len() * levels < 2_000, "{} margins", margins.len());
}

/// A margin is a power of two once it outgrows the margin of a small planet, never narrower
/// than the share of the farthest point the fade needs, and stays finite at absurd heights.
#[test]
fn a_margin_is_a_power_of_two_wide_enough() {
    for altitude in [0.0, 1.0, 999.0, 1.0e5, 3.3e6, 4.0e8] {
        let wide = span(altitude, EARTH);
        assert!(wide.log2().fract() == 0.0, "{wide}");
        assert!(
            f64::from(wide) >= 64.0 * (2.0 * EARTH + altitude),
            "{altitude}"
        );
    }
    assert!(span(1.0e300, EARTH).is_finite());
    assert!(span(f64::NAN, EARTH).is_finite());
}
