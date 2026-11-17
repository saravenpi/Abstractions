//! Boarding: taking the seat of the landed ship and stepping out beside its hatch.

use bevy::ecs::system::SystemParam;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::ShipEvent;
use super::body::Ship;
use super::sky::Skies;
use crate::player::{EYE_OFFSET, Player, place, seat};
use crate::render::GameCamera;
use crate::settings::{Persistent, Settings};
use crate::space::{Anchor, Location, Placed, Travel};

/// Distance in metres from the hull centre within which the player can board.
const BOARD_RANGE: f32 = 7.0;

/// Where the player steps out in body axes: left of the cockpit, in front of the wing.
const HATCH: DVec3 = DVec3::new(-2.0, 0.0, -1.8);

/// Filter of the player body, which is not the camera.
type OnlyBody = (With<Player>, Without<GameCamera>);

/// Filter of the camera, a child of the player body.
type OnlyEyes = (With<GameCamera>, Without<Player>);

/// What boarding reads and writes besides the bodies.
#[derive(SystemParam)]
pub struct Dock<'w> {
    /// Keyboard state.
    keys: Res<'w, ButtonInput<KeyCode>>,
    /// Bindings.
    settings: Res<'w, Persistent<Settings>>,
    /// The render anchor.
    anchor: Res<'w, Anchor>,
    /// What surrounds the ship.
    skies: Res<'w, Skies>,
    /// Where the player is and how the player travels.
    location: ResMut<'w, Location>,
    /// Where boarding and leaving are announced.
    events: MessageWriter<'w, ShipEvent>,
}

impl Dock<'_> {
    /// Returns the render position of the ground or the sea beside the hatch of a ship with pose `at`.
    fn step_out(&self, at: &Placed) -> Vec3 {
        let hatch = at.pos + at.rot * HATCH;
        let radius = self
            .skies
            .planet()
            .map_or(hatch.length(), |planet| planet.radius);
        let ground = radius + self.skies.height(hatch).max(0.0);
        self.anchor.to_render(hatch.normalize_or(DVec3::Y) * ground)
    }
}

/// Seats the player near the landed ship on the board key, and on the same key puts the player
/// back on the ground beside the hatch.
pub fn board(
    mut dock: Dock,
    mut commands: Commands,
    ship: Single<(&Ship, &Placed)>,
    player: Single<&Transform, OnlyBody>,
    mut eyes: Single<&mut Transform, OnlyEyes>,
) {
    let (ship, placed) = *ship;
    let here = dock.anchor.frame == ship.frame;
    if !dock
        .keys
        .just_pressed(dock.settings.controls.keys.ship.board)
        || !ship.landed
        || !here
    {
        return;
    }
    if dock.location.travel == Travel::InShip {
        let feet = dock.step_out(placed);
        let toward = dock.anchor.rotation_to_render(placed.rot) * Vec3::NEG_Z;
        dock.location.travel = Travel::OnFoot;
        **eyes = Transform::from_xyz(0.0, EYE_OFFSET, 0.0);
        commands.queue(move |world: &mut World| {
            place(world, feet, toward);
            seat(world, false);
        });
        dock.events.write(ShipEvent::Left);
    } else if player
        .translation
        .distance(dock.anchor.to_render(placed.pos))
        < BOARD_RANGE
    {
        dock.location.travel = Travel::InShip;
        commands.queue(|world: &mut World| seat(world, true));
        dock.events.write(ShipEvent::Boarded);
    }
}
