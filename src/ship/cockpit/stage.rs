//! The stage of the cockpit tests in a running game: two ships dressed from the shared look,
//! one the player flies and one a puppet, and the ways to read and change them.

use std::time::Duration;

use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use super::display::{equip, fitted, repaint};
use super::work::work;
use super::{Cockpit, Display, Part, Screen};
use crate::ship::body::Ship;
use crate::ship::mesh::{Skin, dress};
use crate::ship::spawn::Yard;
use crate::space::Frame;
use crate::universe::BodyId;

/// Builds two ships from the shared look: the one the player flies, and a puppet without a
/// flight state that nothing but its cockpit component drives.
pub(super) fn build(mut commands: Commands, mut yard: Yard) {
    let looks = yard.looks(&mut commands);
    let glow = looks.skins[Skin::Glow as usize].clone();
    let frame = Frame::Surface(BodyId::default());
    let ship = Ship::at_rest(frame, DVec3::ZERO, DQuat::IDENTITY);
    let place = (Transform::default(), Visibility::default());
    dress(&mut commands.spawn((ship, place)), &looks, glow.clone());
    dress(&mut commands.spawn((Puppet, place)), &looks, glow);
}

/// Marks the ship nobody at this machine flies.
#[derive(Component, Debug, Clone, Copy)]
pub(super) struct Puppet;

/// Returns an app with the two ships, stepping 60 frames per second.
pub(super) fn app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<Image>>()
        .init_resource::<Assets<StandardMaterial>>()
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            1.0 / 60.0,
        )))
        .add_systems(Startup, build)
        .add_systems(Update, (equip.run_if(fitted), work, repaint).chain());
    app.update();
    app
}

/// Changes the cockpit of the ship marked `S`.
pub(super) fn edit<S: Component>(app: &mut App, change: impl FnOnce(&mut Cockpit)) {
    let mut found = app.world_mut().query_filtered::<&mut Cockpit, With<S>>();
    let mut cockpit = found.single_mut(app.world_mut()).expect("one such ship");
    change(&mut cockpit);
}

/// Returns the state of the cockpit of the ship marked `S`.
pub(super) fn state<S: Component>(app: &mut App) -> Cockpit {
    let mut found = app.world_mut().query_filtered::<&Cockpit, With<S>>();
    *found.single(app.world()).expect("one such ship")
}

/// Returns the transform of `wanted` on the ship marked `S`.
pub(super) fn part<S: Component>(app: &mut App, wanted: Part) -> Transform {
    let mut ships = app.world_mut().query_filtered::<Entity, With<S>>();
    let ship = ships.single(app.world()).expect("one such ship");
    let mut found = app.world_mut().query::<(&Part, &Transform, &ChildOf)>();
    let mut of = found
        .iter(app.world())
        .filter(|(part, _, parent)| **part == wanted && parent.parent() == ship);
    *of.next().expect("one such part").1
}

/// Returns how many entities carry `C`.
pub(super) fn count<C: Component>(app: &mut App) -> usize {
    app.world_mut().query::<&C>().iter(app.world()).count()
}

/// Returns the bytes painted on the display `wanted` of the ship marked `S`.
pub(super) fn bytes<S: Component>(app: &mut App, wanted: Screen) -> Vec<u8> {
    let mut found = app.world_mut().query::<(&Screen, &Display, &ChildOf)>();
    let mut ships = app.world_mut().query_filtered::<Entity, With<S>>();
    let ship = ships.single(app.world()).expect("one such ship");
    let image = found
        .iter(app.world())
        .find(|(screen, _, parent)| **screen == wanted && parent.parent() == ship)
        .map(|(_, display, _)| display.image.clone())
        .expect("one such display");
    let images = app.world().resource::<Assets<Image>>();
    images
        .get(&image)
        .and_then(|i| i.data.clone())
        .expect("a picture")
}

/// Returns true when some pixel of the picture `bytes` is lit.
pub(super) fn lit(bytes: &[u8]) -> bool {
    bytes.chunks(4).any(|pixel| pixel[0] == 255)
}
