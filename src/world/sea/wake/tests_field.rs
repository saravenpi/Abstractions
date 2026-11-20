//! Tests of the height of the waves of a packet, its pack for the shader and the cost of reading it.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::block::SeaWakes;
use super::packet::Packet;
use super::tune::{FOOT, LIFE, RESOLVE, WAKES};
use super::wakes::{Wake, Wakes};
use crate::render::smoothstep;
use std::f32::consts::TAU;

/// Gravity in metres per second squared of the tests.
const G: f32 = 9.81;

/// Returns the sea coordinates of a frame position: its x and z.
fn locate(at: DVec3) -> Vec2 {
    Vec2::new(at.x as f32, at.z as f32)
}

/// The height of a ring on its own crest is the height of the source, spread and faded as the
/// formula says, and the packet repacked from its rows is the same packet.
#[test]
fn a_ring_crest_has_the_formula_height() {
    let (amp, size, age) = (0.2, 2.0, 0.5);
    let packet = Packet::hover(G, size, (amp, 1.0));
    assert_eq!(Packet::from_rows(packet.rows()), packet);
    let front = packet.front * age;
    let spread = (1.0 + front / size).sqrt().recip();
    let fade = (1.0_f32 - age / LIFE).powi(2);
    let height = packet.bit((front, 0.0), 1.0 + age, 0.0).x;
    assert!((height - amp * spread * fade).abs() < 1e-5, "{height}");
    assert!((packet.front - (G / (TAU / 6.0)).sqrt()).abs() < 1e-5);
}

/// A hovering source rings the sea in every direction alike.
#[test]
fn a_hovering_ring_is_round() {
    let packet = Packet::hover(G, 2.0, (0.2, 0.0));
    let front = packet.front * 0.5;
    let a = packet.bit((front, 0.0), 0.5, 0.0);
    let b = packet.bit((0.0, -front), 0.5, 0.0);
    assert!((a.x - b.x).abs() < 1e-6);
}

/// The block holds the packet where the shader reads it: the height at a sample point is the
/// formula as seen from a footprint of one cell, placed and turned with the entry.
#[test]
fn the_packed_height_matches_the_formula() {
    let (size, age) = (2.0, 0.5);
    let packet = Packet::hover(G, size, (0.2, 1.0));
    let mut wakes = Wakes::default();
    wakes.push(Wake {
        at: DVec3::new(10.0, 0.0, 5.0),
        ahead: DVec3::new(10.0, 0.0, 6.0),
        over: DVec3::new(10.0, 1.0, 5.0),
        packet,
    });
    let now = 1.0 + age;
    let block = SeaWakes::pack(&wakes, now, locate);
    let front = packet.front * age;
    let at = Vec2::new(10.0 - front, 5.0);
    let spread = (1.0 + front / size).sqrt().recip();
    let fade = (1.0_f32 - age / LIFE).powi(2);
    let keep = 1.0 - smoothstep(RESOLVE.0, RESOLVE.1, FOOT * packet.k / TAU);
    let expected = 0.2 * spread * fade * keep;
    assert!((block.height(at, now) - expected).abs() < 1e-5);
}

/// Places far from every entry feel nothing, and a full buffer costs a bounded number of rows.
#[test]
fn reading_the_waves_is_bounded() {
    let mut wakes = Wakes::default();
    for i in 0..20 * WAKES {
        let born = i as f32 * 0.01;
        wakes.push(Wake {
            at: DVec3::new(i as f64, 0.0, 0.0),
            ahead: DVec3::new(i as f64 + 1.0, 0.0, 0.0),
            over: DVec3::new(i as f64, 1.0, 0.0),
            packet: Packet::moving(30.0, G, 2.0, (0.3, born)),
        });
    }
    let now = 20.0 * WAKES as f32 * 0.01;
    assert_eq!(wakes.len(), WAKES);
    let block = SeaWakes::pack(&wakes, now, locate);
    assert!(block.count.x as usize <= WAKES);
    assert_eq!(block.height(Vec2::new(1.0e5, 1.0e5), now), 0.0);
    assert_eq!(block.height(Vec2::new(0.0, 5.0e3), now), 0.0);
    let sum: f32 = (0..400)
        .map(|i| block.height(Vec2::new(i as f32 * 0.5, 4.0), now).abs())
        .sum();
    assert!(sum.is_finite());
}

/// A moving source's ring is kept where it touches the wedge and dies straight behind and
/// straight to the side.
#[test]
fn a_moving_ring_builds_its_arms() {
    let packet = Packet::moving(20.0, G, 2.0, (0.3, 0.0));
    let r = 10.0;
    let meet = std::f32::consts::FRAC_PI_2 + (packet.front / packet.speed).asin();
    let kept = |angle: f32| packet.arc(-r * angle.cos(), r);
    assert!((kept(meet) - 1.0).abs() < 1e-5);
    assert!(kept(0.0) < 0.01);
    assert!(kept(std::f32::consts::FRAC_PI_2 - 0.9) < 0.01);
    assert_eq!(Packet::hover(G, 2.0, (0.3, 0.0)).arc(-r, r), 1.0);
}
