//! The walk: every frame, each drawn animal heads for its place at universe time, or its den when
//! wounded, acts on the stance its senses set, steps its motion on the drawn ground and poses its
//! parts.

use bevy::ecs::system::SystemParam;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::Frozen;
use super::beast::{Beast, Part};
use super::gate::Spec;
use super::ground::Heights;
use super::herds::{Herds, Land};
use super::lead::goal;
use super::motion::{Detail, Motion};
use super::steer::{Aim, Pace, cruise, drive};
use super::tune::{DRIFT, FROUDE_TOP, MAX_STEP, NEAR};
use super::want::Lead;
use crate::life::Species;
use crate::life::body::Locomotion;
use crate::life::herd::Herd;
use crate::life::hunt::Hunt;
use crate::player::Player;
use crate::space::{Anchor, Clock, Placed};
use crate::world::Terrain;

/// What the walk reads and writes.
#[derive(SystemParam)]
pub struct Walk<'w, 's> {
    clock: Res<'w, Clock>,
    time: Res<'w, Time>,
    terrain: Res<'w, Terrain>,
    anchor: Res<'w, Anchor>,
    spec: Res<'w, Spec>,
    herds: ResMut<'w, Herds>,
    robot: Query<'w, 's, &'static Transform, With<Player>>,
    remotes: Remotes<'w, 's>,
    beasts: Query<'w, 's, (&'static mut Beast, &'static mut Placed), Without<Frozen>>,
    parts: Parts<'w, 's>,
}

/// The drawn parts of the animals.
type Parts<'w, 's> = Query<'w, 's, &'static mut Transform, (With<Part>, Without<Player>)>;

/// What the walk keeps from frame to frame, so it never allocates once warm.
#[derive(Default)]
pub struct Scratch {
    /// Herds of the drawn animals at this frame's time, by identity.
    herds: Vec<(u64, Option<Herd>)>,
    /// Heights of the drawn ground read so far.
    heights: Heights,
}

/// What every animal reads this frame.
struct Ctx<'a> {
    terrain: &'a Terrain,
    time: f64,
    dt: f32,
    robot: DVec3,
    scratch: &'a Scratch,
    hunts: &'a [Hunt],
}

/// Steps every drawn animal and poses its parts.
pub fn walk(mut w: Walk, mut scratch: Local<Scratch>) {
    let globe = w.terrain.globe().clone();
    let (Some(life), Some(planet), Ok(robot)) =
        (globe.life(), w.spec.planet.clone(), w.robot.single())
    else {
        return;
    };
    let robot = w.anchor.to_frame(robot.translation);
    if w.terrain.is_changed() {
        scratch.heights.clear();
    }
    let land = Land {
        globe: &globe,
        life,
        planet: &planet,
    };
    gather(&mut scratch, &mut w, land);
    let ctx = Ctx {
        terrain: &w.terrain,
        time: w.clock.seconds,
        dt: w.time.delta_secs().min(MAX_STEP),
        robot,
        scratch: &scratch,
        hunts: &w.herds.hunts,
    };
    for (mut beast, mut placed) in &mut w.beasts {
        let species = life.species.get(usize::from(beast.index));
        if let Some(species) = species.filter(|_| !matches!(beast.lead, Lead::Carcass { .. })) {
            step(&ctx, (&mut beast, &mut placed), species, &mut w.parts);
        }
    }
}

/// Fills the scratch with this frame's herds of the drawn animals.
fn gather(scratch: &mut Scratch, w: &mut Walk, land: Land) {
    let time = w.clock.seconds;
    scratch.herds.clear();
    for (beast, _) in &w.beasts {
        let Lead::Herd { id, .. } = beast.lead else {
            continue;
        };
        if !scratch.herds.iter().any(|(h, _)| *h == id) {
            scratch.herds.push((id, w.herds.herd(land, id, time)));
        }
    }
}

/// Returns where `beast` of `species` should go this frame, in its frame: its den when it has one.
fn aim(ctx: &Ctx, beast: &Beast, species: &Species) -> Aim {
    let radius = ctx.terrain.globe().radius();
    if let Some(den) = beast.wits.den {
        return Aim {
            place: beast.spot.local(den * radius),
            flow: Vec3::ZERO,
            excite: 0.5,
        };
    }
    let own = beast.lead.key().0;
    let herd = ctx
        .scratch
        .herds
        .iter()
        .find(|(id, _)| *id == own)
        .and_then(|(_, h)| h.as_ref());
    let still = Aim {
        place: beast.motion.root.at,
        flow: Vec3::ZERO,
        excite: 0.0,
    };
    goal(&beast.lead, herd, (ctx.hunts, species), ctx.time, radius).map_or(still, |g| Aim {
        place: beast.spot.local(g.dir * radius),
        flow: (beast.spot.rot.inverse() * g.flow).as_vec3(),
        excite: g.excite,
    })
}

/// Returns the pace of `beast` of `species`: full detail near the local robot.
fn pace(ctx: &Ctx, beast: &Beast, species: &Species) -> Pace {
    let here = beast.spot.planet(beast.motion.root.at);
    let near = here.distance(ctx.robot) < f64::from(NEAR);
    let gravity = ctx.terrain.globe().gravity() as f32;
    let body = &species.body;
    let gait = match body.locomotion {
        Locomotion::Walk => (FROUDE_TOP * gravity * body.height.max(0.05)).sqrt(),
        _ => f32::MAX,
    };
    Pace {
        gravity,
        top: species.history.speed.min(gait) * beast.wits.health.pace(),
        detail: if near { Detail::Full } else { Detail::Cheap },
    }
}

/// Steps one animal of `species` and poses its parts.
fn step(
    ctx: &Ctx,
    (beast, placed): (&mut Beast, &mut Placed),
    species: &Species,
    parts: &mut Parts,
) {
    let aim = aim(ctx, beast, species);
    let sensed = beast.wits.focus.map(|f| (f.at, beast.wits.stance));
    let temper = beast.temper;
    beast.mood.feel(&temper, sensed, ctx.dt);
    let spot = beast.spot;
    let heights = &ctx.scratch.heights;
    let ground = |p: Vec2| spot.ground_by(ctx.terrain, p, |x, z| heights.mesh(ctx.terrain, x, z));
    let pace = pace(ctx, beast, species);
    let drive = drive(
        &beast.motion,
        &species.body,
        (&aim, &beast.mood),
        pace,
        &ground,
    );
    beast.motion.step(&species.body, &drive, &ground, ctx.dt);
    if beast.motion.root.at.xz().length() > DRIFT {
        refit(ctx, (beast, placed), species);
    }
    for (part, (at, turn, scale)) in beast.parts.iter().zip(beast.motion.pose(&species.body)) {
        if let Some(mut t) = part.and_then(|p| parts.get_mut(p).ok()) {
            t.set_if_neq(Transform {
                translation: at,
                rotation: turn,
                scale,
            });
        }
    }
}

/// Moves `beast` into a new flat frame under where it stands, facing the way it faced, with its
/// motion started again there.
fn refit(ctx: &Ctx, (beast, placed): (&mut Beast, &mut Placed), species: &Species) {
    let radius = ctx.terrain.globe().radius();
    let old = beast.spot;
    let here = old.planet(beast.motion.root.at);
    let ahead = old.rot * (beast.motion.root.turn * Vec3::NEG_Z).as_dvec3();
    let spot = super::place::Spot::at(here.normalize(), radius);
    let facing = (spot.rot.inverse() * ahead)
        .as_vec3()
        .with_y(0.0)
        .normalize_or(Vec3::NEG_Z);
    let aloft = matches!(species.body.locomotion, Locomotion::Fly | Locomotion::Swarm);
    let lift = if aloft { cruise(&species.body) } else { 0.0 };
    let at = Vec3::new(0.0, spot.ground(ctx.terrain, Vec2::ZERO) + lift, 0.0);
    let seed = beast.lead.key().0 ^ u64::from(beast.lead.key().1);
    beast.motion = Motion::new(&species.body, at, facing, seed);
    beast.mood.threat = spot.local(old.planet(beast.mood.threat));
    beast.spot = spot;
    *placed = Placed {
        pos: spot.origin,
        rot: spot.rot,
    };
}
