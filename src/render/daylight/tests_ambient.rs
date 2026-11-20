//! Tests of the ambient light by direction: the sky colour for an up normal, the bounce of the
//! ground for a down one, the twilight arch, and the colour of the ground in the place block.

use bevy::prelude::*;

use super::PlaceUniform;
use super::ambient::{BOUNCE_CAP, bounce, ground, hemisphere, twilight, with_ground};
use crate::render::scatter::luminance;

/// The sky colour comes for a normal up, the bounce for a normal down, their mean across.
#[test]
fn the_hemisphere_is_sky_above_and_ground_below() {
    let sky = Vec3::new(0.2, 0.4, 0.9);
    let ground = Vec3::new(0.3, 0.2, 0.05);
    assert_eq!(hemisphere(sky, ground, 1.0), sky);
    assert_eq!(hemisphere(sky, ground, -1.0), ground);
    let across = hemisphere(sky, ground, 0.0);
    assert!((across - (sky + ground) * 0.5).abs().max_element() < 1.0e-6);
    assert_eq!(hemisphere(sky, ground, 3.0), sky);
    let tilts = [-1.0, -0.5, 0.0, 0.5, 1.0].map(|t| luminance(hemisphere(sky, ground, t)));
    assert!(tilts.windows(2).all(|pair| pair[1] != pair[0]));
}

/// The bounce wears the colour of the ground, grows with the sun and the sky, is nothing over a
/// black ground, and never passes its cap over the sky.
#[test]
fn the_bounce_follows_the_ground() {
    let sky = Vec3::splat(350.0);
    let sun = Vec3::splat(12_000.0);
    let green = Vec3::new(0.1, 0.3, 0.12);
    let noon = bounce(green, sun, 1.0, sky);
    assert!(noon.y > noon.x && noon.y > noon.z);
    assert!(luminance(noon) <= BOUNCE_CAP * luminance(sky) + 1.0e-3);
    let low = bounce(green, sun, 0.1, sky);
    assert!(luminance(low) < luminance(noon));
    assert_eq!(bounce(Vec3::ZERO, sun, 1.0, sky), Vec3::ZERO);
    let night = bounce(green, Vec3::ZERO, -0.5, Vec3::splat(40.0));
    assert!((night - green * 40.0).abs().max_element() < 1.0e-3);
}

/// Under a cap that does not bind, the bounce is the ground colour times the half-sphere light.
#[test]
fn a_dark_ground_is_not_capped() {
    let sky = Vec3::splat(1000.0);
    let dark = Vec3::splat(0.02);
    let given = bounce(dark, Vec3::splat(3000.0), 1.0, sky);
    let level = 3000.0 / std::f32::consts::PI + 1000.0;
    assert!((given.x - 0.02 * level).abs() < 1.0e-3);
}

/// The twilight arch is nothing in day and in deep night and peaks as the star sets.
#[test]
fn the_twilight_arch_peaks_at_sunset() {
    assert_eq!(twilight(0.5), 0.0);
    assert_eq!(twilight(-0.5), 0.0);
    assert!((twilight(-0.02) - 1.0).abs() < 1.0e-6);
    assert!(twilight(-0.1) > 0.0 && twilight(0.06) > 0.0);
    assert!(twilight(-0.1) < twilight(-0.02) && twilight(0.06) < twilight(-0.02));
}

/// The colour of the ground goes into the free slots of the block and comes back, and the slots
/// the rest of the block reads do not move.
#[test]
fn the_ground_colour_rides_in_the_free_slots() {
    let place = PlaceUniform {
        air: Vec4::new(60_000.0, 60.0, 1.0, 0.0),
        eye_sky: Vec4::new(1.0, 2.0, 3.0, 0.0),
        ..default()
    };
    let green = Vec3::new(0.1, 0.3, 0.12);
    let with = with_ground(place, green);
    assert_eq!(ground(&with), green);
    assert_eq!(with.air.truncate(), place.air.truncate());
    assert_eq!(with.eye_sky.truncate(), place.eye_sky.truncate());
    assert_eq!(ground(&place), Vec3::ZERO);
}
