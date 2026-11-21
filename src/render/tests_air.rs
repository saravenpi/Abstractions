//! Air tests: what the record of a planet makes of its sky share, its clouds, its halo and its
//! fog, and that the home record leaves all of them as they were.

use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::aerial::Haze;
use super::air::Air;
use super::fog::{cull_distance, extinction};
use super::observer::Observer;
use super::vantage::{LocalSystem, Nearby, Vantage};
use super::weather::WeatherLook;
use crate::universe::{PlanetClass, PlanetRecord, Universe};

/// Returns the home record with another class and pressure.
fn world(class: PlanetClass, pressure: f64) -> PlanetRecord {
    PlanetRecord {
        class,
        pressure,
        ..PlanetRecord::TEMPERATE
    }
}

/// Returns the sky a camera at sea level gets when the home planet of seed 7 has this record.
fn at_sea_level(record: PlanetRecord) -> (Vantage, LocalSystem) {
    let universe = Universe::new(7);
    let body = universe.home();
    let mut local = Nearby::read(&universe, body.star);
    let index = usize::from(body.index);
    local.system.planets[index].record = record;
    let planet = local.system.planets[index].clone();
    let center = planet.position(0.0);
    let observer = Observer {
        eye: center + DVec3::X * planet.radius,
        center,
        spin: DQuat::IDENTITY,
        to_render: DQuat::IDENTITY,
    };
    let vantage = Vantage::of(&observer, &local, &planet, Vec3::ZERO);
    (vantage, LocalSystem(Some(local)))
}

/// The home record gives the home air, and the home air changes no number: a whole sky, one
/// atmosphere of depth, the colour it is given, the cloud cover of the day and the old halo.
#[test]
fn home_record_is_the_home_air() {
    let home = Air::of(&PlanetRecord::TEMPERATE);
    assert_eq!(home, Air::HOME);
    assert_eq!((home.share(), home.depth(), home.haze()), (1.0, 1.0, 1.0));
    for base in [Vec3::new(0.18, 0.4, 0.92), Vec3::ZERO, Vec3::splat(0.7)] {
        assert_eq!(home.stain(base, Vec3::new(1.0, 0.5, 0.3)), base);
    }
    assert_eq!(home.clouds, 0.5);
    assert_eq!(home.halo(Vec3::ONE), Vec4::new(0.32, 0.55, 1.0, 1.0));
    let (vantage, local) = at_sea_level(PlanetRecord::TEMPERATE);
    assert_eq!(vantage.air, 1.0);
    assert_eq!(local.air(vantage.body), Air::HOME);
    assert_eq!(LocalSystem::default().air(3), Air::HOME);
}

/// The share of sky rises with the pressure from nothing to the home sky and stays there; the
/// depth stays at one up to the home pressure, then rises and levels off under 2.5.
#[test]
fn share_and_depth_follow_the_pressure() {
    let air = |pressure: f64| Air::of(&world(PlanetClass::Island, pressure));
    assert_eq!((air(0.0).share(), air(0.009).share()), (0.0, 0.0));
    assert_eq!(air(-3.0).share(), 0.0);
    let steps: Vec<Air> = (1..=100).map(|i| air(f64::from(i) / 100.0)).collect();
    for pair in steps.windows(2) {
        assert!(pair[1].share() > pair[0].share() && pair[0].depth() == 1.0);
    }
    assert!((air(0.4).share() - 0.63).abs() < 0.02);
    let thick: Vec<Air> = (0..=40).map(|i| air(1.0 + f64::from(i) / 10.0)).collect();
    for pair in thick.windows(2) {
        assert!(pair[1].depth() > pair[0].depth() && pair[1].share() == 1.0);
    }
    assert!(air(5.0).depth() > 2.2 && air(500.0).depth() <= 2.5);
}

/// The camera at sea level has as much air above it as the planet has, and the planet below
/// is the one whose air the sky systems read.
#[test]
fn vantage_air_is_the_share_of_the_planet() {
    for class in PlanetClass::ALL {
        for pressure in [0.0, 0.2, 0.4, 1.0, 5.0] {
            let record = world(class, pressure);
            let (vantage, local) = at_sea_level(record);
            let air = local.air(vantage.body);
            assert_eq!(air, Air::of(&record), "{class:?} {pressure}");
            assert_eq!(vantage.air, air.share(), "{class:?} {pressure}");
        }
    }
}

/// The mean cloud share of a class is the home share on an island world, none on a barren one,
/// and it rises with the share of the class.
#[test]
fn cloud_cover_follows_the_class() {
    let cover = |class: PlanetClass| Air::of(&world(class, 1.0)).clouds;
    assert_eq!(cover(PlanetClass::Island), 0.5);
    assert_eq!(cover(PlanetClass::Barren), 0.0);
    assert_eq!(Air::of(&world(PlanetClass::Barren, 1.0)).clouds, 0.0);
    let ranked = [
        PlanetClass::Barren,
        PlanetClass::Desert,
        PlanetClass::Lava,
        PlanetClass::Ice,
        PlanetClass::Island,
        PlanetClass::Ocean,
    ];
    for pair in ranked.windows(2) {
        assert!(cover(pair[1]) > cover(pair[0]), "{pair:?}");
    }
    assert!((cover(PlanetClass::Ocean) - 0.7).abs() < 1e-5);
}

/// A planet without air has no halo. The halo grows with the pressure without blowing out,
/// takes the tint of the air and the colour of the star, and is the old blue at home.
#[test]
fn halo_follows_the_air() {
    let halo = |class: PlanetClass, pressure: f64| Air::of(&world(class, pressure)).halo(Vec3::ONE);
    for class in PlanetClass::ALL {
        assert_eq!(halo(class, 0.0).w, 0.0, "{class:?}");
        assert_eq!(halo(class, 0.005).w, 0.0, "{class:?}");
    }
    let strengths: Vec<f32> = [0.02, 0.1, 0.4, 1.0, 2.0, 5.0]
        .iter()
        .map(|&pressure| halo(PlanetClass::Island, pressure).w)
        .collect();
    assert!(strengths.windows(2).all(|pair| pair[1] > pair[0]));
    assert!(strengths[3] == 1.0 && strengths[5] < 1.6);
    let home = halo(PlanetClass::Island, 1.0);
    let desert = halo(PlanetClass::Desert, 1.0);
    assert!(home.z > 2.0 * home.x && desert.x > desert.y && desert.y > desert.z);
    let lava = halo(PlanetClass::Lava, 1.0);
    assert!(lava.x > 1.6 * lava.y && lava.y > lava.z);
    let red = Air::HOME.halo(Vec3::new(1.0, 0.5, 0.3));
    assert!(red.z / red.x < 0.5 * home.z / home.x && red.max_element() <= 1.0);
}

/// The view on foot reaches tens of kilometres at the home pressure, farther as the air thins,
/// to the reach of the world without air, and closes in under thick air without closing shut.
#[test]
fn fog_follows_the_pressure() {
    let sight = |pressure: f64| {
        let (vantage, local) = at_sea_level(world(PlanetClass::Desert, pressure));
        let air = local.air(vantage.body);
        let haze = Haze::of(&air, vantage.atmosphere, &WeatherLook::CALM);
        cull_distance(extinction(&haze, (2.0, 2.0), 0.0), vantage.radius)
    };
    let sights: Vec<f32> = [5.0, 2.0, 1.0, 0.6, 0.3, 0.1, 0.03, 0.0]
        .iter()
        .map(|&pressure| sight(pressure))
        .collect();
    assert!(
        sights.windows(2).all(|pair| pair[1] > pair[0]),
        "{sights:?}"
    );
    assert!((30_000.0..120_000.0).contains(&sights[2]), "{sights:?}");
    assert!(sights[0] > 10_000.0 && sights[7] > 1.0e8);
}
