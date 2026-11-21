//! Planting: what the ground does to a thrown javelin that meets it. A point that leads, arriving
//! steep and fast enough on soft ground, drives in as deep as its energy carries it and the shaft
//! stands at the angle it came in at; anything else glances off and comes to lie on the ground.

use bevy::prelude::*;

use super::super::flight::{Spear, depth, glance};
use super::super::tune::STICK;
use super::ground::{Ground, Soil};
use super::tune::{DEEPEST, LEADS, LIE, STEEP};

/// What becomes of a javelin that meets the ground.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Meet {
    /// It stands in the ground, its tip buried this many metres along its shaft.
    Plant(f32),
    /// It glances off with this velocity.
    Glance(Vec3),
}

/// Returns what ground of unit `normal` and `soil` does to `spear` arriving with its shaft along the
/// unit `axis`: the work of the point along the shaft over the pressure the soil holds drives it in.
pub fn meet(spear: &Spear, axis: Vec3, (normal, soil): (Vec3, Soil)) -> Meet {
    let speed = spear.velocity.length();
    let way = spear.velocity / speed.max(f32::EPSILON);
    let along = axis.dot(way);
    let leads = spear.item.throw.stable && along >= LEADS;
    let steep = -way.dot(normal) >= STEEP;
    let energy = 0.5 * spear.item.throw.mass * speed * speed * along.max(0.0);
    let deep = match soil {
        Soil::Soft(pressure) if leads && steep => {
            depth(energy, spear.item.build.head.area(), pressure)
        }
        _ => 0.0,
    };
    match deep >= STICK {
        true => Meet::Plant(deep.min(DEEPEST)),
        false => Meet::Glance(glance(spear.velocity, normal)),
    }
}

/// Returns where a javelin of `length` metres that came to rest with its tip at `tip` and its shaft
/// along `axis` lies on `ground`: the tip and the butt each on the ground, the shaft between them,
/// as a place whose +Y runs from the butt to the tip.
pub fn lie(ground: &impl Ground, tip: Vec3, axis: Vec3, length: f32) -> Transform {
    let normal = ground.normal(tip);
    let tip = ground.lift(tip) + normal * LIE;
    let flat = axis
        .reject_from(normal)
        .normalize_or(normal.any_orthonormal_vector());
    let butt = tip - flat * length;
    let butt = ground.lift(butt) + ground.normal(butt) * LIE;
    let shaft = (tip - butt).normalize_or(flat);
    Transform::from_translation(tip).with_rotation(Quat::from_rotation_arc(Vec3::Y, shaft))
}

/// Returns where a javelin planted `deep` metres along its shaft `axis` stands, from the point
/// `at` where it met the ground: the tip buried, the shaft at the angle it came in at.
pub fn planted(at: Vec3, axis: Vec3, deep: f32) -> Transform {
    Transform::from_translation(at + axis * deep)
        .with_rotation(Quat::from_rotation_arc(Vec3::Y, axis))
}
