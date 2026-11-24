//! Boarding: taking the seat of the landed ship and stepping out beside its hatch.

mod fleet;
mod spot;

pub use fleet::{BoardGrant, Fleet, SwapShip};

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
use crate::sim::PlayerId;
use crate::space::{Anchor, Location, Placed, Travel};
use crate::world::Terrain;

/// Distance in metres from the hull centre within which the player can board.
const BOARD_RANGE: f32 = 7.0;

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
    pub(super) anchor: Res<'w, Anchor>,
    /// What surrounds the ship.
    pub(super) skies: Res<'w, Skies>,
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
    pub(super) floor: Floor<'w, 's>,
    /// Where the feet went down beside the hatch on the last frame leaving was offered, in the
    /// frame of the anchor: the spot a press acts on when this frame finds none.
    spot: Local<'s, Option<DVec3>>,
    /// The action the interact key does this frame, which boarding and leaving are offered to.
    interact: ResMut<'w, Interact>,
    /// The terrain of the planet, whose drawn ground the feet go down on before its collider loads.
    pub(super) terrain: Option<Res<'w, Terrain>>,
    /// Online fleet ownership and boarding handshake state.
    fleet: Option<ResMut<'w, Fleet>>,
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

/// Applies an approved online boarding grant, swapping remote ship state in when needed, and seats
/// the player inside the local `Ship` entity.
fn apply_grant(
    grant: BoardGrant,
    dock: &mut Dock,
    commands: &mut Commands,
    (entity, ship, placed, vis): (Entity, &mut Ship, &mut Placed, &mut Visibility),
) {
    if let BoardGrant::Remote(s) = grant {
        let mut next = Ship::at_rest(s.frame, s.pos, s.rot);
        next.vel = s.vel;
        next.landed = s.landed;
        next.touching = s.landed;
        next.damage = s.damage;
        next.gear = s.gear;
        next.stowed = s.gear < 0.5;
        next.headlight = s.headlight;
        *ship = next;
        *placed = Placed {
            pos: s.pos,
            rot: s.rot,
        };
        dock.autopilot.cancel();
    }
    *vis = Visibility::Inherited;
    commands.entity(entity).remove::<ColliderDisabled>();
    dock.location.travel = Travel::InShip;
    commands.queue(|world: &mut World| seat(world, true));
    dock.events.write(ShipEvent::Boarded);
}

/// Seats the player near a landed ship on the interact key, and on the same key puts the
/// player back on the ground beside the hatch, with the motors stopped and the autopilot off.
pub fn board(
    mut dock: Dock,
    mut commands: Commands,
    ship: Single<(Entity, &mut Ship, &mut Placed, &mut Visibility)>,
    player: Single<&Transform, OnlyBody>,
    mut eyes: Single<&mut Transform, OnlyEyes>,
) {
    let (entity, mut ship, mut placed, mut vis) = ship.into_inner();
    if dock
        .fleet
        .as_deref_mut()
        .is_some_and(|f| std::mem::take(&mut f.yield_local))
    {
        *vis = Visibility::Hidden;
        commands.entity(entity).insert(ColliderDisabled);
    }
    if let Some(grant) = dock.fleet.as_deref_mut().and_then(|f| f.granted.take()) {
        apply_grant(
            grant,
            &mut dock,
            &mut commands,
            (entity, &mut ship, &mut placed, &mut vis),
        );
        return;
    }
    let busy = dock
        .computer
        .as_deref()
        .is_some_and(|state| *state.get() == Computer::Open);
    let docked = ship.landed && grounded(dock.skies.planet());
    let online = dock.fleet.as_deref().is_some_and(|f| f.online);
    let active_id = dock.fleet.as_deref().and_then(|f| f.active);
    let local_ok = (!online || active_id.is_some()) && docked && dock.anchor.frame == ship.frame;
    let near_local = local_ok && in_reach(&dock, &ship, &placed, player.translation);
    if dock.location.travel == Travel::OnFoot {
        let remote_id = dock.fleet.as_deref().and_then(|f| f.near_remote);
        let near = !busy && (near_local || (online && remote_id.is_some()));
        dock.hatch.set_if_neq(Hatch {
            near,
            waiting: false,
        });
        *dock.spot = None;
        let target = if near_local { active_id } else { remote_id };
        embark(&mut dock, &mut commands, near, (online, target));
        return;
    }
    if busy || !docked || dock.anchor.frame != ship.frame {
        dock.hatch.set_if_neq(Hatch {
            near: near_local,
            waiting: false,
        });
        *dock.spot = None;
        return;
    }
    disembark(
        &mut dock,
        &mut commands,
        (&mut ship, &placed),
        &mut eyes,
        near_local,
    );
}

/// Offers leaving to the pilot of the landed ship while the ground beside the hatch is ready.
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

/// Offers boarding to the player on foot `near` a landed ship hatch, either seating immediately
/// offline or queueing an online boarding request for `target`.
fn embark(
    dock: &mut Dock,
    commands: &mut Commands,
    near: bool,
    (online, target): (bool, Option<PlayerId>),
) {
    if !near {
        return;
    }
    dock.interact.offer(Verb::Board, "");
    let key = dock.settings.controls.keys.ship.board;
    if !dock.interact.pressed(Verb::Board, &dock.keys, key) {
        return;
    }
    if online {
        if let (Some(fleet), Some(target)) = (dock.fleet.as_deref_mut(), target) {
            fleet.pending_board = Some(target);
        }
        return;
    }
    dock.location.travel = Travel::InShip;
    commands.queue(|world: &mut World| seat(world, true));
    dock.events.write(ShipEvent::Boarded);
}

/// Puts the player with its feet at the render position `feet` beside the hatch, with the motors
/// stopped and the autopilot off, and queues a server leave notification when online.
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
    if let Some(fleet) = dock.fleet.as_deref_mut().filter(|f| f.online) {
        fleet.pending_leave = fleet.active;
    }
    dock.events.write(ShipEvent::Left);
}
