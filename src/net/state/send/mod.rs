//! Sending the robot, the ship and the companion of this machine: one batch a frame, each
//! record at its own rate, the batch and the body written into buffers that are kept.

mod kinds;
mod vitals;

pub use kinds::{Here, Pilot, Sent, Trail};

use abstractions_net::wire::push_within;
use abstractions_net::wire::{AT_REST, Body, Cadences, CompanionBody, Head, Record};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::Remote;
use super::bodies::{Bodies, star_of};
use super::frame::Place;
use super::rate::REST_SPEED;
use crate::avatar::strike::Strike;
use crate::avatar::{Avatar, AvatarPose};
use crate::companion::{Companion, Display, Float};
use crate::condition::Condition;
use crate::net::link::Link;
use crate::player::{Motion, Player};
use crate::render::Heat;
use crate::ship::{Helm, Ship, ShipTelemetry, WarpState};
use crate::sim::PlayerId;
use crate::space::{Anchor, Clock, Location, Placed, Travel};
use crate::universe::{StarId, Universe};

/// The batch being written, kept from frame to frame so that sending allocates nothing once
/// it has grown, and when each kind is due.
#[derive(Resource, Debug, Default)]
pub struct Outbox {
    /// The datagram of the frame.
    pub batch: Vec<u8>,
    /// The body of the record being written.
    body: Vec<u8>,
    /// When the next record of each kind is due.
    cadence: Cadences,
    /// The star this machine was last at.
    star: Option<StarId>,
    /// Where the companion was on the last frame.
    trail: Trail,
}

/// What every record of a frame shares.
struct Stamp {
    /// The id the server gave this player.
    player: PlayerId,
    /// Universe time of the frame.
    time: f64,
    /// Local time in seconds, which the rates count in.
    now: f64,
    /// Most bytes the datagram may carry.
    max: usize,
}

impl Outbox {
    /// Adds the record of `sent` to the batch when it holds one and its rate says it is due.
    fn offer<B: Body>(&mut self, stamp: &Stamp, sent: Option<Sent<B>>) {
        let Some(sent) = sent else {
            return;
        };
        if !self.cadence.due(B::KIND, stamp.now, sent.rest) {
            return;
        }
        self.body.clear();
        sent.body.encode(&sent.frame, &mut self.body);
        let head = Head {
            player: stamp.player,
            kind: B::KIND,
            frame: sent.frame,
            time: stamp.time,
            flags: if sent.rest { AT_REST } else { 0 },
        };
        let record = Record {
            head,
            body: &self.body,
        };
        push_within(&mut self.batch, &record, stamp.max);
    }
}

/// Where this machine stands.
#[derive(SystemParam)]
pub struct Scene<'w> {
    /// The render anchor.
    anchor: Option<Res<'w, Anchor>>,
    /// Where the player is.
    location: Option<Res<'w, Location>>,
    /// The universe.
    universe: Option<Res<'w, Universe>>,
    /// Universe time.
    clock: Res<'w, Clock>,
    /// Local time.
    time: Res<'w, Time<Real>>,
    /// The instruments of the ship.
    telemetry: Option<Res<'w, ShipTelemetry>>,
    /// What the pilot asks of the ship.
    helm: Option<Res<'w, Helm>>,
    /// The glow of the craft of this machine.
    heat: Option<Res<'w, Heat>>,
    /// How whole each part of the robot is.
    condition: Option<Res<'w, Condition>>,
}

/// The things of this machine that are sent.
#[derive(SystemParam)]
pub struct Locals<'w, 's> {
    /// The player bodies.
    players: Query<
        'w,
        's,
        (
            Entity,
            &'static PlayerId,
            &'static GlobalTransform,
            &'static Motion,
        ),
        With<Player>,
    >,
    /// The robots, with the body each hangs from.
    avatars: Query<
        'w,
        's,
        (
            &'static ChildOf,
            &'static AvatarPose,
            Option<&'static Strike>,
        ),
        With<Avatar>,
    >,
    /// The ship of this machine.
    ship: Query<'w, 's, (&'static Ship, Option<&'static Placed>)>,
    /// The companion of this machine.
    companion: Query<'w, 's, (&'static GlobalTransform, &'static Float, &'static Display), OwnBot>,
}

/// Filter of the companion of this machine: not a puppet of another player's.
type OwnBot = (With<Companion>, Without<Remote>);

/// Returns where this machine stands, or nothing before its world exists.
fn here(scene: &Scene, ship: Option<&Ship>, last: Option<StarId>) -> Option<Here> {
    let (anchor, location) = (*scene.anchor.as_deref()?, scene.location.as_deref()?);
    let seated = location.travel == Travel::InShip;
    let resting = ship.is_none_or(|ship| ship.landed || ship.vel.length() < REST_SPEED);
    Some(Here {
        anchor,
        place: Place::from(anchor.frame),
        time: scene.clock.seconds,
        seated,
        landed: ship.is_some_and(|ship| ship.landed),
        still: !seated || resting,
        last: star_of(Place::from(anchor.frame)).or(last),
    })
}

impl Scene<'_> {
    /// Returns what the pilot of the ship of this machine holds: the stick, pitch up then roll
    /// left, and the boost.
    fn pilot(&self) -> Pilot {
        self.helm.as_deref().map_or(([0.0; 2], 0.0), |helm| {
            let turn = helm.controls.turn.as_vec3();
            ([turn.x, turn.z], f32::from(u8::from(helm.controls.boost)))
        })
    }

    /// Returns what the warp drive of the ship of this machine does.
    fn drive(&self) -> WarpState {
        self.telemetry
            .as_deref()
            .map(|t| t.warp)
            .unwrap_or_default()
    }
}

/// The body of the player at this machine, how it moves and how its robot is posed with its swing.
type Mine<'a> = (&'a GlobalTransform, &'a Motion, (&'a AvatarPose, (u8, f32)));

impl Locals<'_, '_> {
    /// Returns the body of the player at this machine, how it moves and how its robot is posed.
    fn mine(&self) -> Option<Mine<'_>> {
        let local = |(_, id, ..): &(Entity, &PlayerId, _, _)| **id == PlayerId::LOCAL;
        let (entity, _, at, motion) = self.players.iter().find(local)?;
        let hung =
            |(parent, ..): &(&ChildOf, &AvatarPose, Option<&Strike>)| parent.parent() == entity;
        let (_, pose, strike) = self.avatars.iter().find(hung)?;
        let swing = strike.map_or((0, 0.0), super::swing::pack);
        Some((at, motion, (pose, swing)))
    }

    /// Returns the record of the companion of this machine, which was at `was` a frame ago.
    fn bot(&self, here: &Here, was: Trail, bodies: &mut Bodies) -> Option<Sent<CompanionBody>> {
        let (at, float, display) = self.companion.single().ok()?;
        kinds::companion(here, at, (&float.pose, display.shown), was, bodies)
    }
}

/// Sends the records that are due in one batch, once the server gave this player an id.
pub fn send(
    mut link: ResMut<Link>,
    mut out: ResMut<Outbox>,
    mut bodies: ResMut<Bodies>,
    scene: Scene,
    locals: Locals,
) {
    let ship = locals.ship.single().ok();
    let found = here(&scene, ship.map(|(ship, _)| ship), out.star);
    let (Some(player), Some(here), Some(universe)) = (link.you, found, scene.universe.as_deref())
    else {
        return;
    };
    bodies.sync(universe);
    out.star = here.last;
    out.batch.clear();
    let (now, max) = (scene.time.elapsed_secs_f64(), link.max_datagram());
    let stamp = Stamp {
        player,
        time: here.time,
        now,
        max,
    };
    let walker = locals.mine();
    let walker =
        walker.and_then(|(at, motion, hands)| kinds::robot(&here, at, motion, hands, &mut bodies));
    out.offer(&stamp, walker);
    out.offer(&stamp, scene.vitals(&here, walker));
    let flown = scene.flown(&here);
    let hull = ship.and_then(|(ship, placed)| kinds::ship(&here, ship, placed, flown, &mut bodies));
    out.offer(&stamp, hull);
    let bot = locals.bot(&here, out.trail, &mut bodies);
    out.trail = bot.map(|sent| (sent.frame, sent.body.position, here.time));
    out.offer(&stamp, bot);
    if !out.batch.is_empty() && max > 0 {
        link.send_records(&out.batch);
    }
}
