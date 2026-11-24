//! The scan: a few times a second, which animals near the local robot should be drawn. New ones are
//! spawned on the ground and fade in; those that went far, died or were left over fade out.

use std::sync::Arc;

use bevy::ecs::system::SystemParam;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::beast::{Beast, Birth, spawn};
use super::fade::leave;
use super::gate::{Spec, footing};
use super::herds::{Herds, herd_arc};
use super::looks::Looks;
use super::place::Spot;
use super::steer::cruise;
use super::sync::RemoteFauna;
use super::tune::{MOST, REACH, SLACK};
use super::want::{From, Lead, Want, lingers, members, visitors};
use crate::avatar::Avatar;
use crate::life::Life;
use crate::life::body::Locomotion;
use crate::life::herd::floor;
use crate::player::Player;
use crate::space::{Anchor, Clock};
use crate::universe::Universe;
use crate::world::{Globe, Terrain};

/// Share of [`REACH`] within which remote players and hurt animals stay in the scan.
const COOP_REACH: f64 = 2.5 * REACH;

/// The robots of the other players standing free on the planet.
type Remotes<'w, 's> =
    Query<'w, 's, &'static Transform, (With<Avatar>, Without<ChildOf>, Without<Player>)>;

/// What the scan reads and writes.
#[derive(SystemParam)]
pub struct Scan<'w, 's> {
    commands: Commands<'w, 's>,
    clock: Res<'w, Clock>,
    time: Res<'w, Time>,
    terrain: Res<'w, Terrain>,
    anchor: Res<'w, Anchor>,
    universe: Option<Res<'w, Universe>>,
    herds: ResMut<'w, Herds>,
    spec: ResMut<'w, Spec>,
    looks: ResMut<'w, Looks>,
    stores: (
        ResMut<'w, Assets<Mesh>>,
        ResMut<'w, Assets<StandardMaterial>>,
    ),
    robot: Query<'w, 's, &'static Transform, With<Player>>,
    remotes: Remotes<'w, 's>,
    beasts: Query<'w, 's, (Entity, &'static mut Beast)>,
    remote: Option<Res<'w, RemoteFauna>>,
}

/// Reads which animals should be drawn, spawns the new ones and fades out the others.
pub fn scan(mut s: Scan, mut wants: Local<Vec<Want>>, mut scratch: Local<Vec<Want>>) {
    let Some((globe, from)) = look(&mut s) else {
        return;
    };
    let Some(life) = globe.life() else {
        return;
    };
    members(life, (&s.herds.herds, &s.herds.ledger), from, &mut wants);
    visitors(life, &s.herds.carcasses, from, &mut wants);
    visitors(life, &s.herds.slain, from, &mut wants);
    friends(&s, life, from, (&mut wants, &mut scratch));
    if let Some(remote) = s.remote.as_deref() {
        super::sync::fold_wants(
            remote,
            (life, &s.herds.carcasses, &s.herds.ledger),
            from,
            &mut wants,
        );
    }
    let mut room = MOST.saturating_sub(keep(&mut s, &wants, from));
    let now = s.time.elapsed_secs();
    for want in wants.iter().filter(|w| w.metres <= REACH) {
        if room == 0 {
            break;
        }
        if !drawn(&s, want.lead) && born(&mut s, (life, globe.seed()), want, now) {
            room -= 1;
        }
    }
}

/// Folds the wants around every remote player within [`COOP_REACH`] of the local robot into `wants`.
fn friends(s: &Scan, life: &Life, from: From, (wants, scratch): (&mut Vec<Want>, &mut Vec<Want>)) {
    for remote in &s.remotes {
        let dir = s.anchor.to_frame(remote.translation).normalize();
        if herd_arc(dir, from.robot) * from.radius > COOP_REACH {
            continue;
        }
        let view = From { robot: dir, ..from };
        members(life, (&s.herds.herds, &s.herds.ledger), view, scratch);
        visitors(life, &s.herds.carcasses, view, scratch);
        visitors(life, &s.herds.slain, view, scratch);
        for item in scratch.drain(..) {
            match wants.iter_mut().find(|w| w.lead.key() == item.lead.key()) {
                Some(w) => w.metres = w.metres.min(item.metres),
                None => wants.push(item),
            }
        }
    }
    wants.sort_unstable_by(|a, b| {
        a.metres
            .total_cmp(&b.metres)
            .then(a.lead.key().cmp(&b.lead.key()))
    });
}

/// Reads the planet under the anchor and the herds around the local robot, and returns the globe and
/// where the wants are read from, when the anchor stands on a living planet with a robot.
fn look(s: &mut Scan) -> Option<(Arc<Globe>, From)> {
    let body = footing(&s.anchor)?;
    if s.spec.planet.as_ref().is_none_or(|p| p.id != body) {
        s.spec.planet = Some(Arc::new(s.universe.as_deref()?.planet(body)));
    }
    let globe = s.terrain.globe().clone();
    let (robot, planet) = (s.robot.single().ok()?, s.spec.planet.clone()?);
    globe.life()?;
    let at = s.anchor.to_frame(robot.translation).normalize();
    let time = s.clock.seconds;
    s.herds.scan((&globe, &planet), at, time);
    let radius = globe.radius();
    Some((
        globe.clone(),
        From {
            robot: at,
            time,
            radius,
        },
    ))
}

/// Fades out every drawn animal that is neither wanted nor a living herd member within [`REACH`] and
/// [`SLACK`] of the robot of `from` or of a nearby player, nor hurt within [`COOP_REACH`], and returns
/// how many stay. A member is kept while it is near, wherever its place in its herd went, so one that
/// flees away from its herd's place fades out only once it leaves the drawn ring, never in front of the
/// player.
fn keep(s: &mut Scan, wants: &[Want], from: From) -> usize {
    let now = s.time.elapsed_secs();
    let mut kept = 0;
    for (_, mut beast) in &mut s.beasts {
        if beast.fade.leaving() {
            continue;
        }
        let wanted = wants.iter().any(|w| w.lead.key() == beast.lead.key());
        let here = beast.spot.planet(beast.motion.root.at).normalize();
        let friend = s.remotes.iter().any(|t| {
            let dir = s.anchor.to_frame(t.translation).normalize();
            herd_arc(dir, from.robot) * from.radius <= COOP_REACH
                && herd_arc(here, dir) * from.radius <= REACH + SLACK
        });
        let hurt = beast.wits.provoked > 0.0
            || beast.wits.health.share() < 1.0
            || beast.wits.broken.0 != 0
            || beast.wits.leaking;
        let active = hurt && herd_arc(here, from.robot) * from.radius <= COOP_REACH;
        let synced = s
            .remote
            .as_deref()
            .is_some_and(|r| r.contains(beast.lead.key()))
            && herd_arc(here, from.robot) * from.radius <= COOP_REACH;
        let stays = wanted
            || friend
            || active
            || synced
            || lingers(beast.lead, here, from, &s.herds.ledger);
        if stays && kept < MOST {
            kept += 1;
        } else {
            leave(&mut s.commands, &mut beast, now);
        }
    }
    kept
}

/// Returns true when the animal led by `lead` is drawn, leaving or not.
fn drawn(s: &Scan, lead: Lead) -> bool {
    s.beasts.iter().any(|(_, b)| b.lead.key() == lead.key())
}

/// Spawns the animal `want` of `life` on the planet of seed `planet`, on the ground where it stands,
/// facing its herd's way, at `now` game seconds, and returns true; or returns false and spawns nothing
/// where its place lies under water deeper than its body wades, see [`floor`]: a land animal is never
/// drawn standing on the bed of the sea or of a pond.
fn born(s: &mut Scan, (life, planet): (&Life, u64), want: &Want, now: f32) -> bool {
    let Some(species) = life.species.get(usize::from(want.species)) else {
        return false;
    };
    let globe = s.terrain.globe();
    let remote = s.remote.as_deref().and_then(|r| r.get(want.lead.key()));
    if remote.is_none() && globe.has_sea() && globe.height(want.dir) < floor(species) {
        return false;
    }
    let (spot, at, facing) = match remote {
        Some(target) => super::sync::birth_pose(target, &s.terrain, &species.body),
        None => {
            let spot = Spot::at(want.dir, globe.radius());
            let lift = match species.body.locomotion {
                Locomotion::Fly | Locomotion::Swarm => cruise(&species.body),
                _ => 0.0,
            };
            let at = Vec3::new(0.0, spot.ground(&s.terrain, Vec2::ZERO) + lift, 0.0);
            (spot, at, heading(s, want, &spot))
        }
    };
    let (meshes, materials) = (&mut s.stores.0, &mut s.stores.1);
    let meshes = s
        .looks
        .parts((planet, life), want.species, (meshes, materials))
        .to_vec();
    let skins = s.looks.paints((life, want.species), materials);
    let birth = Birth {
        lead: want.lead,
        species: (species, want.species),
        at: (spot, at, facing),
        now,
    };
    spawn(&mut s.commands, birth, &meshes, &skins);
    true
}

/// Returns the way `want` faces in the frame `spot`: its herd's heading, or towards the carcass it
/// visits.
fn heading(s: &Scan, want: &Want, spot: &Spot) -> Vec3 {
    let way = match want.lead {
        Lead::Herd { id, .. } => s
            .herds
            .herds
            .iter()
            .find(|h| h.id == id)
            .map_or(DVec3::X, |h| h.heading),
        Lead::Visit { at, .. } => at - want.dir,
        Lead::Carcass { key, .. } => {
            spot.rot * DVec3::new((key as f64).sin(), 0.0, (key as f64).cos())
        }
    };
    (spot.rot.inverse() * way)
        .as_vec3()
        .with_y(0.0)
        .normalize_or(Vec3::NEG_Z)
}
