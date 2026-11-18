//! What a gas giant sounds like: a deep wind that never dies down, and no ground or shore.

use std::sync::Arc;

use super::{CLICK, Mixer, RATE, Shared, Stats, listen};
use crate::audio::planet::{GIANT_WIND, Surf, surf, wind};
use crate::audio::scene::AudioScene;
use crate::settings::Audio as Volumes;
use crate::universe::{PlanetClass, PlanetRecord, Water};

/// Returns the record of a gas giant with deep air and, for the sake of the test, a sea.
fn giant() -> PlanetRecord {
    PlanetRecord {
        class: PlanetClass::GasGiant,
        pressure: 5.0,
        water: Water::Liquid,
        ..PlanetRecord::TEMPERATE
    }
}

/// Renders twenty seconds of a strong wind on a planet of `kind`, with the music off.
fn gale(kind: PlanetClass) -> Stats {
    let scene = AudioScene {
        class: kind,
        wind: 0.9,
        ..AudioScene::default()
    };
    let shared = Arc::new(Shared::default());
    let quiet = Volumes {
        music: 0.0,
        ..Volumes::default()
    };
    shared.publish(&scene, &quiet);
    let mut mixer = Mixer::new(RATE, shared);
    let stats = listen(&mut mixer, 20.0);
    assert!(!stats.broken && stats.peak <= 1.0 && stats.jump < CLICK);
    stats
}

/// The wind of a gas giant is heard, lower than the same wind at home: far smaller steps from
/// one sample to the next for its level.
#[test]
fn a_gas_giant_wind_roars_low() {
    let (home, deep) = (gale(PlanetClass::Island), gale(PlanetClass::GasGiant));
    assert!(home.rms() > 0.005 && deep.rms() > 0.005);
    let rough = |stats: &Stats| f64::from(stats.jump) / stats.rms();
    assert!(
        rough(&deep) < 0.5 * rough(&home),
        "{} {}",
        rough(&deep),
        rough(&home)
    );
}

/// The wind of a gas giant never drops under its floor, any other air keeps the wind it has,
/// no air has none, and a gas giant has no shore whatever its record says.
#[test]
fn a_gas_giant_is_never_still_and_has_no_shore() {
    assert_eq!(wind(&giant(), 0.0), GIANT_WIND);
    assert_eq!(wind(&giant(), 0.95), 0.95);
    assert_eq!(wind(&PlanetRecord::TEMPERATE, 0.3), 0.3);
    let airless = PlanetRecord {
        pressure: 0.0,
        ..giant()
    };
    assert_eq!(wind(&airless, 0.5), 0.0);
    assert_eq!(surf(&giant()), Surf::Silent);
    assert_eq!(surf(&PlanetRecord::TEMPERATE), Surf::Waves);
}
