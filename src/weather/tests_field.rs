//! Tests of the cloud field over a real planet: weather is a place, rain falls only under
//! thick cloud, and nothing steps in time or across the cover grid.

use bevy::math::DVec3;

use super::field::{Field, Overhead};
use super::grid::{Patch, SNAPSHOT};
use super::puff::{CELL, WRAP, density_at, drift};
use super::rules::{Planet, RAIN, Weather, forecast, under};
use super::wet::Wet;
use crate::universe::{HOME_RADIUS, PlanetRecord, PlanetSpec, Universe};
use crate::world::Globe;

/// One sixtieth of a second, the frame the smoothness bounds are given for.
const FRAME: f64 = 1.0 / 60.0;

/// Returns `count` unit directions spread evenly over the sphere.
fn sphere(count: u32) -> impl Iterator<Item = DVec3> {
    let turn = std::f64::consts::PI * (3.0 - 5.0_f64.sqrt());
    (0..count).map(move |index| {
        let y = 1.0 - 2.0 * (f64::from(index) + 0.5) / f64::from(count);
        let (sin, cos) = (turn * f64::from(index)).sin_cos();
        let ring = (1.0 - y * y).sqrt();
        DVec3::new(cos * ring, y, sin * ring)
    })
}

/// Returns the home planet of universe 7 and its globe.
fn home() -> (PlanetSpec, Globe) {
    let universe = Universe::new(7);
    let spec = universe.planet(universe.home());
    let globe = Globe::of(&spec);
    (spec, globe)
}

/// Returns the weather on the ground of `globe` in direction `dir` at `seconds`, read the way
/// the game reads it at the camera, with the broad cover it was read under.
fn weather(spec: &PlanetSpec, globe: &Globe, dir: DVec3, seconds: f64) -> (Weather, f32) {
    let field = Field::of(spec, false);
    let patch = Patch::around(dir);
    let overhead = Overhead {
        cover: field.cover(&patch, Wet::default().over(globe, &patch), seconds),
        storm: field.storm(dir, seconds),
        share: field.share(seconds),
        under: 1.0,
    };
    let weather = forecast(&Planet::of(&spec.record), &globe.air(dir, 0.0), &overhead);
    (weather, overhead.cover)
}

/// At one second, two places of one planet have different weather: somewhere it rains or
/// snows and somewhere the sky is clear, and wherever anything falls the broad cover there
/// is at least the cover rain starts at, so the place is drawn under thick cloud.
#[test]
fn weather_is_a_place_and_rain_needs_thick_cloud() {
    let (spec, globe) = home();
    for seconds in [2000.0, 5417.0, 90_000.5] {
        let seen: Vec<(Weather, f32)> = sphere(600)
            .map(|dir| weather(&spec, &globe, dir, seconds))
            .collect();
        let falling = |w: &Weather| w.rain + w.snow;
        for (weather, cover) in &seen {
            assert!(falling(weather) == 0.0 || *cover >= RAIN.0, "{weather:?}");
            assert_eq!(weather.cloud, *cover);
        }
        let wet = seen.iter().filter(|(w, _)| falling(w) > 0.2).count();
        let clear = seen.iter().filter(|(w, _)| w.cloud < 0.2).count();
        assert!(
            wet > 5 && clear > 5,
            "{wet} wet, {clear} clear at {seconds}"
        );
        assert!(
            seen.iter().any(|(w, _)| w.storm > 0.3) && seen.iter().any(|(w, _)| w.storm == 0.0)
        );
    }
}

/// Rain and snow stop at the top of the cloud band and ease out on the way up through it.
#[test]
fn rain_stays_under_the_cloud_top() {
    let (base, top) = (900.0, 1500.0);
    assert_eq!((under(0.0, base, top), under(900.0, base, top)), (1.0, 1.0));
    assert_eq!(
        (under(1500.0, base, top), under(4000.0, base, top)),
        (0.0, 0.0)
    );
    let steps: Vec<f32> = (0..=600)
        .map(|m| under(900.0 + m as f32, base, top))
        .collect();
    assert!(
        steps
            .windows(2)
            .all(|pair| pair[1] <= pair[0] && pair[0] - pair[1] < 0.005)
    );
    assert_eq!(under(1200.0, 1500.0, 1500.0), 1.0);
}

/// The broad cover, the storm and the puffs move by less than a small bound in one frame at
/// every place and second sampled, across the knots of the noise, across the snapshots of
/// the cover and through the calm opening at home: nothing steps.
#[test]
fn the_field_never_steps_in_time() {
    let field = Field {
        home: true,
        ..Field::new(&PlanetRecord::TEMPERATE, 11, HOME_RADIUS)
    };
    let cells = (HOME_RADIUS as f32 + 1200.0) / CELL;
    let knots = [
        0.0,
        97.0,
        300.0,
        480.0,
        SNAPSHOT,
        7.0 * SNAPSHOT,
        2910.0,
        86_400.0,
    ];
    let around = knots
        .iter()
        .flat_map(|knot| [-FRAME, -0.5 * FRAME, 0.0].map(|by| knot + by));
    let times: Vec<f64> = around.chain((0..40).map(|i| 13.7 * f64::from(i))).collect();
    for dir in sphere(40) {
        let patch = Patch::around(dir);
        for &t in times.iter().filter(|t| **t >= 0.0) {
            let cover = |t: f64| field.cover(&patch, [0.7, 0.9, 0.5, 0.8], t);
            assert!((cover(t + FRAME) - cover(t)).abs() < 0.001, "cover at {t}");
            assert!((field.storm(dir, t + FRAME) - field.storm(dir, t)).abs() < 0.002);
            let puff = |t: f64| density_at(dir, cells, t, cover(t));
            assert!((puff(t + FRAME) - puff(t)).abs() < 0.02, "puff at {t}");
        }
    }
    assert!(drift(WRAP - 1.0).length() < 2048.0 && drift(WRAP + 3.0) == drift(3.0));
}

/// Walking a metre at a time round the planet, over the edges of the cube the cover grid is
/// laid on, the broad cover never jumps, and it does change from place to place.
#[test]
fn the_cover_never_steps_from_place_to_place() {
    let field = Field::new(&PlanetRecord::TEMPERATE, 11, HOME_RADIUS);
    let axis = DVec3::new(0.3, 1.0, -0.2).normalize();
    let start = axis.cross(DVec3::X).normalize();
    let metre = 1.0 / HOME_RADIUS;
    let cover = |step: u32| {
        let dir = bevy::math::DQuat::from_axis_angle(axis, f64::from(step) * metre) * start;
        field.cover(&Patch::around(dir), [0.8; 4], 1234.5)
    };
    let walk: Vec<f32> = (0..38_000).map(cover).collect();
    let jump = walk.windows(2).map(|pair| (pair[1] - pair[0]).abs());
    assert!(jump.fold(0.0, f32::max) < 0.003);
    let (low, high) = walk.iter().fold((1.0_f32, 0.0_f32), |(low, high), c| {
        (low.min(*c), high.max(*c))
    });
    assert!(high - low > 0.5, "{low} to {high}");
}
