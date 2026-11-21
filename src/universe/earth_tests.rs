//! The climate of an Earth-like planet comes out near the Earth.

use super::climate::settle;

/// An Earth-like planet, one atmosphere of air at one astronomical unit from a Sun with seas
/// over 60 to 70 percent of it, settles within 3 K of the 288 K the Earth has.
#[test]
fn an_earth_comes_out_near_288() {
    let earth = settle(1.0, 1.0, 0.4, 0.5);
    println!(
        "earth {:.1} K, albedo {:.3}",
        earth.temperature, earth.albedo
    );
    assert!((285.0..=291.0).contains(&earth.temperature), "{earth:?}");
}
