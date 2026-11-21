//! The scan: a few times a second, which animals near the local robot should be drawn. New ones are
//! spawned on the ground and fade in; those that went far, died or were left over fade out.

use std::sync::Arc;

use bevy::ecs::system::SystemParam;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::beast::{Beast, Birth, spawn};
use super::fade::leave;
use super::gate::{Spec, footing};
use super::herds::Herds;
use super::looks::Looks;
use super::place::Spot;
use super::steer::cruise;
use super::tune::{MOST, REACH};
use super::want::{From, Lead, Want, lingers, members, visitors};
use crate::life::Life;
use crate::life::body::Locomotion;
use crate::life::herd::floor;
use crate::player::Player;
use crate::space::{Anchor, Clock};
use crate::universe::Universe;
use crate::world::{Globe, Terrain};

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
    beasts: Query<'w, 's, (Entity, &'static mut Beast)>,
}

/// Reads which animals should be drawn, spawns the new ones and fades out the others.
pub fn scan(mut s: Scan, mut wants: Local<Vec<Want>>) {
    let Some((globe, from)) = look(&mut s) else {
        return;
    };
    let Some(life) = globe.life() else {
        return;
    };
    members(life, (&s.herds.herds, &s.herds.ledger), from, &mut wants);
    visitors(life, &s.herds.carcasses, from, &mut wants);
    visitors(life, &s.herds.slain, from, &mut wants);
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
/// [`SLACK`](super::tune::SLACK) of the robot of `from`, and returns how many stay. A member is kept
/// while it is near, wherever its place in its herd went, so one that flees away from its herd's place
/// fades out only once it leaves the drawn ring, never in front of the player.
fn keep(s: &mut Scan, wants: &[Want], from: From) -> usize {
    let now = s.time.elapsed_secs();
    let mut kept = 0;
    for (_, mut beast) in &mut s.beasts {
        if beast.fade.leaving() {
            continue;
        }
        let wanted = wants.iter().any(|w| w.lead.key() == beast.lead.key());
        let here = beast.spot.planet(beast.motion.root.at).normalize();
        let stays = wanted || lingers(beast.lead, here, from, &s.herds.ledger);
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
    if globe.has_sea() && globe.height(want.dir) < floor(species) {
        return false;
    }
    let spot = Spot::at(want.dir, s.terrain.globe().radius());
    let lift = match species.body.locomotion {
        Locomotion::Fly | Locomotion::Swarm => cruise(&species.body),
        _ => 0.0,
    };
    let at = Vec3::new(0.0, spot.ground(&s.terrain, Vec2::ZERO) + lift, 0.0);
    let facing = heading(s, want, &spot);
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
