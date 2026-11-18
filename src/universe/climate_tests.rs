//! Climate tests: each rule of the planet physics on its own, against values worked out by hand.

use super::climate::{FREEZING, settle, spread, state};
use super::*;

/// The home star is at exactly 5,780 K and a star of the same light per surface is as hot.
#[test]
fn star_temperature_follows_light_per_surface() {
    assert_eq!(star::temperature(1.0, 1.0), 5780.0);
    assert!((star::temperature(4.0, 2.0) - 5780.0).abs() < 1e-9);
    assert!((star::temperature(16.0, 1.0) - 11_560.0).abs() < 1e-9);
    assert!(star::temperature(0.05, 0.3) < 5780.0);
}

/// Flux is 1 at the reference orbit of a unit star and thins as the square of the distance.
#[test]
fn flux_thins_with_the_square_of_the_orbit() {
    assert_eq!(zone::flux(1.0, zone::FLUX_ORBIT), 1.0);
    assert!((zone::flux(1.0, 2.0 * zone::FLUX_ORBIT) - 0.25).abs() < 1e-12);
    assert!((zone::flux(3.0, 0.5 * zone::FLUX_ORBIT) - 12.0).abs() < 1e-12);
}

/// Around the home star the habitable zone runs from a flux of 1.0512 down to 0.3438.
#[test]
fn zone_edges_match_the_fit() {
    let zone = |flux| zone::zone(5780.0, flux);
    assert_eq!((zone(1.06), zone(1.05)), (Zone::Hot, Zone::Habitable));
    assert_eq!((zone(0.35), zone(0.34)), (Zone::Habitable, Zone::Cold));
    assert_eq!(zone::zone(3000.0, 1.0), Zone::Hot);
    assert_eq!(zone::zone(3000.0, 0.25), Zone::Habitable);
    assert_eq!(zone::zone(100.0, 0.9), zone::zone(2600.0, 0.9));
    assert_eq!(zone::zone(7200.0, 1.2), zone::zone(40_000.0, 1.2));
}

/// Gravity holds air, heat strips it, small light hot planets keep none and none passes the cap.
#[test]
fn air_follows_gravity_and_heat() {
    let mid = air::pressure(0.5, 1.0, 1.0, 1.0);
    assert!(air::pressure(0.5, 1.5, 1.0, 1.0) > mid);
    assert!(air::pressure(0.5, 1.0, 3.0, 1.0) > mid);
    assert!(air::pressure(0.5, 1.0, 1.0, 30.0) < mid);
    assert!(air::pressure(0.7, 1.0, 1.0, 1.0) > mid);
    assert!(air::pressure(0.9, 0.4, 0.6, 50.0) < air::NO_AIR);
    assert_eq!(air::pressure(1.0, 1.8, 6.0, 0.1), air::PRESSURE_CAP);
}

/// The home air warms by 33 K, water boils at 373 K under it and hotter under thicker air.
#[test]
fn greenhouse_and_boiling_follow_pressure() {
    assert_eq!(air::greenhouse(1.0), 33.0);
    assert!(air::greenhouse(4.0) > air::greenhouse(1.0));
    assert!((air::boiling(1.0) - 373.15).abs() < 1e-9);
    assert!((424.0..427.0).contains(&air::boiling(5.0)));
    assert!(air::boiling(air::TRIPLE_POINT) < FREEZING);
}

/// Water is none when dry or warm without air, ice under 273 K, liquid up to boiling, then steam.
#[test]
fn water_state_follows_heat_and_pressure() {
    assert_eq!(state(288.0, 1.0, 0.05), Water::None);
    assert_eq!(state(272.9, 1.0, 0.5), Water::Ice);
    assert_eq!(state(288.0, 0.005, 0.5), Water::None);
    assert_eq!(state(273.0, 1.0, 0.5), Water::Liquid);
    assert_eq!(state(373.0, 1.0, 0.5), Water::Liquid);
    assert_eq!(state(374.0, 1.0, 0.5), Water::Steam);
    assert_eq!(state(400.0, 5.0, 0.5), Water::Liquid);
    assert!((0.4..=0.75).contains(&spread(0.1)) && (0.4..=0.75).contains(&spread(0.5499)));
    assert!((0.85..=1.0).contains(&spread(0.55)) && (0.85..=1.0).contains(&spread(0.9999)));
}

/// A planet like the home one settles near 288 K with seas, and the same far out freezes over.
#[test]
fn climate_settles_like_the_home_planet() {
    let home = settle(1.0, 1.0, 0.3);
    assert_eq!(home.water, Water::Liquid);
    assert!((0.2..0.4).contains(&home.albedo), "{home:?}");
    assert!((280.0..296.0).contains(&home.temperature), "{home:?}");
    let sum = 278.5 * (1.0 - home.albedo).powf(0.25) + home.greenhouse;
    assert_eq!(home.temperature, sum);
    assert_eq!(home, settle(1.0, 1.0, 0.3));
    let far = settle(0.02, 1.0, 0.3);
    assert_eq!(far.water, Water::Ice);
    assert!(far.albedo > home.albedo && far.temperature < FREEZING);
    let near = settle(40.0, 1.0, 0.3);
    assert_eq!(near.water, Water::Steam);
    assert!(near.greenhouse > 10.0 * home.greenhouse);
}

/// Most planets lean by less than 30 degrees, a few lie on their side, none past a quarter turn.
#[test]
fn tilt_leans_low() {
    let star = Universe::new(3).system(Universe::new(3).home().star).star;
    let tilts: Vec<f64> = (0..4000u64)
        .map(|seed| PlanetRecord::derive(&star, 1e8, 1.0, 1.0, hash::mix(seed)).tilt)
        .collect();
    let share = |from: f64, to: f64| {
        let hits = tilts
            .iter()
            .filter(|t| (from..=to).contains(&t.to_degrees()));
        hits.count() as f64 / tilts.len() as f64
    };
    assert_eq!(share(0.0, 90.0), 1.0);
    assert!(
        (0.6..0.8).contains(&share(0.0, 30.0)),
        "{}",
        share(0.0, 30.0)
    );
    assert!((0.01..0.1).contains(&share(80.0, 90.0)));
}
