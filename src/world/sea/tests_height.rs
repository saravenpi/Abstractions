//! Tests of the height above the sea the water shader measures from the focus: on a planet the size of
//! the Earth it stays within a millimetre of the height worked out in 64 bits.

use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::height::Sight;
use crate::world::Sited;
use crate::world::sea::follow::placing;
use crate::world::sea::uniform::WaterUniform;

const SHADER: &str = include_str!("water.wesl");

/// Radius in metres of the planet the tests stand on.
const RADIUS: f64 = 6.4e6;

/// A seat on the sea of a planet of [`RADIUS`], its render pose, and the values of a focus standing at
/// `local` in its frame.
fn sighted(local: Vec3) -> (Sited, Transform, WaterUniform) {
    let up = DVec3::new(0.48, 0.6, -0.64).normalize();
    let sited = Sited {
        pos: up * RADIUS,
        rot: DQuat::from_rotation_arc(DVec3::Y, up),
    };
    let pose = Transform::from_xyz(12.0, -3.0, 40.0).with_rotation(Quat::from_rotation_z(0.3));
    let at = sited.pos + sited.rot * local.as_dvec3();
    let eye = pose.transform_point(local);
    let water = placing(
        WaterUniform::default(),
        &pose,
        RADIUS as f32,
        Vec2::ZERO,
        Vec2::ZERO,
    );
    let water = Sight::of(&pose, &sited, (eye, at)).onto(water, RADIUS);
    (sited, pose, water)
}

/// Around a focus on the shore and one gliding 300 m up, every point within 200 m reads its height
/// above the sea within a millimetre, and the focus reads its own.
#[test]
fn the_sea_height_errs_under_a_millimetre_at_earth_size() {
    for focus in [Vec3::new(30.0, 1.7, -40.0), Vec3::new(-250.0, 300.0, 90.0)] {
        let (sited, pose, water) = sighted(focus);
        let unit = |n: u32| (n.wrapping_mul(2_654_435_761) >> 8) as f32 / (1u32 << 24) as f32;
        for i in 0..3000 {
            let spread = Vec3::new(unit(3 * i), unit(3 * i + 1), unit(3 * i + 2)) * 2.0 - 1.0;
            let local = focus + spread * 200.0;
            let truth = (sited.pos + sited.rot * local.as_dvec3()).length() - RADIUS;
            let read = f64::from(water.sea_height(pose.transform_point(local)));
            assert!(
                (read - truth).abs() < 1.0e-3,
                "{local}: {read} against {truth}"
            );
        }
        let own = (sited.pos + sited.rot * focus.as_dvec3()).length() - RADIUS;
        assert!((f64::from(water.eye.w) - own).abs() < 1.0e-3);
    }
}

/// The shader holds the same formula and reads both the height of the camera and the sag of the sea
/// under a far point through it, never from the distance to the planet centre.
#[test]
fn the_shader_measures_as_the_uniform_does() {
    for line in [
        "let p = world - water.eye.xyz;",
        "let rise = dot(p, p) + 2.0 * (radius + water.eye.w) * dot(p, water.upward.xyz) + water.upward.w;",
        "let eye_height = sea_height(view.world_position);",
        "let sag = max(-sea_height(world), 0.0);",
    ] {
        assert!(SHADER.contains(line), "{line}");
    }
    assert!(!SHADER.contains("- water.globe.w"));
}
