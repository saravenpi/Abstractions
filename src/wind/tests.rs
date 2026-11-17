//! Tests of the wind: its range and its direction across a rebase.

use std::f64::consts::PI as PI64;
use std::sync::Arc;

use bevy::math::{DQuat, DVec3};

use super::*;
use crate::space::rebased;
use crate::universe::Universe;
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
