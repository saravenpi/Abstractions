//! Tests of the aerial perspective: the clear home air shows tens of kilometres, takes blue
//! first, thins with height, and closes in under weather and thick air.

use bevy::prelude::*;

use super::tune::{AEROSOL, AEROSOL_SCALE, HOME_SCALE};
use super::{CONTRAST, Haze, mean, sight, through};
use crate::render::air::Air;
use crate::render::scatter::DEPTH;
use crate::render::weather::WeatherLook;
use crate::universe::TOP_SCALES;

/// Height in metres where the air of the Earth ends: twelve scale heights of 8 km.
const TOP: f32 = HOME_SCALE * TOP_SCALES as f32;

/// Returns the haze of the home air on a planet like the Earth under the weather `look`.
fn home(look: &WeatherLook) -> Haze {
    Haze::of(&Air::HOME, TOP, look)
}

/// The home air on a planet like the Earth is the constant, its gas is the Rayleigh scattering
/// measured on the Earth within a sixth, and a clear day shows between 40 and 70 km in green.
#[test]
fn clear_home_air_shows_tens_of_kilometres() {
    let clear = home(&WeatherLook::CALM);
    assert!((clear.gas - Haze::HOME.gas).abs().max_element() < 1.0e-9);
    assert_eq!((clear.aerosol, clear.scale), (AEROSOL, HOME_SCALE));
    let measured = Vec3::new(5.802e-6, 13.558e-6, 33.1e-6);
    assert!(((clear.gas - measured) / measured).abs().max_element() < 0.16);
    let seen = CONTRAST / clear.toward(0.0, 0.0).y;
    assert!((40_000.0..70_000.0).contains(&seen), "{seen}");
    assert!(sight(clear.toward(0.0, 0.0)) > seen);
}

/// The air takes blue before green before red, so a far ridge loses its blue to the haze first,
/// and nothing is taken at no distance.
#[test]
fn the_air_takes_blue_first() {
    let air = home(&WeatherLook::CALM).toward(2.0, 0.0);
    assert!(air.z > air.y && air.y > air.x && air.x > 0.0);
    assert_eq!(through(air, 0.0), Vec3::ONE);
    let (near, far) = (through(air, 5_000.0), through(air, 30_000.0));
    assert!(near.y > 0.6 && near.y < 0.85, "{near}");
    assert!(far.y > 0.05 && far.y < 0.2 && far.z < far.x, "{far}");
}

/// The mean density of a path is 1 along the sea, falls as either end climbs, is the same both
/// ways, and from far above leaves a vertical path the depth of the whole layer: the optical depth
/// that reddens a low star, plus the aerosols.
#[test]
fn the_air_thins_along_a_climbing_path() {
    assert_eq!(mean(0.0, 0.0, 8000.0), 1.0);
    assert_eq!(mean(-50.0, 0.0, 8000.0), 1.0);
    assert!((mean(8000.0, 8000.0, 8000.0) - (-1.0f32).exp()).abs() < 1.0e-6);
    assert_eq!(mean(3000.0, 0.0, 8000.0), mean(0.0, 3000.0, 8000.0));
    let means: Vec<f32> = (0..40)
        .map(|k| mean(k as f32 * 1000.0, 0.0, 8000.0))
        .collect();
    assert!(means.windows(2).all(|pair| pair[1] < pair[0]), "{means:?}");
    let depth = mean(400_000.0, 0.0, 8000.0) * 400_000.0;
    assert!((depth - 8000.0).abs() < 1.0, "{depth}");
    let down = home(&WeatherLook::CALM).toward(400_000.0, 0.0) * 400_000.0;
    let whole = DEPTH + AEROSOL * (HOME_SCALE * AEROSOL_SCALE);
    assert!((down - whole).abs().max_element() < 0.005, "{down}");
}

/// Rain closes the sight in to a few kilometres and dust to under two.
#[test]
fn weather_moves_the_sight() {
    let seen = |haze: Haze| CONTRAST / haze.toward(2.0, 0.0).y;
    let clear = seen(home(&WeatherLook::CALM));
    let rain = seen(home(&WeatherLook {
        haze: 1.0,
        dust: 0.0,
    }));
    let dust = seen(home(&WeatherLook {
        haze: 0.0,
        dust: 1.0,
    }));
    assert!(rain < 5_000.0 && rain > 1_500.0, "{rain}");
    assert!(
        dust < 1_500.0 && dust > 300.0 && clear > 10.0 * rain,
        "{dust}"
    );
}

/// Thick air closes the sight in, thin air opens it, and without air nothing is taken and the
/// sight has no end.
#[test]
fn pressure_moves_the_sight() {
    let seen = |haze: Haze| CONTRAST / haze.toward(2.0, 0.0).y;
    let of = |pressure: f32| Air {
        pressure,
        ..Air::HOME
    };
    let sights: Vec<f32> = [5.0, 2.0, 1.0, 0.3, 0.05]
        .map(|pressure| seen(Haze::of(&of(pressure), TOP, &WeatherLook::CALM)))
        .to_vec();
    assert!(
        sights.windows(2).all(|pair| pair[1] > pair[0]),
        "{sights:?}"
    );
    let none = Haze::of(&of(0.0), TOP, &WeatherLook::CALM);
    assert_eq!(none.toward(2.0, 0.0), Vec3::ZERO);
    assert_eq!(sight(none.toward(2.0, 0.0)), f32::INFINITY);
    assert_eq!(
        Haze::of(&Air::HOME, 0.0, &WeatherLook::CALM).toward(2.0, 0.0),
        Vec3::ZERO
    );
}
