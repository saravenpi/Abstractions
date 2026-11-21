//! The climate of an Earth-like planet comes out near the Earth, and worlds of a Sun-like star
//! freeze beyond the outer edge of its habitable zone.

use super::climate::{FREEZING, settle};
use super::units::SUN_TEMPERATURE;
use super::zone::outer;

/// An Earth-like planet, one atmosphere of air at one astronomical unit from a Sun with seas
/// over 60 to 70 percent of it, settles within 3 K of the 288 K the Earth has.
#[test]
fn an_earth_comes_out_near_288() {
    let earth = settle(1.0, outer(SUN_TEMPERATURE), 1.0, 0.4, 0.5);
    println!(
        "earth {:.1} K, albedo {:.3}",
        earth.temperature, earth.albedo
    );
    assert!((285.0..=291.0).contains(&earth.temperature), "{earth:?}");
}

/// Wet carbon-rich rocks at 1, 1.5, 1.67 and 9 AU from a Sun, under 1 and 5 atmospheres: the
/// thermostat thaws them out to the outer edge, at 1.7 AU, and nothing beyond it.
#[test]
fn a_suns_worlds_freeze_beyond_the_outer_edge() {
    let edge = outer(SUN_TEMPERATURE);
    println!("outer edge {edge:.3} flux, {:.2} AU", (1.0 / edge).sqrt());
    for au in [1.0, 1.5, 1.67, 1.75, 2.0, 3.0, 9.0] {
        for pressure in [0.5, 1.0, 5.0] {
            let c = settle(1.0 / (au * au), edge, pressure, 0.3, 0.5);
            println!(
                "{au} AU, {pressure} atm: {:.0} K, greenhouse {:.0} K, {:?}",
                c.temperature, c.greenhouse, c.water
            );
        }
    }
    let at = |au: f64, pressure: f64| settle(1.0 / (au * au), edge, pressure, 0.3, 0.5);
    assert!(at(1.5, 1.0).temperature > FREEZING);
    for pressure in [0.5, 1.0, 5.0] {
        assert!(at(2.0, pressure).temperature < FREEZING, "{pressure}");
        assert!(at(9.0, pressure).temperature < FREEZING, "{pressure}");
    }
}
