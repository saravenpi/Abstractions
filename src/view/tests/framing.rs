//! Framing: where the body sits on screen, how the views differ, and when the body fades.

use bevy::prelude::*;

use super::world::Boxes;
use super::{FOV, moment, run, third};
use crate::view::tune::{PITCH_LIMIT, RADIANS_PER_PIXEL, ZOOM_RANGE};
use crate::view::{Heading, Kind, Look, Rig, Shown, ViewMode, ViewPrefs};

/// Returns where `point` falls on a 16 by 9 screen seen through `shown`: x and y from -1 to 1,
/// and its depth in front of the camera.
fn on_screen(shown: &Shown, point: Vec3) -> Vec3 {
    let eye = shown.pose.rotation.inverse() * (point - shown.pose.position);
    let half = (0.5 * shown.pose.fov).tan() * -eye.z;
    Vec3::new(eye.x / (half * 16.0 / 9.0), eye.y / half, -eye.z)
}

/// From looking fully down to fully up, the body stays in front of the camera, left of the
/// centre of the screen and in a band below it.
#[test]
fn pitch_curve_keeps_the_body_in_a_band() {
    let world = Boxes::default();
    for step in 0..=40 {
        let pitch = PITCH_LIMIT * (step as f32 / 20.0 - 1.0);
        let mut body = moment(Kind::OnFoot, third());
        let mut rig = Rig::default();
        rig.face(Heading { yaw: 0.7, pitch });
        let shown = run(&mut rig, &mut body, &world, 1);
        let at = on_screen(&shown, body.target.position);
        assert!(
            at.z > 1.0,
            "the body is {} m in front at pitch {pitch}",
            at.z
        );
        assert!(
            at.x < -0.03 && at.x > -0.3,
            "x is {} at pitch {pitch}",
            at.x
        );
        assert!(
            at.y < -0.02 && at.y > -0.45,
            "y is {} at pitch {pitch}",
            at.y
        );
    }
}

/// Looking up brings the camera closer and lower and widens the view; looking down does the opposite.
#[test]
fn pitch_curve_drives_distance_height_and_fov() {
    let view = |pitch: f32| {
        let mut body = moment(Kind::OnFoot, third());
        let mut rig = Rig::default();
        rig.face(Heading { yaw: 0.0, pitch });
        let shown = run(&mut rig, &mut body, &Boxes::default(), 1);
        let pivot = body.target.position + Vec3::Y * 0.8;
        (shown.pose.position.distance(pivot), shown.pose.fov)
    };
    let (up, level, down) = (view(1.0), view(0.0), view(-1.0));
    assert!(up.0 < level.0 - 0.5 && level.0 < down.0 - 0.5);
    assert!(up.1 > level.1 && level.1 > down.1 && (level.1 - FOV).abs() < 1.0e-6);
}

/// First person looks from the eyes with the body hidden; third person looks from behind the
/// right shoulder with the body drawn, and both keep the field of view setting.
#[test]
fn first_and_third_person_on_foot() {
    let world = Boxes::default();
    let mut body = moment(Kind::OnFoot, ViewPrefs::default());
    let first = run(&mut Rig::default(), &mut body, &world, 1);
    assert!(first.pose.position.distance(Vec3::new(0.0, 1.6, -0.16)) < 1.0e-5);
    assert_eq!((first.fade.0, first.pose.fov), (0.0, FOV));
    body.prefs = third();
    let behind = run(&mut Rig::default(), &mut body, &world, 1);
    let at = behind.pose.position;
    assert!(at.z > 3.0 && at.x > 0.3 && at.y > 1.6 && behind.fade.0 == 1.0);
    assert!((behind.pose.rotation * Vec3::NEG_Z).distance(Vec3::NEG_Z) < 1.0e-5);
}

/// The body fades out as an obstacle pushes the third person camera onto it.
#[test]
fn body_fades_when_the_camera_is_close() {
    let cramped = Boxes::floor().with([-4.0, 0.0, 0.7], [4.0, 4.0, 0.9]);
    let mut body = moment(Kind::OnFoot, third());
    let shown = run(&mut Rig::default(), &mut body, &cramped, 30);
    assert!(shown.fade.0 < 0.05 && cramped.clearance(shown.pose.position) > 0.29);
}

/// The two preferences are independent, start in first person and survive a round trip to TOML.
#[test]
fn view_modes_toggle_independently() {
    let mut prefs = ViewPrefs::default();
    assert_eq!(
        (prefs.on_foot, prefs.ship),
        (ViewMode::First, ViewMode::First)
    );
    prefs.toggle(Kind::Flying);
    assert_eq!(
        (prefs.on_foot, prefs.ship),
        (ViewMode::Third, ViewMode::First)
    );
    prefs.toggle(Kind::Ship);
    prefs.toggle(Kind::OnFoot);
    assert_eq!(
        (prefs.on_foot, prefs.ship),
        (ViewMode::First, ViewMode::Third)
    );
    let text = toml::to_string(&prefs).unwrap();
    assert_eq!(toml::from_str::<ViewPrefs>(&text).unwrap(), prefs);
    assert_eq!(
        toml::from_str::<ViewPrefs>("").unwrap(),
        ViewPrefs::default()
    );
}

/// The wheel zooms within the limits, and the mouse settings scale and flip the look.
#[test]
fn zoom_and_mouse_settings_are_respected() {
    let mut prefs = ViewPrefs::default();
    prefs.zoom_by(100.0);
    assert_eq!(prefs.zoom, ZOOM_RANGE.0);
    prefs.zoom_by(-100.0);
    assert_eq!(prefs.zoom, ZOOM_RANGE.1);
    let plain = Look::from_mouse(Vec2::new(10.0, 10.0), 1.0, 1.0, false);
    let tuned = Look::from_mouse(Vec2::new(10.0, 10.0), 1.0, 2.0, true);
    assert!((plain.yaw_delta + 10.0 * RADIANS_PER_PIXEL).abs() < 1.0e-7);
    assert!(
        plain.pitch_delta < 0.0 && (tuned.pitch_delta + 2.0 * plain.pitch_delta).abs() < 1.0e-7
    );
    assert!((tuned.yaw_delta - 2.0 * plain.yaw_delta).abs() < 1.0e-7);
    let mut body = moment(Kind::OnFoot, third());
    let near = run(&mut Rig::default(), &mut body, &Boxes::default(), 1)
        .pose
        .position
        .z;
    body.prefs.zoom = 2.0;
    let far = run(&mut Rig::default(), &mut body, &Boxes::default(), 1)
        .pose
        .position
        .z;
    assert!((far - 2.0 * near).abs() < 1.0e-4);
}
