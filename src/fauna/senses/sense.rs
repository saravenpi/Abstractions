//! The sense system: every frame, the robots and chasing hunters near the local robot, and each drawn
//! animal's senses and stance towards them, each animal on every third frame.

use bevy::ecs::system::SystemParam;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::Blood;
use super::cues::loudness;
use super::feel::Threat;
use super::judge::{Ctx, judge};
use super::tune::{GRACE, JUMP, ROBOT_KG, SENSE_EVERY, WALK};
use crate::avatar::{Avatar, AvatarPose};
use crate::fauna::beast::Beast;
use crate::fauna::gate::footing;
use crate::fauna::ground::Heights;
use crate::fauna::herds::Herds;
use crate::fauna::tune::MAX_STEP;
use crate::fauna::want::Lead;
use crate::life::Life;
use crate::life::herd::HerdState;
use crate::player::Player;
use crate::space::{Anchor, Clock};
use crate::universe::BodyId;
use crate::wind::Wind;
use crate::world::Terrain;

/// What the senses read and write.
#[derive(SystemParam)]
pub struct Sense<'w, 's> {
    clock: Res<'w, Clock>,
    time: Res<'w, Time>,
    terrain: Res<'w, Terrain>,
    anchor: Res<'w, Anchor>,
    wind: Option<Res<'w, Wind>>,
    herds: Res<'w, Herds>,
    blood: ResMut<'w, Blood>,
    robot: Query<'w, 's, (Entity, &'static Transform), With<Player>>,
    pose: Query<'w, 's, &'static AvatarPose, (With<Avatar>, With<ChildOf>)>,
    remotes: Remotes<'w, 's>,
    beasts: Query<'w, 's, (Entity, &'static mut Beast)>,
}

/// The robots of the other players, standing free, with their poses when known.
type Remotes<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static Transform, Option<&'static AvatarPose>),
    (With<Avatar>, Without<ChildOf>, Without<Player>),
>;

/// What the senses keep from frame to frame, so they never allocate once warm.
#[derive(Default)]
pub struct Scratch {
    /// The threats of this frame.
    threats: Vec<Threat>,
    /// Herds whose lookout fled last frame, with the planet position of what it fled.
    alarms: Vec<(u64, DVec3)>,
    /// Herds whose lookout flees this frame.
    raised: Vec<(u64, DVec3)>,
    /// Heights of the drawn ground read so far.
    heights: Heights,
    /// Frames counted, to phase each animal's sense.
    frame: u32,
    /// Planet under the robot, game seconds of the last landing, and the robot's last position.
    landed: (Option<BodyId>, f32, Option<DVec3>),
}

/// Senses the threats of every drawn animal and sets its stance.
pub fn sense(mut s: Sense, mut scratch: Local<Scratch>) {
    let globe = s.terrain.globe().clone();
    let (Some(life), Ok((me, robot))) = (globe.life(), s.robot.single()) else {
        return;
    };
    let at = s.anchor.to_frame(robot.translation);
    let now = s.time.elapsed_secs();
    ready(&s, &mut scratch, life, (me, at, now));
    let scratch = &mut *scratch;
    let ctx = Ctx {
        terrain: &s.terrain,
        herds: &s.herds.herds,
        time: s.clock.seconds,
        dt: s.time.delta_secs().min(MAX_STEP) * SENSE_EVERY as f32,
        wind: blowing(&s, robot.translation, at),
        grace: now - scratch.landed.1 < GRACE,
        threats: &scratch.threats,
        alarms: &scratch.alarms,
        heights: &scratch.heights,
    };
    for (entity, mut beast) in &mut s.beasts {
        let species = life.species.get(usize::from(beast.index));
        let alive = !matches!(beast.lead, Lead::Carcass { .. }) && turn(entity, scratch.frame);
        if let Some(species) = species.filter(|_| alive) {
            judge(
                &ctx,
                (&mut beast, species),
                (&mut scratch.raised, &mut s.blood),
            );
        }
    }
}

/// Returns true when the animal `entity` senses on frame `frame`: one frame in [`SENSE_EVERY`], by its
/// entity index.
fn turn(entity: Entity, frame: u32) -> bool {
    entity
        .index_u32()
        .wrapping_add(frame)
        .is_multiple_of(SENSE_EVERY)
}

/// Readies the scratch for this frame: the ground heights, the last landing, the threats of `life`
/// around the local robot `me` at planet position `at` at game second `now`, and, every third frame,
/// the alarms of the last three.
fn ready(s: &Sense, scratch: &mut Scratch, life: &Life, (me, at, now): (Entity, DVec3, f32)) {
    if s.terrain.is_changed() {
        scratch.heights.clear();
    }
    land(&mut scratch.landed, footing(&s.anchor), at, now);
    scratch.threats.clear();
    robots(s, &mut scratch.threats, (me, at));
    hunters(s, life, &mut scratch.threats);
    scratch.frame = scratch.frame.wrapping_add(1);
    if scratch.frame.is_multiple_of(SENSE_EVERY) {
        std::mem::swap(&mut scratch.alarms, &mut scratch.raised);
        scratch.raised.clear();
    }
}

/// Returns the wind velocity in planet axes at the local robot, at render position `render` and
/// planet position `at`, or none without wind.
fn blowing(s: &Sense, render: Vec3, at: DVec3) -> DVec3 {
    s.wind.as_deref().map_or(DVec3::ZERO, |w| {
        s.anchor.to_frame(render + w.velocity()) - at
    })
}

/// Moves the last landing to `now` when the robot came onto another planet `body` or jumped far
/// from its last position, now `at`.
fn land(
    landed: &mut (Option<BodyId>, f32, Option<DVec3>),
    body: Option<BodyId>,
    at: DVec3,
    now: f32,
) {
    let jumped = landed.2.is_some_and(|p| p.distance(at) > JUMP);
    if landed.0 != body || jumped {
        landed.1 = now;
    }
    landed.0 = body;
    landed.2 = Some(at);
}

/// Returns the threat of the robot `key` at planet position `pos` with `pose`.
fn robot(key: Entity, pos: DVec3, pose: Option<&AvatarPose>) -> Threat {
    let pose = pose.copied().unwrap_or_default();
    let speed = pose.speed.hypot(pose.vertical_speed);
    let impact = if pose.absorb { pose.impact } else { 0.0 };
    Threat {
        pos,
        speed,
        loud: loudness(speed, pose.thrust, impact, pose.crouch),
        mass: ROBOT_KG,
        key: key.to_bits(),
        herd: 0,
        robot: true,
        crouch: pose.crouch,
    }
}

/// Adds the local robot `me` at planet position `at` and the robots of the other players to
/// `threats`.
fn robots(s: &Sense, threats: &mut Vec<Threat>, (me, at): (Entity, DVec3)) {
    threats.push(robot(me, at, s.pose.iter().next()));
    for (key, t, pose) in &s.remotes {
        threats.push(robot(key, s.anchor.to_frame(t.translation), pose));
    }
}

/// Adds the drawn hunters of `life` whose herd is on a chase to `threats`.
fn hunters(s: &Sense, life: &Life, threats: &mut Vec<Threat>) {
    for (_, beast) in &s.beasts {
        let Lead::Herd { id, slot } = beast.lead else {
            continue;
        };
        let herd = s.herds.herds.iter().find(|h| h.id == id);
        let Some(herd) = herd.filter(|h| h.state == HerdState::Chase) else {
            continue;
        };
        let mass = life
            .species
            .get(usize::from(beast.index))
            .map_or(1.0, |sp| sp.history.mass);
        threats.push(Threat {
            pos: beast.spot.planet(beast.motion.root.at),
            speed: herd.speed,
            loud: herd.speed / WALK,
            mass,
            key: id.rotate_left(8) ^ u64::from(slot),
            herd: id,
            robot: false,
            crouch: 0.0,
        });
    }
}
