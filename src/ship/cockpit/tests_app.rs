//! The cockpit in a running game: two ships dressed from the shared look, each showing its
//! own state, with no collider, and displays that repaint only when their value changes.

use avian3d::prelude::Collider;
use bevy::prelude::*;

use super::pixels::Ink;
use super::stage::{Puppet, app, bytes, count, edit, lit, part, state};
use super::tests::levers;
use super::{Cockpit, Dial, Display, Lamp, Levers, Part, Readings, Screen};
use crate::ship::body::Ship;
use crate::ship::mesh::Looks;

/// A cockpit is a stick, a lever, a gear lever, a warp switch, two pedals, a strap and three
/// displays, and none of it has a collider: the hull box is all the player
/// and the ground ever touch. Two ships share every mesh and every material of the cockpit; each
/// display has its own image and its own material.
#[test]
fn the_cockpit_adds_no_collider_and_is_shared() {
    let mut app = app();
    assert_eq!(count::<Collider>(&mut app), 0);
    assert_eq!(count::<Cockpit>(&mut app), 2);
    assert_eq!(count::<Part>(&mut app), 2 * 7);
    assert_eq!(
        (count::<Screen>(&mut app), count::<Display>(&mut app)),
        (6, 6)
    );
    assert_eq!(
        part::<Ship>(&mut app, Part::Stick).translation,
        super::STICK
    );
    assert_eq!(
        part::<Puppet>(&mut app, Part::Lever).translation,
        super::LEVER
    );
    let looks = app.world().resource::<Looks>().parts().len();
    assert_eq!(count::<Mesh3d>(&mut app), 2 * 47);
    assert_eq!(looks, 22);
    assert_eq!(app.world().resource::<Assets<Mesh>>().len(), 24);
    assert_eq!(app.world().resource::<Assets<Image>>().len(), 6);
    assert_eq!(app.world().resource::<Assets<StandardMaterial>>().len(), 16);
}

/// Returns an app whose local ship is asked for `asked` and reads half a spool, after 90 frames.
fn settled(asked: Levers) -> App {
    let mut app = app();
    edit::<Ship>(&mut app, |cockpit| {
        cockpit.asked = asked;
        cockpit.read.bars[Dial::Charge as usize] = Some(0.5);
    });
    (0..90).for_each(|_| app.update());
    app
}

/// Returns true when the part `wanted` of the local ship is turned by `want`.
fn turned(app: &mut App, wanted: Part, want: Quat) -> bool {
    part::<Ship>(app, wanted).rotation.angle_between(want) < 1.0e-3
}

/// The stick never steps on its way to what is asked, and the stick, the lever, the gear
/// lever and the warp switch end up turned as the controls say.
#[test]
fn the_levers_turn_with_what_is_asked() {
    let asked = Levers {
        gear: 1.0,
        warp: 0.5,
        ..levers([1.0, 0.5, -1.0], 0.8)
    };
    let mut app = app();
    edit::<Ship>(&mut app, |cockpit| cockpit.asked = asked);
    let mut last = part::<Ship>(&mut app, Part::Stick).rotation;
    for _ in 0..90 {
        app.update();
        let now = part::<Ship>(&mut app, Part::Stick).rotation;
        assert!(now.angle_between(last) < 0.08, "the stick jumped");
        last = now;
    }
    assert!(turned(&mut app, Part::Stick, asked.stick_turn()));
    assert!(turned(&mut app, Part::Lever, asked.lever_turn()));
    assert!(turned(&mut app, Part::Gear, asked.gear_turn()));
    assert!(turned(&mut app, Part::Switch, asked.switch_turn()));
}

/// The pedals turn with the yaw, and a gauge shows what it reads, or is dark when it reads
/// nothing.
#[test]
fn the_pedals_and_the_gauges_follow_the_ship() {
    let asked = levers([0.0, 0.5, 0.0], 0.0);
    let mut app = settled(asked);
    assert!(turned(&mut app, Part::Pedal(0), asked.pedal_turn(0)));
    assert!(turned(&mut app, Part::Pedal(1), asked.pedal_turn(1)));
    let cockpit = state::<Ship>(&mut app);
    assert!((cockpit.gauges[Dial::Charge as usize].expect("lit") - 0.5).abs() < 1.0e-3);
    assert_eq!(cockpit.gauges[Dial::Altitude as usize], None);
}

/// A display is painted again only when the value it shows moves by a step: a steady value,
/// or one that moves less than half a percent, leaves the picture alone, and a value that goes
/// away darkens it.
#[test]
fn a_display_repaints_only_when_its_value_changes() {
    let mut app = app();
    let show = |speed: Option<f64>| move |cockpit: &mut Cockpit| cockpit.read.speed = speed;
    edit::<Ship>(&mut app, show(Some(1500.0)));
    app.update();
    let first = bytes::<Ship>(&mut app, Screen::Flight);
    assert!(lit(&first));
    let mut images = app.world_mut().resource_mut::<Assets<Image>>();
    let handles: Vec<_> = images.iter().map(|(id, _)| id).collect();
    for id in handles {
        if let Some(mut image) = images.get_mut(id) {
            image.data = image.data.as_ref().map(|data| vec![7; data.len()]);
        }
    }
    edit::<Ship>(&mut app, show(Some(1500.0)));
    app.update();
    edit::<Ship>(&mut app, show(Some(1500.5)));
    app.update();
    assert!(
        bytes::<Ship>(&mut app, Screen::Flight)
            .iter()
            .all(|byte| *byte == 7)
    );
    edit::<Ship>(&mut app, show(Some(1700.0)));
    app.update();
    let moved = bytes::<Ship>(&mut app, Screen::Flight);
    assert!(lit(&moved) && moved != first);
    edit::<Ship>(&mut app, show(None));
    app.update();
    assert!(!lit(&bytes::<Ship>(&mut app, Screen::Flight)));
    assert!(!lit(&bytes::<Puppet>(&mut app, Screen::Flight)));
}

/// A ship nobody at this machine flies shows its own cockpit: its parts follow its own
/// component, whatever the other ship does, and the two ships never show the same thing.
#[test]
fn a_puppet_ship_shows_its_own_cockpit() {
    let mut app = app();
    let theirs = Levers {
        gear: 1.0,
        ..levers([-1.0, 0.0, 1.0], 1.0)
    };
    edit::<Puppet>(&mut app, |cockpit| {
        cockpit.asked = theirs;
        cockpit.read = Readings {
            bars: [Some(0.25), Some(1.0), None, Some(0.5), Some(1.0)],
            ..default()
        };
        cockpit.read.lamps[Lamp::Hot as usize] = true;
    });
    edit::<Ship>(&mut app, |cockpit| {
        cockpit.asked = levers([1.0, 0.0, 0.0], 0.0)
    });
    (0..90).for_each(|_| app.update());
    let stick = part::<Puppet>(&mut app, Part::Stick).rotation;
    assert!(stick.angle_between(theirs.stick_turn()) < 1.0e-3);
    let lever = part::<Puppet>(&mut app, Part::Lever).rotation;
    assert!(lever.angle_between(theirs.lever_turn()) < 1.0e-3);
    let mine = part::<Ship>(&mut app, Part::Stick).rotation;
    assert!(mine.angle_between(levers([1.0, 0.0, 0.0], 0.0).stick_turn()) < 1.0e-3);
    let cockpit = state::<Puppet>(&mut app);
    assert!((cockpit.gauges[0].expect("lit") - 0.25).abs() < 1.0e-3 && cockpit.gauges[2].is_none());
    let amber = |bytes: Vec<u8>| bytes.chunks(4).any(|pixel| pixel == Ink::Amber.rgba());
    assert!(amber(bytes::<Puppet>(&mut app, Screen::Status)));
    assert!(!amber(bytes::<Ship>(&mut app, Screen::Status)));
}
