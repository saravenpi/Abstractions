//! Tests of the column of air: real scale heights at real sizes, clouds in the weather layer,
//! and heights that hold together from a 3 km planet to a 70,000 km giant.

use std::time::Instant;

use super::column::scale_height;
use super::tests_mix::{four, world};
use super::tune::{LOOSEST, TOP_SCALES};
use super::{Mix, born, dense, heights};
use crate::universe::{PlanetClass, PlanetRecord, Water};

/// Radius in metres and surface gravity in metres per second squared of the Earth, Mars, Venus
/// and Titan, in the order of [`four`].
const BODIES: [(f64, f64); 4] = [
    (6.371e6, 9.81),
    (3.390e6, 3.71),
    (6.052e6, 8.87),
    (2.575e6, 1.352),
];

/// Returns the four records of [`four`], each with the air it is born with under seed 1.
fn aired() -> [PlanetRecord; 4] {
    four().map(|(record, speed)| PlanetRecord {
        mix: born(&record, speed, 0.0, 1),
        ..record
    })
}

/// The home record at the size and the pull of the Earth has a scale height of 8.4 km, between
/// 7 and 10, an atmosphere 12 of them high, clouds from about 1 km up to about 11 km.
#[test]
fn an_earth_has_the_air_of_the_earth() {
    let (radius, gravity) = BODIES[0];
    let scale = scale_height(&PlanetRecord::TEMPERATE, gravity, radius);
    assert!(7000.0 < scale && scale < 10_000.0, "{scale}");
    assert!((scale - 8430.0).abs() < 50.0, "{scale}");
    let [top, base, cloud_top] = heights(&PlanetRecord::TEMPERATE, gravity, radius);
    assert_eq!(top, TOP_SCALES * scale);
    assert!(800.0 < base && base < 1500.0, "{base}");
    assert!(9000.0 < cloud_top && cloud_top < 13_000.0, "{cloud_top}");
}

/// An Earth without life, Mars, Venus and Titan get the scale heights they really have, within
/// a kilometre or two: about 8.3, 11, 16 and 21 km. The heights are printed.
#[test]
fn other_worlds_get_their_real_scale_heights() {
    let real = [
        (7.8e3, 8.8e3),
        (10.0e3, 12.0e3),
        (15.0e3, 17.0e3),
        (19.0e3, 23.0e3),
    ];
    for ((record, (radius, gravity)), (low, high)) in aired().into_iter().zip(BODIES).zip(real) {
        let scale = scale_height(&record, gravity, radius);
        let [top, base, cloud_top] = heights(&record, gravity, radius);
        let (gas, thick) = (record.mix.main_gas(), dense(&record));
        println!(
            "{gas:?}: scale {scale:.0} top {top:.0} clouds {base:.0} to {cloud_top:.0} dense {thick:.3}"
        );
        assert!(low < scale && scale < high, "{scale}");
        assert!(0.0 < base && base < cloud_top && cloud_top < top);
    }
}

/// Clouds sit lower over a wet world than over a dry one, top out higher under thicker air, and
/// air too thin for a weather layer still has clouds up to one scale height.
#[test]
fn clouds_follow_the_water_and_the_pressure() {
    let (radius, gravity) = BODIES[0];
    let at = |record: &PlanetRecord| heights(record, gravity, radius);
    let sea = |ocean: f64| PlanetRecord {
        ocean,
        ..PlanetRecord::TEMPERATE
    };
    assert!(at(&sea(0.95))[1] < at(&sea(0.6))[1] && at(&sea(0.6))[1] < at(&sea(0.05))[1]);
    let thick = world(PlanetClass::Island, 288.0, 4.0, Water::Liquid);
    assert!(at(&thick)[2] > at(&PlanetRecord::TEMPERATE)[2]);
    let thin = world(PlanetClass::Desert, 288.0, 0.05, Water::None);
    let scale = scale_height(&thin, gravity, radius);
    assert_eq!(at(&thin)[2], scale);
    assert!(at(&thin)[1] <= 0.5 * scale);
}

/// From a 3 km planet to a 70,000 km giant, hot or cold, thin or thick, the heights are finite
/// and ordered, the scale height never passes the radius over 20, and at real sizes it is the
/// free one, which a stronger pull lowers and heat raises.
#[test]
fn heights_hold_together_at_every_size() {
    let records = aired();
    for step in 0..=40 {
        let radius = 3000.0 * (7.0e7f64 / 3000.0).powf(f64::from(step) / 40.0);
        for gravity in [1.0, 3.7, 9.81, 22.0, 40.0] {
            for record in records {
                let [top, base, cloud_top] = heights(&record, gravity, radius);
                assert!(top.is_finite() && 0.0 < base && base < cloud_top && cloud_top < top);
                assert!(
                    top <= TOP_SCALES * radius / LOOSEST + 1e-6,
                    "{top} at {radius}"
                );
            }
        }
    }
    let free = |record: &PlanetRecord, gravity: f64| scale_height(record, gravity, 6.371e6);
    assert!(free(&records[0], 20.0) < free(&records[0], 9.81));
    assert!(free(&records[2], 9.81) > free(&records[1], 9.81));
    assert_eq!(scale_height(&records[0], 22.0, 6000.0), 6000.0 / LOOSEST);
}

/// The air of the home planet has a density of exactly 1 and exactly its pressure under any
/// other; cold air and heavy air are denser, hot air and light air thinner.
#[test]
fn density_follows_the_gas_law() {
    assert_eq!(dense(&PlanetRecord::TEMPERATE), 1.0);
    for pressure in [0.0, 0.005, 0.01, 0.3, 3.0, 40.0] {
        let record = PlanetRecord {
            pressure,
            ..PlanetRecord::TEMPERATE
        };
        assert_eq!(dense(&record), pressure);
    }
    let [_, mars, venus, titan] = aired();
    assert!((dense(&titan) / 4.4 - 1.0).abs() < 0.1, "{}", dense(&titan));
    assert!(
        (dense(&mars) / 0.0125 - 1.0).abs() < 0.1,
        "{}",
        dense(&mars)
    );
    assert!(
        (dense(&venus) / 2.95 - 1.0).abs() < 0.1,
        "{}",
        dense(&venus)
    );
    let helium = PlanetRecord {
        mix: Mix::of([(super::Gas::Helium, 1.0); 4]),
        ..PlanetRecord::TEMPERATE
    };
    assert!(dense(&helium) < 0.15);
}

/// The heights of a planet cost well under 2 microseconds: the player and the gear ask for
/// them every step.
#[test]
fn heights_are_cheap() {
    let records = aired();
    let start = Instant::now();
    let mut sum = 0.0;
    for i in 0..100_000u32 {
        let record = &records[(i % 4) as usize];
        sum += heights(record, 9.81, 6.0e6 + f64::from(i))[0];
    }
    let spent = start.elapsed();
    assert!(sum > 0.0);
    assert!(spent.as_millis() < 200, "{spent:?} for 100,000 planets");
}
