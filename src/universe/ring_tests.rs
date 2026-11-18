//! Ring tests: how many planets have rings, and that every disc stays inside the Roche limit.

use super::giant_tests::{giant, systems};
use super::rings::{RING_DENSITY, RING_INNER};
use super::*;

/// About 4 gas giants in 10 and 3 solid planets in 100 have rings. Shares are printed.
#[test]
fn rings_have_their_share() {
    let (mut giants, mut solid) = ((0, 0), (0, 0));
    for system in systems() {
        for p in &system.planets {
            let side = if giant(p) { &mut giants } else { &mut solid };
            *side = (side.0 + usize::from(p.rings.is_some()), side.1 + 1);
        }
    }
    let share = |(with, all): (usize, usize)| 100.0 * with as f64 / all as f64;
    println!(
        "rings: {:.1} % of giants, {:.1} % of solid planets",
        share(giants),
        share(solid)
    );
    assert!((35.0..=45.0).contains(&share(giants)), "{giants:?}");
    assert!((2.0..=4.0).contains(&share(solid)), "{solid:?}");
}

/// Every ring disc starts 1.2 to 1.6 radii out, above the air, and ends at most 2.4 radii out:
/// inside the Roche limit and inside the orbit of the innermost moon. Its density is 0.2 to 0.9
/// and its colour is ice or dust.
#[test]
fn rings_stay_inside_the_roche_limit() {
    let (mut icy, mut dusty) = (0, 0);
    for system in systems() {
        for p in &system.planets {
            let Some(rings) = p.rings else { continue };
            let (inner, outer) = (rings.inner / p.radius, rings.outer / p.radius);
            assert!(RING_INNER.0 <= inner && inner <= RING_INNER.1, "{p:?}");
            assert!(inner + 0.3 <= outer && outer <= RING_OUTER && RING_OUTER < ROCHE);
            assert!(rings.inner > p.radius + p.atmosphere_height, "{p:?}");
            assert!(RING_DENSITY.0 <= rings.density && rings.density <= RING_DENSITY.1);
            let first = moon::of(&system.star, p).into_iter().next();
            assert!(first.is_none_or(|m| rings.outer < m.orbit_radius - m.radius));
            let ice = rings.color.x > 0.6;
            assert_eq!(ice, p.record.temperature < 220.0, "{p:?}");
            (icy, dusty) = (icy + usize::from(ice), dusty + usize::from(!ice));
        }
    }
    println!("{icy} icy and {dusty} dusty ring discs");
    assert!(icy > 0 && dusty > 0);
}
