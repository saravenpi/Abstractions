//! Boarding: taking the seat of the landed ship and stepping out beside its hatch.

use avian3d::prelude::ColliderDisabled;
use bevy::ecs::system::SystemParam;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::ShipEvent;
use super::body::Ship;
use super::deck::grounded;
use super::hold::idle;
use super::pilot::Autopilot;
use super::sky::Skies;
use crate::computer::Computer;
use crate::player::{EYE_OFFSET, Floor, Interact, Player, Verb, place, seat};
use crate::render::GameCamera;
use crate::settings::{Persistent, Settings};
use crate::space::{Anchor, Location, Placed, Travel, up};

/// Distance in metres from the hull centre within which the player can board.
const BOARD_RANGE: f32 = 7.0;

/// Where the player steps out in body axes: beside the cockpit, in front of the wing, where
/// the open canopy is within reach.
const HATCH: DVec3 = DVec3::new(-1.6, 0.0, -1.5);

/// Filter of the player body, which is not the camera.
type OnlyBody = (With<Player>, Without<GameCamera>);

/// Filter of the camera, a child of the player body.
type OnlyEyes = (With<GameCamera>, Without<Player>);

/// What boarding reads and writes besides the bodies.
#[derive(SystemParam)]
pub struct Dock<'w, 's> {
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
    /// The route the ship flies, dropped when the pilot leaves.
    autopilot: ResMut<'w, Autopilot>,
    /// The ground colliders beside the hatch.
    floor: Floor<'w, 's>,
    /// Where the feet went down beside the hatch on the last frame leaving was offered, in the
    /// frame of the anchor: the spot a press acts on when this frame finds none.
    spot: Local<'s, Option<DVec3>>,
    /// The action the interact key does this frame, which boarding and leaving are offered to.
    interact: ResMut<'w, Interact>,
}

impl Dock<'_, '_> {
    /// Returns the render position the feet go down on beside the hatch of a ship with pose
    /// `at`: the top of the ground collider there, or the sea over it. Nothing while no collider
    /// is loaded beside the hatch over dry ground: a body put there would fall through.
    fn step_out(&self, at: &Placed) -> Option<Vec3> {
        let hatch = at.pos + at.rot * HATCH;
        let render = self.anchor.to_render(hatch);
        let vertical = up(&self.anchor, render);
        let ground = self.floor.under(render, vertical);
        if self.skies.height(hatch) >= self.skies.sea() {
            return ground;
        }
        let radius = self
            .skies
            .planet()
            .map_or(hatch.length(), |planet| planet.radius);
        let sea = self
            .anchor
            .to_render(hatch.normalize_or(DVec3::Y) * (radius + self.skies.sea()));
        let higher = |feet: Vec3| [sea, feet][usize::from((feet - sea).dot(vertical) > 0.0)];
        Some(ground.map_or(sea, higher))
    }
}

/// Returns true when the player on foot stands close enough to the landed ship to board it.
fn in_reach(dock: &Dock, ship: &Ship, placed: &Placed, body: Vec3) -> bool {
    let here = dock.anchor.frame == ship.frame;
    let near = body.distance(dock.anchor.to_render(placed.pos)) < BOARD_RANGE;
    ship.landed && here && near && dock.location.travel == Travel::OnFoot
}

/// Whether the player on foot stands at the hatch of the landed ship, where the board key
/// works, and whether the pilot waits for the ground beside it. The ship refreshes it every
/// frame of play; anything may read it, for a prompt.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Hatch {
    /// True while the board key would seat the player.
    pub near: bool,
    /// True while the pilot of the landed ship cannot step out: no ground collider is loaded
    /// beside the hatch yet.
    pub waiting: bool,
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

/// Seats the player near the landed ship on the interact key, and on the same key puts the
/// player back on the ground beside the hatch, with the motors stopped and the autopilot off.
/// Both are offered to the interact key, which does them on the frame of the press while the
/// prompt names them. The key does nothing while the board computer is open, and nothing over a
/// gas giant, where there is no ground to step out on. Only a landed ship can be left, so it
/// stays where it stands. Leaving is offered only while the ground beside the hatch has its
/// collider: until then the pilot stays seated and the hatch says it waits.
pub fn board(
    mut dock: Dock,
    mut commands: Commands,
    ship: Single<(&mut Ship, &Placed)>,
    player: Single<&Transform, OnlyBody>,
    mut eyes: Single<&mut Transform, OnlyEyes>,
) {
    let (mut ship, placed) = ship.into_inner();
    let near = in_reach(&dock, &ship, placed, player.translation);
    let docked = ship.landed && grounded(dock.skies.planet());
    let busy = dock
        .computer
        .as_deref()
        .is_some_and(|state| *state.get() == Computer::Open);
    if busy || !docked || dock.anchor.frame != ship.frame {
        dock.hatch.set_if_neq(Hatch {
            near,
            waiting: false,
        });
        *dock.spot = None;
        return;
    }
    if dock.location.travel == Travel::OnFoot {
        dock.hatch.set_if_neq(Hatch {
            near,
            waiting: false,
        });
        embark(&mut dock, &mut commands, near);
        return;
    }
    disembark(
        &mut dock,
        &mut commands,
        (&mut ship, placed),
        &mut eyes,
        near,
    );
}

/// Offers leaving to the pilot of the landed ship while the ground beside the hatch has its
/// collider, and steps out on the frame the interact key goes down while the prompt names it:
/// onto the ground found this frame, or the one found the frame the prompt was settled on.
fn disembark(
    dock: &mut Dock,
    commands: &mut Commands,
    (ship, placed): (&mut Ship, &Placed),
    eyes: &mut Transform,
    near: bool,
) {
    let feet = dock.step_out(placed);
    dock.hatch.set_if_neq(Hatch {
        near,
        waiting: feet.is_none(),
    });
    let here = feet.map(|feet| dock.anchor.to_frame(feet));
    let spot = here.or(*dock.spot);
    *dock.spot = here;
    if here.is_some() {
        dock.interact.offer(Verb::Leave, "");
    }
    let key = dock.settings.controls.keys.ship.board;
    let pressed = dock.interact.pressed(Verb::Leave, &dock.keys, key);
    if let Some(spot) = spot.filter(|_| pressed) {
        let feet = dock.anchor.to_render(spot);
        leave(dock, commands, (ship, placed), (eyes, feet));
    }
}

/// Offers boarding to the player on foot `near` the hatch, and seats the player on the frame
/// the interact key goes down while the prompt names it.
fn embark(dock: &mut Dock, commands: &mut Commands, near: bool) {
    if !near {
        return;
    }
    dock.interact.offer(Verb::Board, "");
    let key = dock.settings.controls.keys.ship.board;
    if dock.interact.pressed(Verb::Board, &dock.keys, key) {
        dock.location.travel = Travel::InShip;
        commands.queue(|world: &mut World| seat(world, true));
        dock.events.write(ShipEvent::Boarded);
    }
}

/// Puts the player with its feet at the render position `feet` beside the hatch, with the motors
/// stopped and the autopilot off, and the camera back on the eyes.
fn leave(
    dock: &mut Dock,
    commands: &mut Commands,
    (ship, placed): (&mut Ship, &Placed),
    (eyes, feet): (&mut Transform, Vec3),
) {
    let toward = dock.anchor.rotation_to_render(placed.rot) * Vec3::NEG_Z;
    dock.location.travel = Travel::OnFoot;
    idle(ship);
    dock.autopilot.cancel();
    *eyes = Transform::from_xyz(0.0, EYE_OFFSET, 0.0);
    commands.queue(move |world: &mut World| {
        place(world, feet, toward);
        seat(world, false);
    });
    dock.events.write(ShipEvent::Left);
}
