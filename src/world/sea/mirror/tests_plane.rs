//! Tests of the water plane: the mirror reflects points and the camera about it, its clip plane
//! keeps what lies above the water, the Fresnel share runs from 2 percent to all of it, and a
//! camera sees water unless it looks up.

use bevy::prelude::*;

use super::plane::{Plane, WATER_F0, fresnel, sees_water};

/// A plane tilted off every axis, as the sea is away from the pole of the render axes.
fn tilted() -> Plane {
    Plane {
        point: Vec3::new(3.0, -1.5, 7.0),
        up: Vec3::new(0.3, 0.9, -0.2).normalize(),
    }
}

/// A point mirrors to the same height under the plane, along its normal, and back again; the
/// matrix does what the point rule does and turns space inside out.
#[test]
fn the_mirror_reflects_a_point_about_the_water() {
    let level = Plane {
        point: Vec3::new(0.0, 2.0, 0.0),
        up: Vec3::Y,
    };
    assert_eq!(
        level.mirror(Vec3::new(1.0, 5.0, 3.0)),
        Vec3::new(1.0, -1.0, 3.0)
    );
    assert_eq!(level.height(Vec3::new(9.0, 5.0, -4.0)), 3.0);
    let plane = tilted();
    for p in [
        Vec3::new(1.0, 4.0, -2.0),
        Vec3::new(-30.0, 0.5, 12.0),
        plane.point,
    ] {
        let image = plane.mirror(p);
        assert!((plane.height(image) + plane.height(p)).abs() < 1e-4, "{p}");
        assert!(plane.height((p + image) * 0.5).abs() < 1e-4, "{p}");
        assert!((p - image).cross(plane.up).length() < 1e-4, "{p}");
        assert!((plane.mirror(image) - p).length() < 1e-4, "{p}");
        assert!(
            (plane.matrix().transform_point3(p) - image).length() < 1e-4,
            "{p}"
        );
    }
    assert!((plane.matrix().determinant() + 1.0).abs() < 1e-4);
}

/// The mirror camera stands where the eye's reflection stands and looks along the reflection of
/// its view.
#[test]
fn the_mirror_camera_is_the_eye_reflected() {
    let plane = tilted();
    let eye = GlobalTransform::from(
        Transform::from_xyz(5.0, 6.0, 2.0).looking_at(Vec3::new(-20.0, -3.0, -40.0), Vec3::Y),
    );
    let mirror = GlobalTransform::from(plane.eye(&eye));
    assert!((mirror.translation() - plane.mirror(eye.translation())).length() < 1e-3);
    let ahead = eye.translation() + eye.forward() * 10.0;
    let seen = mirror.translation() + mirror.forward() * 10.0;
    assert!((seen - plane.mirror(ahead)).length() < 1e-3);
}

/// In the view of the mirror camera the clip plane passes through the water, keeps what stands
/// above it and cuts what lies under it, such as the legs of a wading robot.
#[test]
fn the_clip_plane_keeps_what_is_above_the_water() {
    let plane = tilted();
    let eye = GlobalTransform::from(
        Transform::from_translation(plane.point + plane.up * 3.0 + Vec3::X)
            .looking_at(plane.point + Vec3::new(-10.0, 0.0, -10.0), Vec3::Y),
    );
    let clip = plane.clip(&eye);
    assert!((clip.truncate().length() - 1.0).abs() < 1e-4);
    assert!((clip.w + plane.height(eye.translation())).abs() < 1e-4);
    let view = plane.eye(&eye).inverse();
    let side = |world: Vec3| clip.truncate().dot(view.transform_point3(world)) + clip.w;
    let foot = plane.point + Vec3::new(-6.0, 0.0, -8.0);
    let on = foot - plane.up * plane.height(foot);
    assert!(side(on).abs() < 1e-3);
    assert!(side(on + plane.up * 1.5) > 1.0);
    assert!(side(on - plane.up * 0.5) < -0.4);
}

/// The water mirrors 2 percent of the light looking straight down, at 0 degrees from the normal,
/// all of it at a grazing 90 degrees, and more the lower the view.
#[test]
fn fresnel_runs_from_straight_down_to_grazing() {
    assert!((fresnel(1.0) - WATER_F0).abs() < 1e-6);
    assert!((fresnel(0.0) - 1.0).abs() < 1e-6);
    let angles = [0.0_f32, 30.0, 60.0, 80.0, 89.0];
    let shares: Vec<f32> = angles
        .iter()
        .map(|a| fresnel(a.to_radians().cos()))
        .collect();
    assert!(
        shares.windows(2).all(|pair| pair[0] < pair[1]),
        "{shares:?}"
    );
    assert!(shares[1] < 0.03 && shares[3] > 0.3);
}

/// A camera level with the horizon, or looking a little up, sees water at the bottom of its
/// view; one looking far up does not.
#[test]
fn a_camera_sees_water_unless_it_looks_up() {
    let (fov, aspect) = (75.0_f32.to_radians(), 16.0 / 9.0);
    let toward = |pitch: f32| Vec3::new(0.0, pitch.to_radians().sin(), -pitch.to_radians().cos());
    assert!(sees_water(toward(-80.0), Vec3::Y, fov, aspect));
    assert!(sees_water(toward(0.0), Vec3::Y, fov, aspect));
    assert!(sees_water(toward(20.0), Vec3::Y, fov, aspect));
    assert!(!sees_water(toward(70.0), Vec3::Y, fov, aspect));
    assert!(!sees_water(Vec3::Y, Vec3::Y, fov, aspect));
}
