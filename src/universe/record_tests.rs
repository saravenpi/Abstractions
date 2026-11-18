//! Record tests: over thousands of planets, every class shows up and every record holds together.

use std::f64::consts::FRAC_PI_2;

use super::class::MOLTEN;
use super::climate::FREEZING;
use super::range_tests::sample;
use super::*;

/// Returns every planet of the star sample but the pinned home planets, each with its star.
fn survey() -> Vec<(StarSpec, PlanetSpec)> {
    let mut planets = Vec::new();
    for (u, star) in sample() {
        let system = u.system(star.id);
        let free = system.planets.into_iter().filter(|p| p.id != u.home());
        planets.extend(free.map(|p| (star.clone(), p)));
    }
    assert!(planets.len() >= 5000, "only {} planets", planets.len());
    planets
}

/// Returns the percentage of each class among the planets that pass `keep`, and their count.
fn shares(planets: &[(StarSpec, PlanetSpec)], keep: impl Fn(&StarSpec) -> bool) -> ([f64; 6], f64) {
    let mut counts = [0.0; 6];
    for (_, p) in planets.iter().filter(|(star, _)| keep(star)) {
        counts[p.record.class as usize] += 1.0;
    }
    let total: f64 = counts.iter().sum();
    (counts.map(|n| (1000.0 * n / total).round() / 10.0), total)
}

/// Every class is between 5 and 40 percent of the planets. Shares are printed per star class.
#[test]
fn every_class_has_its_share() {
    let planets = survey();
    let (all, total) = shares(&planets, |_| true);
    println!("ocean, island, desert, ice, lava, barren");
    println!("all {total}: {all:?}");
    for class in [StarClass::A, StarClass::F, StarClass::G, StarClass::K] {
        println!("{class:?} {:?}", shares(&planets, |s| s.class == class));
    }
    println!("M {:?}", shares(&planets, |s| s.class == StarClass::M));
    assert!(
        all.iter().all(|share| (5.0..=40.0).contains(share)),
        "{all:?}"
    );
}

/// Ocean and island worlds have air and liquid water between freezing and their boiling point.
#[test]
fn wet_worlds_hold_liquid_water() {
    for (_, p) in survey() {
        let r = p.record;
        let wet = matches!(r.class, PlanetClass::Ocean | PlanetClass::Island);
        assert_eq!(
            wet,
            r.water == Water::Liquid && r.pressure >= air::NO_AIR,
            "{r:?}"
        );
        if !wet {
            continue;
        }
        assert!(
            r.temperature >= FREEZING && r.temperature < air::boiling(r.pressure),
            "{r:?}"
        );
        let ocean = r.class == PlanetClass::Ocean;
        let span = if ocean { (0.85, 1.0) } else { (0.4, 0.75) };
        assert!(span.0 <= r.ocean && r.ocean <= span.1, "{r:?}");
    }
}

/// Barren worlds have no air, ice worlds are frozen, lava worlds are molten and the others dry.
#[test]
fn dry_worlds_match_their_class() {
    let mut heat = [(0.0, 0.0); 6];
    for (_, p) in survey() {
        let r = p.record;
        heat[r.class as usize].0 += r.temperature;
        heat[r.class as usize].1 += 1.0;
        assert_eq!(
            r.class == PlanetClass::Barren,
            r.pressure < air::NO_AIR,
            "{r:?}"
        );
        let molten = r.class == PlanetClass::Lava;
        assert_eq!(molten, r.pressure >= air::NO_AIR && r.temperature >= MOLTEN);
        assert_eq!(
            molten,
            (0.1..=0.4).contains(&r.ocean) && r.water != Water::Liquid
        );
        if r.class == PlanetClass::Ice {
            assert!(r.water == Water::Ice && r.temperature < FREEZING && r.ocean == 0.0);
        }
        if matches!(r.class, PlanetClass::Desert | PlanetClass::Barren) {
            assert!(r.ocean == 0.0 && (r.water != Water::Liquid || r.pressure < air::NO_AIR));
        }
    }
    let mean = heat.map(|(sum, count)| sum / count);
    println!("mean kelvin, ocean to barren: {mean:.0?}");
    let lava = mean[PlanetClass::Lava as usize];
    assert!(mean.iter().filter(|&&kelvin| kelvin >= lava).count() == 1);
}

/// Within a system the flux falls from each planet to the next one out, and the zone follows it,
/// the home planet included.
#[test]
fn flux_falls_outward() {
    for (u, star) in sample() {
        let planets = u.system(star.id).planets;
        for pair in planets.windows(2) {
            let (near, far) = (pair[0].record, pair[1].record);
            assert!(near.flux > far.flux, "{near:?} {far:?}");
            assert!(near.zone as u8 <= far.zone as u8, "{near:?} {far:?}");
        }
    }
}

/// Every field of every record lies inside the range `PlanetRecord::derive` documents.
#[test]
fn records_stay_in_range() {
    let within = |x: f64, low: f64, high: f64| low <= x && x <= high;
    for (star, p) in survey() {
        let r = p.record;
        assert!(within(star.temperature, 2600.0, 45_000.0), "{star:?}");
        assert_eq!(r.flux, zone::flux(star.luminosity, p.orbit_radius));
        assert_eq!(r.zone, zone::zone(star.temperature, r.flux));
        assert!(within(r.flux, 0.005, 58_000.0), "{r:?}");
        assert!(within(r.tilt, 0.0, FRAC_PI_2), "{r:?}");
        assert!(within(r.pressure, 0.0, air::PRESSURE_CAP), "{r:?}");
        assert!(within(r.albedo, 0.08, 0.6), "{r:?}");
        assert!(within(r.greenhouse, 0.0, 1500.0), "{r:?}");
        assert!(within(r.temperature, 40.0, 6000.0), "{r:?}");
        assert!(within(r.ocean, 0.0, 1.0), "{r:?}");
        assert_eq!(r.class, class::classify(&r));
        assert!(p.atmosphere_height > 0.0 && p.cloud_base > 0.0 && p.cloud_top > 0.0);
    }
}

/// The same seed gives the same records, and two planets of a system do not share their draws.
#[test]
fn records_repeat() {
    for seed in 0..40 {
        let (a, b) = (Universe::new(seed), Universe::new(seed));
        let star = a.home().star;
        let (first, again) = (a.system(star).planets, b.system(star).planets);
        let records = |planets: &[PlanetSpec]| planets.iter().map(|p| p.record).collect::<Vec<_>>();
        assert_eq!(records(&first), records(&again));
        assert_ne!(first[0].record.tilt, first[1].record.tilt);
    }
}
