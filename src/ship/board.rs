//! Boarding: taking the seat of the landed ship and stepping out beside its hatch.

use avian3d::prelude::ColliderDisabled;
use bevy::ecs::system::SystemParam;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::ShipEvent;
use super::body::Ship;
use super::deck::grounded;
use super::sky::Skies;
use crate::computer::Computer;
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
    /// Whether the player stands at the hatch.
    hatch: ResMut<'w, Hatch>,
    /// Whether the board computer is open.
    computer: Option<Res<'w, State<Computer>>>,
}

impl Dock<'_> {
    /// Returns true while the board computer is open and owns the keyboard.
    fn busy(&self) -> bool {
        self.computer
            .as_deref()
            .is_some_and(|state| *state.get() == Computer::Open)
    }

    /// Returns the render position of the ground or the sea beside the hatch of a ship with pose
    /// `at`. On a dry planet it is the ground, however low.
    fn step_out(&self, at: &Placed) -> Vec3 {
        let hatch = at.pos + at.rot * HATCH;
        let radius = self
            .skies
            .planet()
            .map_or(hatch.length(), |planet| planet.radius);
        let ground = radius + self.skies.height(hatch).max(self.skies.sea());
        self.anchor.to_render(hatch.normalize_or(DVec3::Y) * ground)
    }
}

/// Returns true when the player on foot stands close enough to the landed ship to board it.
fn in_reach(dock: &Dock, ship: &Ship, placed: &Placed, body: Vec3) -> bool {
    let here = dock.anchor.frame == ship.frame;
    let near = body.distance(dock.anchor.to_render(placed.pos)) < BOARD_RANGE;
    ship.landed && here && near && dock.location.travel == Travel::OnFoot
}

/// Whether the player on foot stands at the hatch of the landed ship, where the board key
/// works. The ship refreshes it every frame of play; anything may read it, for a prompt.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Hatch {
    /// True while the board key would seat the player.
    pub near: bool,
}

/// Run condition: the player on foot stands at the hatch of the landed ship.
pub fn near_hatch(hatch: Option<Res<Hatch>>) -> bool {
    hatch.is_some_and(|hatch| hatch.near)
}

/// Switches the collider of a seated player body off when it is still on, as after a load.
pub fn reseat(
    mut commands: Commands,
    bodies: Query<Entity, (With<Player>, Without<ColliderDisabled>)>,
) {
    if !bodies.is_empty() {
        commands.queue(|world: &mut World| seat(world, true));
    }
}

/// Seats the player near the landed ship on the board key, and on the same key puts the player
/// back on the ground beside the hatch. The key does nothing while the board computer is open,
/// and nothing over a gas giant, where there is no ground to step out on.
pub fn board(
    mut dock: Dock,
    mut commands: Commands,
    ship: Single<(&Ship, &Placed)>,
    player: Single<&Transform, OnlyBody>,
    mut eyes: Single<&mut Transform, OnlyEyes>,
) {
    let (ship, placed) = *ship;
    let near = in_reach(&dock, ship, placed, player.translation);
    dock.hatch.set_if_neq(Hatch { near });
    let pressed = dock
        .keys
        .just_pressed(dock.settings.controls.keys.ship.board);
    let docked = ship.landed && grounded(dock.skies.planet());
    if !pressed || dock.busy() || !docked || dock.anchor.frame != ship.frame {
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
    } else if near {
        dock.location.travel = Travel::InShip;
        commands.queue(|world: &mut World| seat(world, true));
        dock.events.write(ShipEvent::Boarded);
    }
}
