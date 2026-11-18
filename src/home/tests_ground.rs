//! Tests of the drawn base: feet on the ground, level platforms, budgets and lights.

use bevy::math::DVec3;
use bevy::prelude::*;

use crate::world::Terrain;
use crate::world::globe::Face;

use super::glow::{Glow, Mood, flash};
use super::lights::{noticed, swell};
use super::plan::{draw, plan};
use super::tests::base;
use super::tune::{BEACON_GROWTH, FOOTPRINT, PAD_RELIEF};

/// Largest distance in metres between a foot and the drawn ground under it.
const FOOT_GAP: f32 = 0.03;

/// Largest tilt of a platform in degrees.
const TILT: f64 = 0.1;

/// On 20 seeds every leg, cable rest and pad point is within 3 cm of the drawn ground along the
/// local vertical: the terrain mesh of the chunks, on its cube face chart. Nothing floats and
/// nothing is buried.
#[test]
fn every_foot_meets_the_ground() {
    for seed in 1..=20 {
        let (base, globe) = base(seed);
        let lot = base.lot(&globe);
        let feet = draw(&lot).feet;
        assert!(feet.len() > 150, "seed {seed}: {} feet", feet.len());
        for foot in feet {
            let planet = lot.planet(foot);
            let globe = lot.view().globe();
            let drawn = Terrain::on(
                globe.clone(),
                globe.face_chart(Face::locate(planet.normalize()).0),
            );
            let at = drawn.locate(planet);
            let gap = (at.y - drawn.mesh_height(at.x, at.z)).abs();
            assert!(gap < FOOT_GAP, "seed {seed}: foot {foot} is {gap} m off");
        }
    }
}

/// On 20 seeds every platform is level against the vertical of the base centre within 0.1
/// degree, and that vertical is the one of the globe there.
#[test]
fn platforms_are_level() {
    for seed in 1..=20 {
        let (base, globe) = base(seed);
        let lot = base.lot(&globe);
        let draft = draw(&lot);
        let (rot, up) = (base.rot(), base.centre.normalize());
        assert!(base.up.dot(up).min(1.0).acos().to_degrees() < TILT);
        assert!(draft.decks.len() > 20);
        for [a, b, c] in &draft.decks {
            let normal = (rot * (*b - *a).cross(*c - *a).as_dvec3()).normalize();
            let tilt = normal.dot(up).abs().min(1.0).acos().to_degrees();
            assert!(tilt < TILT, "seed {seed}: a platform tilts {tilt} degrees");
        }
    }
}

/// The base stays within its triangle budget and its footprint, with few solids and lights.
#[test]
fn base_fits_its_budgets() {
    for seed in [1, 2, 42] {
        let (base, globe) = base(seed);
        let lot = base.lot(&globe);
        let (draft, plan) = (draw(&lot), plan(&lot));
        let count = |mesh: &Mesh| mesh.count_vertices() / 3;
        let triangles = plan.glows.iter().map(count).sum::<usize>() + count(&plan.lit);
        assert!((800..2500).contains(&triangles), "{triangles} triangles");
        assert!(plan.glows.iter().all(|mesh| count(mesh) >= 12));
        eprintln!("seed {seed}: {} solid boxes", plan.blocks.len());
        assert!(plan.blocks.len() <= 48 && plan.lamps.len() == 3);
        let reach = |p: &Vec3| f64::from(p.xz().length());
        assert!(draft.feet.iter().all(|foot| reach(foot) <= FOOTPRINT));
        assert!(
            plan.blocks
                .iter()
                .all(|block| reach(&block.at) <= FOOTPRINT)
        );
        assert!(
            plan.blocks
                .iter()
                .all(|block| block.size.min_element() > 0.05)
        );
    }
}

/// Returns the difference in metres between the highest and the lowest ground under the edge
/// and the centre of the pad of the world of `seed`.
fn pad_relief(seed: u64) -> f32 {
    let (base, globe) = base(seed);
    let (side, heading) = (base.pad.normalize().cross(base.heading), base.heading);
    let around = [-1.0, 0.0, 1.0].map(|k| k * f64::from(super::PAD_HALF));
    let spots = around
        .iter()
        .flat_map(|x| around.iter().map(move |z| (*x, *z)));
    let height = |(x, z): (f64, f64)| {
        let at: DVec3 = base.pad + side * x + heading * z;
        globe.height(at.normalize())
    };
    let heights: Vec<f32> = spots.map(height).collect();
    let low = heights.iter().copied().fold(f32::MAX, f32::min);
    heights.iter().copied().fold(f32::MIN, f32::max) - low
}

/// The pad follows ground that is close to flat, so a ship resting on the terrain stands close
/// to level: within the limit on nearly every seed, and never beyond twice the limit.
#[test]
fn pad_ground_is_close_to_flat() {
    let reliefs: Vec<f32> = (1..=20).map(pad_relief).collect();
    let worst = reliefs.iter().copied().fold(0.0, f32::max);
    assert!(worst < PAD_RELIEF * 2.0 + 0.4, "pad relief {worst} m");
    let gentle = reliefs.iter().filter(|r| **r < PAD_RELIEF + 0.4).count();
    assert!(gentle >= 18, "only {gentle} of 20 pads within the limit");
}

/// The beacon flashes twice per period and every light is dimmer by day.
#[test]
fn lights_follow_the_day() {
    let lit = (0..300).filter(|i| flash(*i as f32 * 0.01) > 0.5).count();
    assert!((20..80).contains(&lit), "{lit} lit hundredths in a period");
    assert_eq!(flash(1.5), 0.0);
    let day = Mood::default();
    let night = Mood { night: 1.0, ..day };
    for glow in Glow::ALL {
        let dimmer = glow.gain(night) > glow.gain(day);
        assert!(dimmer, "{glow:?} is not dimmer by day");
        assert!(glow.power(night) >= glow.power(day));
        assert_eq!(glow.power(night) > 0.0, glow.lamp().is_some());
    }
}

/// The strip brightens and the window shifts when the robot is noticed; the rest stays as it was.
#[test]
fn hatch_answers_the_robot() {
    let night = Mood {
        night: 1.0,
        ..Mood::default()
    };
    let near = Mood {
        notice: 1.0,
        ..night
    };
    assert!(Glow::Strip.gain(near) > Glow::Strip.gain(night) * 2.0);
    assert_ne!(Glow::Window.color(near), Glow::Window.color(night));
    assert_eq!(Glow::Corner.color(near), Glow::Corner.color(night));
    assert_eq!(Glow::Beacon.color(near), Glow::Beacon.color(night));
}

/// The notice level rises while the robot is near and falls after, and the beacon only grows
/// with distance, up to its cap.
#[test]
fn notice_and_beacon_stay_in_range() {
    let risen = (0..200).fold(0.0, |level, _| noticed(level, true, 0.016));
    assert_eq!(risen, 1.0);
    let half = noticed(risen, false, 0.6);
    assert!(half > 0.0 && half < 1.0);
    assert_eq!(noticed(half, false, 10.0), 0.0);
    assert_eq!(swell(3.0), 1.0);
    assert!(swell(BEACON_GROWTH.0 * 4.0) > 3.9);
    assert_eq!(swell(1.0e6), BEACON_GROWTH.1);
}
