//! Tests of the air: the absorption matches the table of ISO 9613-1, carbon dioxide at low
//! pressure takes the highs within metres, and no air is a vacuum.

use super::air::{Absorb, Air};
use super::iso::{Loss, Mixture};
use crate::universe::{Gas, Mix, PlanetRecord, Water};

/// Returns the absorption of the air of home at 20 degrees and half humidity, the row the
/// standard is usually quoted at.
fn standard() -> Loss {
    let record = PlanetRecord {
        ocean: 0.4,
        ..PlanetRecord::TEMPERATE
    };
    Loss::of(&Mixture::of(&record, (1.0, 293.15), 343.0))
}

/// At 20 degrees and 50 percent the air takes about 4.7 dB a kilometre at 1 kHz, 10 at 2 kHz
/// and a hundred at 8 kHz, as the table of the standard says, and more the higher the tone.
#[test]
fn the_air_of_home_follows_the_standard() {
    let loss = standard();
    let per_km = |hz: f64| loss.at(hz) * 1000.0;
    assert!((4.0..5.5).contains(&per_km(1000.0)), "{}", per_km(1000.0));
    assert!((8.5..11.5).contains(&per_km(2000.0)), "{}", per_km(2000.0));
    assert!(
        (85.0..130.0).contains(&per_km(8000.0)),
        "{}",
        per_km(8000.0)
    );
    let tones = [125.0, 250.0, 500.0, 1000.0, 2000.0, 4000.0, 8000.0].map(per_km);
    assert!(tones.windows(2).all(|pair| pair[1] > pair[0]));
    let corner = loss.corner(1000.0);
    assert!((loss.at(corner) * 1000.0 - 3.0).abs() < 0.01, "{corner}");
}

/// Thin cold carbon dioxide, the air of a world like Mars, takes a decibel a metre at 1 kHz.
#[test]
fn thin_carbon_dioxide_takes_the_highs_within_metres() {
    let mars = PlanetRecord {
        pressure: 0.006,
        temperature: 220.0,
        water: Water::None,
        ocean: 0.0,
        mix: Mix::of([
            (Gas::CarbonDioxide, 0.95),
            (Gas::Nitrogen, 0.03),
            (Gas::Argon, 0.02),
            (Gas::Oxygen, 0.0),
        ]),
        ..PlanetRecord::TEMPERATE
    };
    let loss = Loss::of(&Mixture::of(&mars, (0.006, 220.0), 230.0));
    assert!(loss.at(1000.0) > 0.5, "{} dB/m", loss.at(1000.0));
    assert!(loss.corner(10.0) < 500.0 && loss.corner(10.0) > 50.0);
}

/// A world with too little air is a vacuum, and no absorption leaves every corner open.
#[test]
fn no_air_is_a_vacuum() {
    let bare = PlanetRecord {
        pressure: 0.0,
        ..PlanetRecord::TEMPERATE
    };
    assert_eq!(Air::sea(&bare), Air::VACUUM);
    assert!(Absorb::NONE.corner(1.0e4) > 1.0e5);
    let home = Air::home();
    assert!(home.absorb.corner(10.0) > 10_000.0 && home.absorb.corner(1000.0) < 1000.0);
}
