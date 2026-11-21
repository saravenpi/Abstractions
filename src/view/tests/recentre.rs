//! Bringing the view in the ship back to the nose: by itself with auto-follow on, and with the
//! recentre key whatever the setting.

use bevy::prelude::*;

use super::stage::{parked, ship};
use super::world::Boxes;
use super::{moment, run, seen, third};
use crate::view::{Heading, Kind, Look, Rig, ViewMode, ViewPrefs};

/// With auto-follow on the view springs back to the nose once the mouse has rested, behind
/// the ship and in the pilot seat, as it always did.
#[test]
fn with_auto_follow_on_the_view_springs_back() {
    let world = Boxes::default();
    for view in [ViewMode::Third, ViewMode::First] {
        let (mut rig, mut ship) = (Rig::default(), ship(true));
        ship.prefs.ship = view;
        ship.look.yaw_delta = 0.03;
        run(&mut rig, &mut ship, &world, 30);
        ship.look = Look::default();
        let held = run(&mut rig, &mut ship, &world, 60);
        assert!((held.heading.yaw - 0.9).abs() < 1.0e-4, "{view:?}");
        let back = run(&mut rig, &mut ship, &world, 400);
        assert!(
            back.heading.yaw.abs() < 1.0e-3,
            "{view:?}: {:?}",
            back.heading
        );
    }
}

/// Low over a planet at speed, the chase camera turns toward where a slipping ship goes with
/// auto-follow on, and keeps to the nose with it off.
#[test]
fn only_auto_follow_turns_the_chase_toward_the_path() {
    let nose = |follow: bool| {
        let mut ship = ship(follow);
        ship.target.ground = Vec3::Y;
        ship.target.velocity = Vec3::new(120.0, 0.0, -120.0);
        let shown = run(&mut Rig::default(), &mut ship, &Boxes::default(), 240);
        (shown.pose.rotation * Vec3::NEG_Z).x
    };
    assert!(nose(true) > 0.2);
    assert!(nose(false).abs() < 1.0e-3, "{}", nose(false));
}

/// With auto-follow off, the recentre key brings a parked chase view back behind the nose:
/// the look goes there on the frame of the press and the camera glides after it in a moment.
#[test]
fn the_recentre_key_brings_the_chase_back_behind_the_nose() {
    let (mut rig, mut ship, _) = parked(Heading {
        yaw: 2.6,
        pitch: -0.4,
    });
    let world = Boxes::default();
    ship.look.recentre = true;
    let pressed = run(&mut rig, &mut ship, &world, 1);
    ship.look.recentre = false;
    assert_eq!(pressed.heading, Heading::default());
    let at = seen(&pressed, &ship.target).0;
    assert!(at.x.abs() > 0.5, "the camera jumped behind the nose: {at}");
    let behind = seen(&run(&mut rig, &mut ship, &world, 30), &ship.target).0;
    assert!(behind.x.abs() < 0.05 && behind.z > 0.0, "{behind}");
    let later = run(&mut rig, &mut ship, &world, 300);
    assert_eq!(later.heading, Heading::default());
}

/// In the pilot seat the recentre key turns the head to the nose at once.
#[test]
fn the_recentre_key_turns_the_pilot_to_the_nose() {
    let (mut rig, mut ship, _) = parked(Heading {
        yaw: 1.2,
        pitch: 0.3,
    });
    let world = Boxes::default();
    ship.prefs.ship = ViewMode::First;
    run(&mut rig, &mut ship, &world, 60);
    ship.look.recentre = true;
    let pressed = run(&mut rig, &mut ship, &world, 1);
    let ahead = seen(&pressed, &ship.target).1 * Vec3::NEG_Z;
    assert_eq!(pressed.heading, Heading::default());
    assert!(ahead.distance(Vec3::NEG_Z) < 1.0e-4, "{ahead}");
}

/// On foot the recentre key changes nothing: the heading there is that of the body.
#[test]
fn the_recentre_key_does_nothing_on_foot() {
    let prefs = ViewPrefs {
        auto_follow: false,
        ..third()
    };
    let (mut rig, mut body) = (Rig::default(), moment(Kind::OnFoot, prefs));
    body.look.yaw_delta = 0.02;
    let turned = run(&mut rig, &mut body, &Boxes::default(), 50);
    body.look = Look {
        recentre: true,
        ..Look::default()
    };
    let pressed = run(&mut rig, &mut body, &Boxes::default(), 1);
    assert_eq!(pressed.heading, turned.heading);
}
