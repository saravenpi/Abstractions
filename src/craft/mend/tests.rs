//! Tests of mending: the grammar of a field patch, the cap on it and the bay of the base.

use bevy::prelude::*;

use super::base::inside;
use super::field::{patch, supply};
use super::tune::{BAY, PATCH_CAP};
use crate::condition::{Condition, Part};
use crate::craft::matter::Class;
use crate::craft::pack::Pack;
use crate::craft::stage::{pack, tissue};
use crate::home::Base;
use crate::space::{Anchor, Frame};
use crate::universe::BodyId;

/// A joint takes a binding and a plate takes something hard; wood and hide patch no plate.
#[test]
fn joints_take_bindings_and_plates_take_hardness() {
    let carried = pack();
    let (bind, _) = supply(&carried, true).expect("hide binds");
    let (plate, _) = supply(&carried, false).expect("bone or stone is hard");
    assert_ne!(bind, plate);
    let mut soft = Pack::default();
    soft.stow(tissue(Class::Wood, 0, 2.0), f32::INFINITY);
    assert!(supply(&soft, true).is_none() && supply(&soft, false).is_none());
}

/// A patch spends material, mends the worst part it can supply and stops at the cap.
#[test]
fn a_patch_spends_material_and_stops_at_the_cap() {
    let mut carried = pack();
    let mass = carried.mass();
    let mut body = Condition::default();
    body.hurt(Part::LegL, 1.0e6, true);
    let before = body.of(Part::LegL);
    assert_eq!(patch(&mut body, &mut carried), Some(Part::LegL));
    assert!(body.of(Part::LegL) > before && carried.mass() < mass);
    for _ in 0..10 {
        patch(&mut body, &mut carried);
    }
    assert!((body.of(Part::LegL) - PATCH_CAP).abs() < 1e-4);
}

/// With nothing to patch with, a hurt body stays hurt.
#[test]
fn no_material_no_patch() {
    let mut body = Condition::default();
    body.hurt(Part::Head, 100.0, true);
    assert_eq!(patch(&mut body, &mut Pack::default()), None);
}

/// The bay is a circle around the base on its own planet.
#[test]
fn the_bay_is_around_the_base() {
    let base = Base {
        body: BodyId::default(),
        centre: bevy::math::DVec3::ZERO,
        up: bevy::math::DVec3::Y,
        east: bevy::math::DVec3::X,
        north: bevy::math::DVec3::NEG_Z,
        wake: bevy::math::DVec3::ZERO,
        pad: bevy::math::DVec3::ZERO,
        heading: bevy::math::DVec3::X,
        door: bevy::math::DVec3::ZERO,
    };
    let anchor = Anchor {
        frame: Frame::Surface(base.body),
        origin: bevy::math::DVec3::ZERO,
        rotation: bevy::math::DQuat::IDENTITY,
        flat: false,
    };
    assert!(inside(&anchor, &base, Vec3::new(BAY - 1.0, 0.0, 0.0)));
    assert!(!inside(&anchor, &base, Vec3::new(BAY + 1.0, 0.0, 0.0)));
}
