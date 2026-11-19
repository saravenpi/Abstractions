//! The cockpit in a running game: two ships dressed from the shared look, each showing its
//! own state, with no collider.

use std::time::Duration;

use avian3d::prelude::Collider;
use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use super::parts::{Lever, Stick, feed, work};
use super::tests::levers;
use super::{Cockpit, Dial, LEVER, STICK};
use crate::player::Player;
use crate::ship::body::Ship;
use crate::ship::helm::Helm;
use crate::ship::mesh::{Looks, Skin, dress};
use crate::ship::spawn::Yard;
use crate::ship::telemetry::{ShipTelemetry, WarpState};
use crate::space::{Frame, Location, Travel};
use crate::universe::BodyId;

/// Builds two ships from the shared look: the one the player flies, and a puppet without a
/// flight state that nothing but its cockpit component drives.
fn build(mut commands: Commands, mut yard: Yard) {
    let looks = yard.looks(&mut commands);
    let glow = looks.skins[Skin::Glow as usize].clone();
    let frame = Frame::Surface(BodyId::default());
    let ship = Ship::at_rest(frame, DVec3::ZERO, DQuat::IDENTITY);
    let place = (Transform::default(), Visibility::default());
    dress(&mut commands.spawn((ship, place)), &looks, glow.clone());
    dress(&mut commands.spawn((Puppet, place)), &looks, glow);
    commands.spawn(Player);
}

/// Marks the ship nobody at this machine flies.
#[derive(Component, Debug, Clone, Copy)]
struct Puppet;

/// Returns an app with the two ships and the local player on foot, stepping 60 frames per second.
fn app() -> App {
    let mut app = App::new();
    let on_foot = Location {
        body: BodyId::default(),
        travel: Travel::OnFoot,
    };
    app.add_plugins(MinimalPlugins)
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<StandardMaterial>>()
        .init_resource::<Helm>()
        .init_resource::<ShipTelemetry>()
        .insert_resource(on_foot)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            1.0 / 60.0,
        )))
        .add_systems(Startup, build)
        .add_systems(Update, (feed, work).chain());
    app.update();
    app
}

/// Returns the transform of the part `C` of the ship marked `S`.
fn part<C: Component, S: Component>(app: &mut App) -> Transform {
    let mut ships = app.world_mut().query_filtered::<Entity, With<S>>();
    let ship = ships.single(app.world()).expect("one such ship");
    let mut found = app
        .world_mut()
        .query_filtered::<(&Transform, &ChildOf), With<C>>();
    let mut of = found
        .iter(app.world())
        .filter(|(_, parent)| parent.parent() == ship);
    *of.next().expect("one such part").0
}

/// Returns how many entities carry `C`.
fn count<C: Component>(app: &mut App) -> usize {
    app.world_mut().query::<&C>().iter(app.world()).count()
}

/// A cockpit is one stick, one lever, three instruments and the fixed cabin, and none of it
/// has a collider: the hull box is all the player and the ground ever touch. Two ships share
/// every mesh and every material: nothing is built for the second one.
#[test]
fn the_cockpit_adds_no_collider_and_is_shared() {
    let mut app = app();
    assert_eq!(count::<Collider>(&mut app), 0);
    assert_eq!((count::<Stick>(&mut app), count::<Lever>(&mut app)), (2, 2));
    assert_eq!(
        (count::<Dial>(&mut app), count::<Cockpit>(&mut app)),
        (6, 2)
    );
    assert_eq!(part::<Stick, Ship>(&mut app).translation, STICK);
    assert_eq!(part::<Lever, Puppet>(&mut app).translation, LEVER);
    let drawn = count::<Mesh3d>(&mut app);
    let looks = app.world().resource::<Looks>().parts().len();
    assert_eq!((drawn, looks), (2 * 18, 14));
    assert_eq!(app.world().resource::<Assets<Mesh>>().len(), 14);
    assert_eq!(app.world().resource::<Assets<StandardMaterial>>().len(), 8);
}

/// The stick of the ship the player flies turns with the helm, its lever with its throttle
/// and its instruments with what they measure, none of them steps on the way, and the local
/// player is its pilot only while seated.
#[test]
fn the_parts_move_with_the_ship() {
    let mut app = app();
    let asked = levers([1.0, 0.0, -1.0], 0.8);
    app.world_mut().resource_mut::<Helm>().controls.turn = asked.stick.as_dvec3();
    let mut ships = app.world_mut().query::<&mut Ship>();
    let mut ship = ships.single_mut(app.world_mut()).expect("a ship");
    (ship.throttle, ship.gear) = (0.8, 0.0);
    app.world_mut().resource_mut::<ShipTelemetry>().warp = WarpState::Spooling(0.5);
    let mut last = part::<Stick, Ship>(&mut app).rotation;
    for _ in 0..90 {
        app.update();
        let now = part::<Stick, Ship>(&mut app).rotation;
        assert!(now.angle_between(last) < 0.08, "the stick jumped");
        last = now;
    }
    assert!(last.angle_between(asked.stick_turn()) < 1.0e-3);
    let lever = part::<Lever, Ship>(&mut app).rotation;
    assert!(lever.angle_between(asked.lever_turn()) < 1.0e-3);
    let mut cockpits = app.world_mut().query_filtered::<&Cockpit, With<Ship>>();
    let cockpit = *cockpits.single(app.world()).expect("a cockpit");
    let shown = cockpit.gauges;
    assert!(shown[0] < 1.0e-3 && (shown[1] - 0.5).abs() < 1.0e-3 && shown[2] < 1.0e-3);
    assert!(cockpit.shown.stick.distance(asked.stick) < 1.0e-3 && cockpit.pilot.is_none());
    app.world_mut().resource_mut::<Location>().travel = Travel::InShip;
    app.update();
    let mut players = app.world_mut().query_filtered::<Entity, With<Player>>();
    let player = players.single(app.world()).expect("a player");
    let cockpit = *cockpits.single(app.world()).expect("a cockpit");
    assert_eq!(cockpit.pilot, Some(player));
}

/// A ship nobody at this machine flies shows its own cockpit: its parts follow its own
/// component, whatever the local helm does, and the two ships never show the same thing.
#[test]
fn a_puppet_ship_shows_its_own_cockpit() {
    let mut app = app();
    app.world_mut().resource_mut::<Helm>().controls.turn = DVec3::X;
    let theirs = levers([-1.0, 0.0, 1.0], 1.0);
    let mut puppets = app
        .world_mut()
        .query_filtered::<&mut Cockpit, With<Puppet>>();
    let mut puppet = puppets.single_mut(app.world_mut()).expect("a puppet");
    (puppet.asked, puppet.read) = (theirs, [0.25, 0.5, 1.0]);
    (0..90).for_each(|_| app.update());
    let stick = part::<Stick, Puppet>(&mut app).rotation;
    assert!(stick.angle_between(theirs.stick_turn()) < 1.0e-3);
    let lever = part::<Lever, Puppet>(&mut app).rotation;
    assert!(lever.angle_between(theirs.lever_turn()) < 1.0e-3);
    let mine = part::<Stick, Ship>(&mut app).rotation;
    assert!(mine.angle_between(levers([1.0, 0.0, 0.0], 0.0).stick_turn()) < 1.0e-3);
    let puppet = *puppets.single(app.world()).expect("a puppet");
    assert!((Vec3::from_array(puppet.gauges) - Vec3::new(0.25, 0.5, 1.0)).length() < 1.0e-3);
    let mut dials = app.world_mut().query::<(&Dial, &Transform, &ChildOf)>();
    let mut ships = app.world_mut().query::<&Cockpit>();
    for (dial, transform, parent) in dials.iter(app.world()) {
        let cockpit = ships.get(app.world(), parent.parent()).expect("a cockpit");
        assert_eq!(*transform, dial.place(cockpit.gauges[*dial as usize]));
    }
}
