//! Tests of what the class of a planet does below its sea level and to its clouds seen from
//! space: a dry planet is never submerged, molten rock glows in its own colour, and the home
//! planet keeps its blue-green sea and its white clouds.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::air::Air;
use super::fog::UNDERWATER;
use super::murk::{clearance, murk, under};
use super::palette::Palette;
use super::sky::{SkyLook, SkyUniform, linear};
use super::vantage::{LocalSystem, Nearby, Vantage};
use crate::universe::{PlanetClass, PlanetRecord, Universe};
use crate::world::Globe;

/// Returns the home record with another class and another share of its surface under liquid.
fn world(class: PlanetClass, ocean: f64) -> PlanetRecord {
    PlanetRecord {
        class,
        ocean,
        ..PlanetRecord::TEMPERATE
    }
}

/// Returns the sky of a camera `altitude` metres above the sea level of the home planet of
/// seed 7 when that planet has this record.
fn standing(record: PlanetRecord, altitude: f32) -> (Vantage, LocalSystem) {
    let universe = Universe::new(7);
    let body = universe.home();
    let mut local = Nearby::read(&universe, body.star);
    local.system.planets[usize::from(body.index)].record = record;
    let vantage = Vantage {
        altitude,
        body: usize::from(body.index),
        ..Vantage::default()
    };
    (vantage, LocalSystem(Some(local)))
}

/// Returns the dome values at noon for a camera with `murk` around it.
fn dome(murk: Option<Vec3>) -> SkyUniform {
    let look = SkyLook {
        up: Vec3::Y,
        dip: 0.0,
        sun_height: 1.0,
        halo: 1.0,
        air: 1.0,
        whiteout: 0.0,
        rim: Vec3::splat(0.2),
        to_galaxy: Mat3::IDENTITY,
        mapped: true,
        murk,
        heading: Vec3::NEG_Z,
        beta: 0.0,
    };
    SkyUniform::new(&Palette::lit(1.0, Vec3::ONE), &look)
}

/// On a planet without a sea nothing is submerged, whatever its class and however low the
/// camera stands: no liquid colour, and the dome keeps its sky, its sun and its rim.
#[test]
fn a_dry_planet_is_never_submerged() {
    for class in PlanetClass::ALL {
        let dry = world(class, 0.0);
        assert_eq!(murk(&dry), None, "{class:?}");
        for altitude in [-400.0, -0.5, 0.0, 30.0] {
            let (vantage, local) = standing(dry, altitude);
            assert_eq!(under(&local, &vantage), None, "{class:?} at {altitude}");
        }
    }
    let (sky, noon) = (dome(None), Palette::lit(1.0, Vec3::ONE));
    assert_eq!(
        (sky.zenith, sky.horizon),
        (linear(noon.zenith), linear(noon.horizon))
    );
    assert!(sky.sun.w > 0.9 && sky.rim.x > 0.0);
}

/// The home sea, and every sea of water, is the old blue-green below its surface and nothing
/// above it; outside a world the sea is the home sea.
#[test]
fn water_keeps_the_home_colour() {
    assert_eq!(murk(&PlanetRecord::TEMPERATE), Some(UNDERWATER));
    for class in [PlanetClass::Ocean, PlanetClass::Island, PlanetClass::Ice] {
        let (vantage, local) = standing(world(class, 0.4), -2.0);
        assert_eq!(under(&local, &vantage), Some(UNDERWATER), "{class:?}");
        let (above, local) = standing(world(class, 0.4), 0.0);
        assert_eq!(under(&local, &above), None, "{class:?}");
    }
    let nowhere = Vantage {
        altitude: -1.0,
        ..Vantage::default()
    };
    assert_eq!(under(&LocalSystem::default(), &nowhere), Some(UNDERWATER));
    let sky = dome(Some(UNDERWATER));
    assert_eq!(
        (sky.zenith, sky.horizon),
        (linear(UNDERWATER), linear(UNDERWATER))
    );
    assert_eq!((sky.sun.w, sky.rim, sky.galaxy.x), (0.0, Vec4::ZERO, 0.0));
}

/// Under a sea of molten rock the fog and the dome are the glowing colour of the liquid: red
/// above all, brighter than white can be, and nothing like the blue-green of water.
#[test]
fn molten_rock_glows_in_its_own_colour() {
    let lava = world(PlanetClass::Lava, 0.3);
    let look = PlanetClass::Lava.look();
    let (vantage, local) = standing(lava, -3.0);
    let seen = under(&local, &vantage).expect("a lava sea has a colour");
    assert_eq!(Some(seen), murk(&lava));
    assert!(seen != UNDERWATER && seen.x > 1.0 && seen.x > 1.5 * seen.y && seen.y > seen.z);
    let lit = linear(seen).truncate();
    assert!(
        (lit - look.sea * (0.5 * look.glow)).length() < 1e-3,
        "{lit}"
    );
    let sky = dome(Some(seen));
    assert_eq!((sky.zenith, sky.sun.w), (linear(seen), 0.0));
}

/// Over the low ground of a dry planet the camera is as high as the ground is far, while a
/// sea holds it at its surface; over high ground both agree.
#[test]
fn low_ground_of_a_dry_planet_counts_down_to_where_it_is() {
    let globe =
        |ocean: f64| Globe::new(3, 3000.0, 9.8).with_record(world(PlanetClass::Desert, ocean));
    let (dry, wet) = (globe(0.0), globe(0.6));
    let dirs = (0..400).map(|i| {
        let turn = f64::from(i) * 2.399_963;
        let y = 1.0 - f64::from(i) / 200.0;
        DVec3::new(turn.cos(), y, turn.sin()).normalize()
    });
    let (mut low, mut high) = (0, 0);
    for dir in dirs {
        let (ground, sea) = (dry.height(dir), wet.height(dir));
        assert_eq!(clearance(&dry, dir, 50.0), 50.0 - ground);
        assert_eq!(clearance(&wet, dir, 50.0), 50.0 - sea.max(0.0));
        low += usize::from(ground < 0.0);
        high += usize::from(ground > 0.0);
    }
    assert!(low > 0 && high > 0, "{low} low and {high} high spots");
}

/// Seen from space the clouds of the home planet stay white, and the clouds of a planet with
/// coloured air take its tint, the same tint as its clouds seen from below at noon.
#[test]
fn clouds_from_space_take_the_tint_of_the_air() {
    assert_eq!(Air::HOME.clouds_from_space(), Vec3::ONE);
    assert_eq!(
        Air::of(&PlanetRecord::TEMPERATE).clouds_from_space(),
        Vec3::ONE
    );
    let home = Palette::through(1.0, Vec3::ONE, &Air::HOME).cloud;
    for class in PlanetClass::ALL {
        let air = Air::of(&world(class, 0.3));
        let (space, tint) = (air.clouds_from_space(), class.look().air);
        assert_eq!(space, linear(tint).truncate(), "{class:?}");
        let below = Palette::through(1.0, Vec3::ONE, &air).cloud / home;
        assert!((below - tint).length() < 1e-5, "{class:?}: {below}");
        assert!(space.max_element() <= 1.0 && space.min_element() > 0.05);
    }
    let desert = Air::of(&world(PlanetClass::Desert, 0.0)).clouds_from_space();
    assert!(desert.x > desert.y && desert.y > desert.z, "{desert}");
}
