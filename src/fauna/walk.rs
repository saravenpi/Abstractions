//! The walk: every frame, each drawn animal heads for its place at universe time, or its den when
//! wounded, acts on the stance its senses set, steps its motion on the drawn ground and poses its
//! parts.

use bevy::ecs::system::SystemParam;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::herds::{Herds, Land};
use super::lead::goal;
use super::motion::Detail;
use super::senses::tune::ROBOT_KG;
use super::steer::{Aim, Pace, drive};
use super::sync::RemoteFauna;
use super::tune::{DRIFT, MAX_STEP, NEAR, STAND_OFF, STRIKE_SHARE};
use super::{
    CaveBound, Frozen, Kind,
    beast::{Beast, Part},
    gate::Spec,
    ground::Heights,
    want::Lead,
};
use crate::avatar::Avatar;
use crate::life::{Species, herd::Herd, hunt::Hunt};
use crate::player::Player;
use crate::space::{Anchor, Clock, Placed};
use crate::world::{Chunks, Terrain};

/// What the walk reads and writes.
#[derive(SystemParam)]
pub struct Walk<'w, 's> {
    clock: Res<'w, Clock>,
    time: Res<'w, Time>,
    terrain: Res<'w, Terrain>,
    chunks: Option<Res<'w, Chunks>>,
    anchor: Res<'w, Anchor>,
    spec: Res<'w, Spec>,
    herds: ResMut<'w, Herds>,
    remote: Option<Res<'w, RemoteFauna>>,
    robot: Query<'w, 's, &'static Transform, With<Player>>,
    remotes: Remotes<'w, 's>,
    beasts: Beasts<'w, 's>,
    parts: Parts<'w, 's>,
}

/// Drawn unfrozen beasts with their optional cave bounds.
type Beasts<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Beast,
        &'static mut Placed,
        Option<&'static CaveBound>,
    ),
    Without<Frozen>,
>;

/// The robots of the other players standing free on the planet.
type Remotes<'w, 's> =
    Query<'w, 's, &'static Transform, (With<Avatar>, Without<ChildOf>, Without<Player>)>;

/// The drawn parts of the animals.
type Parts<'w, 's> =
    Query<'w, 's, &'static mut Transform, (With<Part>, Without<Player>, Without<Avatar>)>;

/// What the walk keeps from frame to frame, so it never allocates once warm.
#[derive(Default)]
pub struct Scratch {
    /// Herds of the drawn animals at this frame's time, by identity.
    herds: Vec<(u64, Option<Herd>)>,
    /// Heights of the drawn ground read so far.
    heights: Heights,
    /// Planet positions of the local and remote robots this frame.
    robots: Vec<DVec3>,
}

/// What every animal reads this frame.
struct Ctx<'a> {
    terrain: &'a Terrain,
    chunks: Option<&'a Chunks>,
    time: f64,
    dt: f32,
    robots: &'a [DVec3],
    scratch: &'a Scratch,
    hunts: &'a [Hunt],
    remote: Option<&'a RemoteFauna>,
}

/// Steps every drawn animal and poses its parts.
pub fn walk(mut w: Walk, mut scratch: Local<Scratch>) {
    let globe = w.terrain.globe().clone();
    let (Some(life), Some(planet), Ok(robot)) =
        (globe.life(), w.spec.planet.clone(), w.robot.single())
    else {
        return;
    };
    let at = w.anchor.to_frame(robot.translation);
    let land = Land {
        globe: &globe,
        life,
        planet: &planet,
    };
    gather(&mut scratch, &mut w, land, at);
    let ctx = Ctx {
        terrain: &w.terrain,
        chunks: w.chunks.as_deref(),
        time: w.clock.seconds,
        dt: w.time.delta_secs().min(MAX_STEP),
        robots: &scratch.robots,
        scratch: &scratch,
        hunts: &w.herds.hunts,
        remote: w.remote.as_deref(),
    };
    for (mut beast, mut placed, cave) in &mut w.beasts {
        let species = life.species.get(usize::from(beast.index));
        if let Some(species) = species.filter(|_| !matches!(beast.lead, Lead::Carcass { .. })) {
            step(&ctx, (&mut beast, &mut placed, cave), species, &mut w.parts);
        }
    }
}

/// Readies the scratch for this frame: fills it with the robots and herds of the drawn animals, forgets
/// which grid each chunk is drawn on, and every height read when the terrain changed.
fn gather(scratch: &mut Scratch, w: &mut Walk, land: Land, robot: DVec3) {
    if w.terrain.is_changed() {
        scratch.heights.clear();
    }
    scratch.heights.regrid();
    scratch.robots.clear();
    scratch.robots.push(robot);
    scratch
        .robots
        .extend(w.remotes.iter().map(|t| w.anchor.to_frame(t.translation)));
    let time = w.clock.seconds;
    scratch.herds.clear();
    for (beast, _, _) in &w.beasts {
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
    let find = |h: &&(u64, Option<Herd>)| h.0 == own;
    let herd = ctx
        .scratch
        .herds
        .iter()
        .find(find)
        .and_then(|h| h.1.as_ref());
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

/// Returns the pace of `beast` of `species`: full detail near the nearest robot.
fn pace(ctx: &Ctx, beast: &Beast, species: &Species) -> Pace {
    let here = beast.spot.planet(beast.motion.root.at);
    let dist = |a: &DVec3, b: &DVec3| {
        a.distance_squared(here)
            .total_cmp(&b.distance_squared(here))
    };
    let nearest = ctx.robots.iter().copied().min_by(dist).unwrap_or(here);
    let near = here.distance(nearest) < f64::from(NEAR + species.body.length);
    let globe = ctx.terrain.globe();
    let level = globe.radius() + f64::from(globe.sea_level());
    let rise = beast.spot.rise(beast.motion.root.at.xz(), level);
    let sea = [f32::NEG_INFINITY, rise][usize::from(globe.has_sea())];
    let body = &species.body;
    Pace {
        gravity: globe.gravity() as f32,
        top: species.history.speed * beast.wits.health.pace() * beast.wits.broken.pace(body),
        detail: if near { Detail::Full } else { Detail::Cheap },
        strike: strike(beast, species),
        sea,
        robot: (beast.spot.local(nearest), species.history.mass < ROBOT_KG),
    }
}

/// Returns the metres from its root within which `beast` of `species` strikes what it goes for:
/// [`STRIKE_SHARE`] of the reach of its longest attack that neither runs nor sweeps behind, or
/// [`STAND_OFF`] past its length when it has none.
fn strike(beast: &Beast, species: &Species) -> f32 {
    let near = [Kind::Trample, Kind::Swipe, Kind::Lunge, Kind::Bite];
    let reach = near
        .into_iter()
        .filter(|k| k.can(species, beast.wits.broken))
        .map(|k| k.reach(&species.body))
        .fold(0.0, f32::max);
    if reach > 0.0 {
        STRIKE_SHARE * reach
    } else {
        STAND_OFF + species.body.length
    }
}

/// Steps one animal of `species` and poses its parts.
fn step(
    ctx: &Ctx,
    (beast, placed, cave): (&mut Beast, &mut Placed, Option<&CaveBound>),
    species: &Species,
    parts: &mut Parts,
) {
    let pace = pace(ctx, beast, species);
    if let Some(target) = ctx.remote.and_then(|r| r.get(beast.lead.key())) {
        let src = (ctx.terrain, ctx.chunks, &ctx.scratch.heights);
        super::sync::follow(
            src,
            (beast, placed),
            (species, parts),
            (target, ctx.time, ctx.dt),
            pace,
        );
        return;
    }
    let aim = aim(ctx, beast, species);
    let sensed = beast.wits.focus.map(|f| (f.at, beast.wits.stance));
    let temper = beast.temper;
    beast.mood.feel(&temper, sensed, ctx.dt);
    let (spot, heights) = (beast.spot, &ctx.scratch.heights);
    let slant = spot.slant(ctx.terrain, beast.motion.root.at.xz());
    let drawn = |x: f32, z: f32| heights.drawn(ctx.terrain, ctx.chunks, x, z);
    let ground = |p: Vec2| {
        cave.map_or_else(
            || spot.ground_near(ctx.terrain, (&slant, p), drawn),
            |c| c.floor_y,
        )
    };
    let drive = drive(
        &beast.motion,
        &species.body,
        (&aim, &beast.mood),
        pace,
        &ground,
    );
    beast.motion.step(&species.body, &drive, &ground, ctx.dt);
    if let Some(c) = cave {
        let off = (beast.motion.root.at - c.center).xz();
        let lim = c.radius.max(0.5);
        if off.length() > lim {
            let cl = off.normalize_or_zero() * lim;
            (beast.motion.root.at.x, beast.motion.root.at.z) =
                (c.center.x + cl.x, c.center.z + cl.y);
        }
        beast.motion.root.at.y = c.floor_y;
    } else if beast.motion.root.at.xz().length() > DRIFT {
        super::sync::refit(ctx.terrain, (beast, placed), species);
    }
    for (part, (at, turn, scale)) in beast.parts.iter().zip(beast.motion.pose(&species.body)) {
        if let Some(mut t) = part.and_then(|p| parts.get_mut(p).ok()) {
            t.set_if_neq(
                Transform::from_translation(at)
                    .with_rotation(turn)
                    .with_scale(scale),
            );
        }
    }
}
