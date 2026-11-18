//! Tests that a market stall is solid where it is drawn: a body that walks into its counter or one of its
//! posts, or flies onto its awning, is stopped.

use bevy::prelude::*;

use super::brush::Build;
use super::site::Kind;
use super::tests::built;
use super::village::{House, draw_house};
use super::{Terrain, compound};
use crate::world::Biome;
use crate::world::structures::touch::{HALF_BODY, pushed, stage};

/// Height of the centre of a body that stands on the floor of the stall, above that floor.
const STANDING: f32 = HALF_BODY + 0.05;

/// Height of the front edge of the awning above the floor, and how far in front of the door wall it reaches.
const EAVE: (f32, f32) = (2.4, 2.2);

/// How much higher the back edge of the awning is than its front edge, and how far behind the middle of the
/// stall that back edge is.
const AWNING: (f32, f32) = (0.5, -1.9);

/// Draws a market stall alone and returns it with its drawing frame back on the stall.
fn stall(terrain: &Terrain) -> Build<'_> {
    let mut b = built(terrain, Kind::Village, 3);
    draw_house(&mut b, Biome::Meadow, House::Stall);
    b.frame(Vec2::ZERO, 0.0);
    b
}

/// Pushes a body from the stall point `from` along the stall direction `along`, in metres per second, for
/// one second, and returns where it ends in the frame of the stall.
fn walked(b: &Build, from: Vec3, along: Vec3) -> Vec3 {
    let mut app = stage(compound(&b.parts).expect("the stall is solid"));
    let end = pushed(&mut app, b.pt(from), b.dir(along), (60, 60.0));
    let back = Quat::from_rotation_y(-b.yaw);
    back * (end - b.pt(Vec3::ZERO))
}

/// A body that walks from the street into the counter stops in front of it.
#[test]
fn the_counter_stops_a_walker() {
    let t = Terrain::new(1);
    let b = stall(&t);
    let end = walked(&b, Vec3::new(-0.7, STANDING, 4.0), Vec3::NEG_Z * 5.0);
    assert!(end.z > 1.3, "the walker reached z = {}", end.z);
}

/// A body that walks from the street into a front post of the awning stops in front of the post, before the
/// counter behind it.
#[test]
fn a_post_stops_a_walker() {
    let t = Terrain::new(1);
    let b = stall(&t);
    for x in [-2.2, 2.2] {
        let end = walked(&b, Vec3::new(x, STANDING, 4.5), Vec3::NEG_Z * 5.0);
        assert!(end.z > 1.8, "the walker at x = {x} reached z = {}", end.z);
    }
}

/// A body that falls on the awning, or glides down onto it from the street at wingsuit speed, stays above it.
#[test]
fn the_awning_stops_a_flyer() {
    let t = Terrain::new(1);
    let b = stall(&t);
    let fall = walked(&b, Vec3::new(-0.1, 6.0, 0.2), Vec3::NEG_Y * 10.0);
    assert!(
        fall.y - HALF_BODY > EAVE.0 - 0.05,
        "the fall ended at {fall}"
    );
    let glide = walked(&b, Vec3::new(-0.1, 5.5, 8.5), Vec3::new(0.0, -5.5, -22.0));
    let cloth = EAVE.0 + AWNING.0 * (EAVE.1 - glide.z) / (EAVE.1 - AWNING.1);
    let over = glide.y - HALF_BODY - cloth;
    let beside = glide.z < AWNING.1 || glide.z > EAVE.1;
    assert!(
        beside || over > -0.05,
        "the glide ended {over} m over the awning at {glide}"
    );
}
