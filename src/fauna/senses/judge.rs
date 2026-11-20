//! The judgment of one animal: its senses this frame, its stance by fight or flight, its herd's
//! alarm, and, when wounded, its way to the den and its blood.

use bevy::math::{DVec3, Vec2, Vec3, Vec3Swizzles};

use super::feel::{Air, Senser, Threat, nose, perceive};
use super::stance::{Foe, Me, choose};
use super::tune::{ALARM, BLOOD_PER_KG, CORNER_PROBE, CORNER_RISE, LOOKOUT};
use super::{Blood, Drip, Focus, Stance};
use crate::fauna::beast::Beast;
use crate::fauna::ground::Heights;
use crate::life::Species;
use crate::life::herd::{Herd, HerdState, lookout};
use crate::life::signs::den;
use crate::life::signs::tune::NEST_CARE;
use crate::world::Terrain;

/// What every animal reads this frame.
pub struct Ctx<'a> {
    /// The drawn terrain.
    pub terrain: &'a Terrain,
    /// The herds at the last scan.
    pub herds: &'a [Herd],
    /// Universe time in seconds.
    pub time: f64,
    /// Seconds of this frame.
    pub dt: f32,
    /// Wind velocity in planet axes, metres per second.
    pub wind: DVec3,
    /// True in the grace after a landing.
    pub grace: bool,
    /// The threats of this frame.
    pub threats: &'a [Threat],
    /// Herds whose lookout fled last frame, with the planet position of what it fled.
    pub alarms: &'a [(u64, DVec3)],
    /// Heights of the drawn ground.
    pub heights: &'a Heights,
}

/// Returns what an animal of `species` in `herd` at `slot` weighs about itself, `health` left.
fn me(species: &Species, (herd, slot): (Option<&Herd>, u8), health: f32, grace: bool) -> Me {
    let rested = |h: &Herd| {
        matches!(
            h.state,
            HerdState::Feed | HerdState::Rest | HerdState::Sleep
        )
    };
    let tends = species.history.care >= NEST_CARE;
    Me {
        mass: species.history.mass,
        health,
        guild: species.role.guild,
        hungry: !herd.is_some_and(rested),
        cornered: false,
        guarding: tends && herd.is_some_and(|h| h.young > 0 && slot >= h.young),
        grace,
    }
}

/// Returns true when the ground of `ground` rises steeply on the way from `threat` past `at`, for an
/// animal `height` metres tall.
fn cornered(ground: &dyn Fn(Vec2) -> f32, at: Vec3, threat: Vec3, height: f32) -> bool {
    let away = (at - threat).xz().normalize_or(Vec2::X);
    let rise = ground(at.xz() + away * CORNER_PROBE) - ground(at.xz());
    rise > CORNER_RISE * height.max(0.3)
}

/// Returns `stance` and `focus`, turned to flight from what the lookout of herd `own` fled last frame
/// when the animal itself is calm or wary.
fn alarmed(
    ctx: &Ctx,
    own: u64,
    beast: &Beast,
    (stance, focus): (Stance, Option<Focus>),
) -> (Stance, Option<Focus>) {
    let calm = matches!(stance, Stance::Calm | Stance::Wary);
    match ctx
        .alarms
        .iter()
        .find(|(id, _)| *id == own)
        .filter(|_| calm)
    {
        Some((_, pos)) => {
            let at = beast.spot.local(*pos);
            let aware = focus.map_or(ALARM, |f| f.aware.max(ALARM));
            (Stance::Flee, Some(Focus { at, aware }))
        }
        None => (stance, focus),
    }
}

/// Returns how `beast` of `species` senses, its senses keener when it is its herd's `watch`.
fn senser(beast: &Beast, species: &Species, watch: bool) -> Senser {
    Senser {
        spot: beast.spot,
        at: beast.motion.root.at,
        eye: species.body.height,
        keen: if watch { LOOKOUT } else { 1.0 },
        nose: nose(species.role.guild),
        herd: beast.lead.key().0,
    }
}

/// Returns the stance and focus of `senser`, weighing up as `me`, `height` metres tall, the threat of
/// `ctx` it is most aware of by `best`, over `ground`.
fn weigh(
    ctx: &Ctx,
    (senser, me, height): (&Senser, Me, f32),
    best: Option<(usize, f32)>,
    ground: &dyn Fn(Vec2) -> f32,
) -> (Stance, Option<Focus>) {
    let Some((index, aware)) = best else {
        return (Stance::Calm, None);
    };
    let threat = &ctx.threats[index];
    let at = senser.spot.local(threat.pos);
    let cornered = aware >= ALARM && cornered(ground, senser.at, at, height);
    let foe = Foe {
        mass: threat.mass,
        robot: threat.robot,
    };
    let stance = choose(aware, &Me { cornered, ..me }, &foe);
    (stance, Some(Focus { at, aware }))
}

/// Senses the threats of `beast` of `species` and sets its stance, adding its herd to `raised` when
/// it is the lookout and flees, and its blood to `blood` when wounded.
pub fn judge(
    ctx: &Ctx,
    (beast, species): (&mut Beast, &Species),
    (raised, blood): (&mut Vec<(u64, DVec3)>, &mut Blood),
) {
    let (own, slot) = beast.lead.key();
    let herd = ctx.herds.iter().find(|h| h.id == own);
    let watch = herd.is_some_and(|h| lookout(h) == slot);
    let spot = beast.spot;
    let ground =
        |p: Vec2| spot.ground_by(ctx.terrain, p, |x, z| ctx.heights.mesh(ctx.terrain, x, z));
    let senser = senser(beast, species, watch);
    let air = Air {
        wind: ctx.wind,
        ground: &ground,
        dt: ctx.dt,
    };
    let best = perceive(&mut beast.wits.meter, &senser, ctx.threats, &air);
    let me = me(species, (herd, slot), beast.wits.health.share(), ctx.grace);
    let weighed = weigh(ctx, (&senser, me, species.body.height), best, &ground);
    let (stance, focus) = alarmed(ctx, own, beast, weighed);
    if let Some(f) = focus.filter(|_| watch && stance == Stance::Flee) {
        raised.push((own, spot.planet(f.at)));
    }
    (beast.wits.stance, beast.wits.focus) = (stance, focus);
    wound(ctx, (beast, species), herd, blood);
}

/// Points a wounded `beast` of `species` at the den of `herd` nearest it, or the herd's home, and
/// lets it bleed; an unhurt one keeps no den.
fn wound(
    ctx: &Ctx,
    (beast, species): (&mut Beast, &Species),
    herd: Option<&Herd>,
    blood: &mut Blood,
) {
    if !beast.wits.health.wounded() {
        beast.wits.den = None;
        return;
    }
    let radius = ctx.terrain.globe().radius();
    let here = beast.spot.planet(beast.motion.root.at);
    let dir = here.normalize();
    beast.wits.den = herd.map(|h| den(h, species, radius, dir).unwrap_or(h.home));
    let ahead = (beast.motion.root.turn * Vec3::NEG_Z)
        .with_y(0.0)
        .normalize_or(Vec3::NEG_Z);
    let drip = Drip {
        at: dir,
        facing: beast.spot.rot * ahead.as_dvec3(),
        species: beast.index,
        size: BLOOD_PER_KG * species.history.mass.cbrt(),
        time: ctx.time,
    };
    beast.wits.bled = Some(blood.bleed(drip, beast.wits.bled, radius));
}
