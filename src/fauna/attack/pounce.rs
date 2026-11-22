//! The attacks of the animals: one that is aware of the robot and fights or hunts it picks the attack
//! its body and its distance allow, winds up where the robot can see it, and lands a [`Blow`] when
//! the robot is still in reach as the blow arrives.

use bevy::ecs::system::SystemParam;
use bevy::math::{DVec3, Vec2, Vec3Swizzles};
use bevy::prelude::*;

use super::Dead;
use super::brace::hold;
use super::foe::{Foe, foe_of};
use super::force::{contact, joules, meets};
use super::kinds::Kind;
use super::ring::heard;
use super::timing::Attacking;
use super::tune::{SLACK, SPREAD, STRIKE};
use crate::audio::Impact;
use crate::avatar::Avatar;
use crate::avatar::blow::Blow;
use crate::fauna::Frozen;
use crate::fauna::beast::Beast;
use crate::fauna::senses::Stance;
use crate::fauna::senses::tune::ALARM;
use crate::fauna::tune::MAX_STEP;
use crate::life::Species;
use crate::player::Player;
use crate::space::Anchor;
use crate::world::Terrain;

/// The robots of the other players standing free on the planet.
type Remotes<'w, 's> =
    Query<'w, 's, (Entity, &'static Transform), (With<Avatar>, Without<ChildOf>, Without<Player>)>;

/// What the attacks read and write.
#[derive(SystemParam)]
pub struct Fight<'w, 's> {
    commands: Commands<'w, 's>,
    time: Res<'w, Time>,
    terrain: Res<'w, Terrain>,
    anchor: Res<'w, Anchor>,
    robot: Query<'w, 's, &'static Transform, With<Player>>,
    remotes: Remotes<'w, 's>,
    blows: MessageWriter<'w, Blow>,
    sounds: MessageWriter<'w, Impact>,
}

/// Returns how far the target lies in the horizontal plane of `beast` and the unit way it faces.
fn geometry(beast: &Beast, foe: &Foe) -> (Vec2, Vec2) {
    let facing = (beast.motion.root.turn * Vec3::NEG_Z).xz();
    (
        (foe.local - beast.motion.root.at).xz(),
        facing.normalize_or_zero(),
    )
}

/// Returns the height in metres over the ground at which an attack of `kind` of a body of `height`
/// lands.
fn height(kind: Kind, height: f32) -> f32 {
    match kind {
        Kind::Bite | Kind::Lunge => 1.2 * height,
        Kind::Swipe => height,
        Kind::Charge => 0.7 * height,
        Kind::Sweep | Kind::Trample => 0.2,
    }
}

/// Returns the blow of `attack` by an animal of `species` on `foe` under `gravity`. A dash arrives at
/// the speed its run so far allows, the other attacks at the speed their part is thrown at. The
/// energy and the momentum are those the robot takes as a free body: see [`joules`].
fn landing(attack: &Attacking, species: &Species, foe: &Foe, gravity: f32) -> Blow {
    let (kind, body) = (attack.kind, &species.body);
    let speed = if kind.dashes() {
        let run = species.history.speed * (attack.t - attack.wind).max(0.0);
        contact(kind, (species.history.speed, run), (body.length, gravity))
    } else {
        kind.reach(body) / STRIKE
    };
    let lift = (height(kind, body.height) - 1.0).clamp(-SPREAD, SPREAD);
    Blow {
        at: foe.render + foe.up * lift,
        joules: joules(kind, species.history.mass, speed),
        cut: kind.cuts(),
        dir: foe.away,
        momentum: meets(kind, species.history.mass) * speed,
    }
}

/// Starts the attack `beast` of `species` can make at `foe`, if it fights and one fits.
fn open(
    commands: &mut Commands,
    (entity, beast): (Entity, &Beast),
    species: &Species,
    foe: &Foe,
    gravity: f32,
) {
    let fights = matches!(beast.wits.stance, Stance::Attack | Stance::Threaten);
    let aware = beast
        .wits
        .focus
        .is_some_and(|f| f.aware >= ALARM && f.at.distance(foe.local) < 3.0);
    if !(fights && aware) {
        return;
    }
    let shape = geometry(beast, foe);
    let broken = beast.wits.broken;
    let fits = |k: &Kind| k.can(species, broken) && k.fits(&species.body, shape);
    if let Some(kind) = Kind::ORDER.into_iter().find(fits) {
        let begin = Attacking::begin(kind, species.body.length, gravity);
        commands.entity(entity).insert(begin);
    }
}

/// Returns true when `foe` is still within the reach of `kind` of `beast`.
fn reaches(kind: Kind, (beast, species): (&Beast, &Species), foe: &Foe) -> bool {
    let gap = (foe.local - beast.motion.root.at).xz();
    gap.length() <= kind.reach(&species.body) * SLACK
}

/// Moves the attack on `dt` seconds under `gravity`: a dash drives the body forward at its top speed,
/// through its velocity so that its legs run and the walk carries it, and the blow lands once.
fn advance(
    (beast, attack): (&mut Beast, &mut Attacking),
    species: &Species,
    (foe, local, dt, gravity): (&Foe, bool, f32, f32),
    (blows, sounds): (&mut MessageWriter<Blow>, &mut MessageWriter<Impact>),
) {
    attack.t += dt;
    if !attack.striking() {
        return;
    }
    if attack.kind.dashes() {
        let (_, facing) = geometry(beast, foe);
        let run = facing * species.history.speed;
        beast.motion.root.vel.at = Vec3::new(run.x, 0.0, run.y);
        beast.motion.root.vel.vel = Vec3::ZERO;
    }
    if !attack.struck && reaches(attack.kind, (beast, species), foe) {
        attack.struck = true;
        let blow = landing(attack, species, foe, gravity);
        sounds.write(heard(attack.kind, species, &blow));
        if local {
            blows.write(blow);
        }
    }
}

/// Returns the target `foe` of an attacking `beast`, and whether it is the local robot.
fn target(f: &Fight, beast: &Beast, (robot, here): (&Transform, DVec3)) -> (Foe, bool) {
    let aim = beast.wits.focus.map_or_else(
        || beast.spot.planet(beast.motion.root.at),
        |focus| beast.spot.planet(focus.at),
    );
    let (picked, pos, local) = f
        .remotes
        .iter()
        .map(|(_, t)| (t, f.anchor.to_frame(t.translation), false))
        .chain(std::iter::once((robot, here, true)))
        .min_by(|a, b| {
            a.1.distance_squared(aim)
                .total_cmp(&b.1.distance_squared(aim))
        })
        .unwrap_or((robot, here, true));
    (foe_of(&f.anchor, beast, (picked, pos)), local)
}

/// Filter of the animals that can act: not dead and not held still.
type Alive = (Without<Dead>, Without<Frozen>);

/// Lets every animal that fights the robot wind up, strike and recover. An animal a hitstop holds
/// waits with it, as its walk does.
pub fn attack(
    mut f: Fight,
    mut beasts: Query<(Entity, &mut Beast, Option<&mut Attacking>), Alive>,
) {
    let globe = f.terrain.globe().clone();
    let (Some(life), Ok(robot)) = (globe.life(), f.robot.single()) else {
        return;
    };
    let (gravity, dt) = (globe.gravity() as f32, f.time.delta_secs().min(MAX_STEP));
    let here = f.anchor.to_frame(robot.translation);
    for (entity, mut beast, going) in &mut beasts {
        let Some(sp) = life.species.get(usize::from(beast.index)) else {
            continue;
        };
        let length = sp.body.length;
        match going {
            Some(a) if a.over() => {
                f.commands.entity(entity).remove::<Attacking>();
                hold(&mut beast, None, length);
            }
            Some(mut a) => {
                let (foe, local) = target(&f, &beast, (robot, here));
                let args = (&foe, local, dt, gravity);
                let out = (&mut f.blows, &mut f.sounds);
                advance((&mut *beast, &mut *a), sp, args, out);
                hold(&mut beast, Some(&a), length);
            }
            None => {
                let foe = foe_of(&f.anchor, &beast, (robot, here));
                hold(&mut beast, None, length);
                open(&mut f.commands, (entity, &*beast), sp, &foe, gravity);
            }
        }
    }
}
