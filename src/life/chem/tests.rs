//! Tests of the pigment curve and the chemistry roll.

use super::*;

/// Returns a world with home air at `temperature` kelvin under a star at `star` kelvin.
fn world(temperature: f32, star: f32) -> World {
    World {
        gravity: 22.0,
        pressure: 1.0,
        temperature,
        flux: 1.0,
        radius: 6000.0,
        ocean: 0.6,
        star,
    }
}

/// Returns the mean of the three channels of a colour.
fn value(c: [f32; 3]) -> f32 {
    (c[0] + c[1] + c[2]) / 3.0
}

/// A yellow star gives green, a red dwarf dark red, a cool dwarf near black and a hot star pale blue white.
#[test]
fn pigment_follows_the_star() {
    let home = pigment(5800.0);
    assert!(home[1] > home[0] && home[1] > home[2]);
    let red = pigment(3000.0);
    assert!(red[0] > red[1] && red[0] > red[2] && value(red) < value(home));
    assert!(value(pigment(2200.0)) < 0.05);
    let blue = pigment(9000.0);
    assert!(blue[2] >= blue[0] && blue.iter().all(|c| *c > 0.5));
}

/// The pigment never jumps: 10 K apart always gives colours within 0.01.
#[test]
fn pigment_is_smooth() {
    for k in (1500..15_000).step_by(10) {
        let (a, b) = (pigment(k as f32), pigment(k as f32 + 10.0));
        assert!((0..3).all(|i| (a[i] - b[i]).abs() < 0.01), "jump at {k} K");
    }
}

/// The solvent follows the temperature, and nothing fits between the liquids.
#[test]
fn solvent_follows_the_temperature() {
    let at = |t| solvent(&world(t, 5800.0)).map(|(s, _)| s);
    assert_eq!(at(300.0), Some(Solvent::Water));
    assert_eq!(at(220.0), Some(Solvent::Ammonia));
    assert_eq!(at(140.0), Some(Solvent::Hydrocarbon));
    assert_eq!(at(2000.0), Some(Solvent::Silicate));
    assert_eq!(at(800.0), None);
}

/// The same seed gives the same chemistry, and a lava world lives on heat with silica tissues.
#[test]
fn chemistry_is_seeded() {
    let home = world(288.0, 5800.0);
    assert_eq!(chemistry(&home, 7), chemistry(&home, 7));
    let lava = chemistry(&world(1800.0, 5800.0), 7);
    assert_eq!(
        (lava.energy, lava.structure),
        (Energy::Heat, Structure::Silica)
    );
}

/// Life in water on starlight fills the air with oxygen and takes the carbon dioxide out; life
/// in another liquid, or on another energy, leaves the air it found.
#[test]
fn only_water_and_light_make_oxygen() {
    use crate::universe::{Gas, Mix};
    let born = Mix::of([
        (Gas::Nitrogen, 0.9),
        (Gas::CarbonDioxide, 0.09),
        (Gas::Argon, 0.01),
        (Gas::Argon, 0.0),
    ]);
    let green = Chemistry {
        solvent: Solvent::Water,
        energy: Energy::Light,
        ..chemistry(&world(288.0, 5800.0), 3)
    };
    let alive = air(&green, born);
    assert!(alive.share(Gas::Oxygen) > 0.2 && alive.share(Gas::CarbonDioxide) == 0.0);
    assert_eq!(alive.main_gas(), Gas::Nitrogen);
    let other = [
        Chemistry {
            solvent: Solvent::Hydrocarbon,
            ..green
        },
        Chemistry {
            energy: Energy::Chemistry,
            ..green
        },
    ];
    assert!(other.iter().all(|chemistry| air(chemistry, born) == born));
}
