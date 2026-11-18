//! Tests of the wind: its range, its direction across a rebase and how it follows the air.

use std::f64::consts::PI as PI64;
use std::sync::Arc;

use bevy::math::{DQuat, DVec3};

use super::*;
use crate::space::rebased;
use crate::universe::{PlanetRecord, Universe};
use crate::world::globe::Face;
use crate::world::{Globe, Terrain};

/// The wind direction stays a unit vector and the strength stays in range.
#[test]
fn wind_stays_in_range() {
    for step in 0..500 {
        let wind = wind_at(step as f32 * 7.3);
        assert!((wind.direction.length() - 1.0).abs() < 1e-4);
        assert!((0.0..=1.0).contains(&wind.strength));
    }
}

/// Returns the compass angle of render direction `wind` at render position `at`: its angle
/// from the planet east in the tangent plane there.
fn compass(anchor: &Anchor, at: Vec3, wind: Vec2) -> f64 {
    let up = anchor.to_frame(at).normalize();
    let blown = anchor.rotation.inverse() * Vec3::new(wind.x, 0.0, wind.y).as_dvec3();
    let east = DVec3::Y.cross(up).normalize();
    blown.dot(up.cross(east)).atan2(blown.dot(east))
}

/// Across a rebase, and across an anchor whose axes were also twisted around the vertical,
/// the wind keeps its compass direction where the player stands within 1e-3 rad.
#[test]
fn wind_is_continuous_across_a_rebase() {
    let globe = Arc::new(Globe::new(4, 3000.0, 22.0));
    let view = Terrain::on(globe.clone(), globe.face_chart(Face::PosX)).around(40.0, -25.0);
    let stand = Vec3::new(40.0, 0.0, -25.0);
    let old = Anchor::surface(Universe::new(4).home(), &view, stand);
    let walked = stand + Vec3::new(28.0, 0.0, 21.0);
    let new = rebased(&old, 3000.0, walked).expect("a rebase past the limit");
    let twisted = Anchor {
        rotation: DQuat::from_rotation_y(0.8) * new.rotation,
        ..new
    };
    for next in [new, twisted] {
        let blowing = wind_at(77.0).direction;
        let mut veer = Veer::default();
        let before = compass(&old, walked, veer.track(&old, blowing));
        let at = next.to_render(old.to_frame(walked));
        let after = compass(&next, at, veer.track(&next, blowing));
        let change = (after - before + PI64).rem_euclid(2.0 * PI64) - PI64;
        assert!(change.abs() < 1e-3, "{before} -> {after}");
        assert_eq!(veer.track(&next, blowing), veer.track(&next, blowing));
    }
}

/// Returns the home record with another air pressure.
fn air(pressure: f64) -> PlanetRecord {
    PlanetRecord {
        pressure,
        ..PlanetRecord::TEMPERATE
    }
}

/// The thickness is the pressure: nothing under the limit, exactly 1 at home, held at the
/// thickest, never falling as the pressure grows, and nothing for a pressure that is no number.
#[test]
fn thickness_follows_the_pressure() {
    let seen = PRESSURES.map(|pressure| thickness(&air(pressure)));
    assert_eq!(seen, [0.0, 0.0, 0.3, 1.0, 3.0]);
    assert_eq!(thickness(&air(40.0)), THICKEST);
    assert_eq!(thickness(&air(NO_AIR)), NO_AIR);
    assert_eq!(thickness(&air(f64::NAN)), 0.0);
    assert_eq!(thickness(&air(-2.0)), 0.0);
    assert!(seen.windows(2).all(|pair| pair[0] <= pair[1]));
    let classes = samples().map(|record| thickness(&record) > 0.0);
    assert_eq!(classes, [true, true, true, true, true, false]);
}

/// In home air the wind is what the clock gives, bit for bit. Without air it is still, and in
/// between it never falls as the air thickens. Its direction is never touched.
#[test]
fn the_wind_blows_as_hard_as_the_air_carries_it() {
    for step in 0..500 {
        let clock = wind_at(step as f32 * 7.3);
        let blown = PRESSURES.map(|pressure| carried(clock, thickness(&air(pressure)) as f32));
        assert_eq!(blown[3], clock);
        assert_eq!((blown[0].strength, blown[1].strength), (0.0, 0.0));
        assert!(
            blown
                .windows(2)
                .all(|pair| pair[0].strength <= pair[1].strength)
        );
        assert!(blown.iter().all(|wind| wind.direction == clock.direction));
        assert!(
            blown
                .iter()
                .all(|wind| (0.0..=1.0).contains(&wind.strength))
        );
    }
    let gust = Wind {
        strength: 0.5,
        ..wind_at(0.0)
    };
    assert!(carried(gust, 0.3).strength < 0.5 && carried(gust, 3.0).strength > 0.5);
    assert_eq!(carried(gust, f32::NAN).strength, 0.0);
}

/// What drifts follows the air: at home the wind velocity is the clock direction at 1 m/s plus
/// 5 m/s at full strength, bit for bit, in thin air it is slower, in thick air it keeps the
/// standing breeze, and without air it is exactly zero, so smoke, flames and leaves stay still.
#[test]
fn nothing_drifts_without_air() {
    for step in 0..500 {
        let clock = wind_at(step as f32 * 7.3);
        let flat = Vec3::new(clock.direction.x, 0.0, clock.direction.y);
        assert_eq!(clock.velocity(), flat * (1.0 + 5.0 * clock.strength));
        let blown = PRESSURES.map(|pressure| carried(clock, thickness(&air(pressure)) as f32));
        assert_eq!(blown[3].velocity(), clock.velocity());
        assert_eq!(
            (blown[0].velocity(), blown[1].velocity()),
            (Vec3::ZERO, Vec3::ZERO)
        );
        assert!(blown[2].velocity().length() < clock.velocity().length());
        assert!(blown[2].velocity().length() > 0.5 && blown[4].air == 1.0);
        assert!(blown[4].velocity().length() >= clock.velocity().length());
    }
}

/// Without a planet under the anchor the air is the home air, and the wind system leaves the
/// clock wind as it is.
#[test]
fn the_wind_system_keeps_the_home_wind() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins(plugin);
    app.update();
    app.update();
    let seconds = app.world().resource::<Time>().elapsed_secs();
    assert_eq!(*app.world().resource::<Thickness>(), Thickness(1.0));
    assert_eq!(*app.world().resource::<Wind>(), wind_at(seconds));
    app.world_mut().insert_resource(Thickness(0.0));
    app.world_mut().run_system_cached(blow).unwrap();
    assert_eq!(app.world().resource::<Wind>().strength, 0.0);
}
