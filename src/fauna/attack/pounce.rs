//! The attacks of the animals: one that is aware of the robot and fights or hunts it picks the attack
//! its body and its distance allow, winds up where the robot can see it, and lands a [`Blow`] when
//! the robot is still in reach as the blow arrives.

use bevy::ecs::system::SystemParam;
use bevy::math::{Vec2, Vec3Swizzles};
use bevy::prelude::*;

use super::Dead;
use super::kinds::Kind;
use super::ring::heard;
use super::timing::{Attacking, joules};
use super::tune::{SLACK, SPREAD, STRIKE};
use crate::audio::Impact;
use crate::avatar::blow::Blow;
use crate::fauna::beast::Beast;
use crate::fauna::senses::Stance;
use crate::fauna::senses::tune::ALARM;
use crate::fauna::tune::MAX_STEP;
use crate::life::Species;
use crate::player::Player;
use crate::space::Anchor;
use crate::world::Terrain;

/// What the attacks read and write.
#[derive(SystemParam)]
pub struct Fight<'w, 's> {
    commands: Commands<'w, 's>,
    time: Res<'w, Time>,
    terrain: Res<'w, Terrain>,
    anchor: Res<'w, Anchor>,
    robot: Query<'w, 's, &'static Transform, With<Player>>,
    blows: MessageWriter<'w, Blow>,
    sounds: MessageWriter<'w, Impact>,
}

/// The robot an animal fights: its place in the frame of the animal and in render space.
#[derive(Clone, Copy)]
struct Foe {
    local: Vec3,
    render: Vec3,
    up: Vec3,
}

/// Returns how far the target lies in the horizontal plane of `beast` and the unit way it faces.
fn geometry(beast: &Beast, foe: &Foe) -> (Vec2, Vec2) {
    let facing = (beast.motion.root.turn * Vec3::NEG_Z).xz();
    (
        (foe.local - beast.motion.root.at).xz(),
        facing.normalize_or_zero(),
    )
}

/// Returns true when `beast` is aware enough of the robot, at `foe`, to attack it.
fn wants(beast: &Beast, foe: &Foe) -> bool {
    let fights = matches!(beast.wits.stance, Stance::Attack | Stance::Threaten);
    fights
        && beast
            .wits
            .focus
            .is_some_and(|f| f.aware >= ALARM && f.at.distance(foe.local) < 3.0)
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

/// Returns the blow of an attack of `kind` by an animal of `species` on `foe`.
fn landing(kind: Kind, species: &Species, foe: &Foe) -> Blow {
    let body = &species.body;
    let speed = if kind.dashes() {
        species.history.speed
    } else {
        kind.reach(body) / STRIKE
    };
    let lift = (height(kind, body.height) - 1.0).clamp(-SPREAD, SPREAD);
    Blow {
        at: foe.render + foe.up * lift,
        joules: joules(kind, species.history.mass, speed),
        cut: kind.cuts(),
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
    if !wants(beast, foe) {
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

/// Moves the attack on `dt` seconds: a dash carries the body forward, and the blow lands once.
fn advance(
    (beast, attack): (&mut Beast, &mut Attacking),
    species: &Species,
    (foe, dt): (&Foe, f32),
    (blows, sounds): (&mut MessageWriter<Blow>, &mut MessageWriter<Impact>),
) {
    attack.t += dt;
    if !attack.striking() {
        return;
    }
    if attack.kind.dashes() {
        let (_, facing) = geometry(beast, foe);
        let run = facing * species.history.speed * dt;
        beast.motion.root.at += Vec3::new(run.x, 0.0, run.y);
    }
    if !attack.struck && reaches(attack.kind, (beast, species), foe) {
        attack.struck = true;
        let blow = landing(attack.kind, species, foe);
        sounds.write(heard(attack.kind, species, &blow));
        blows.write(blow);
    }
}

/// Lets every animal that fights the robot wind up, strike and recover.
pub fn attack(
    mut f: Fight,
    mut beasts: Query<(Entity, &mut Beast, Option<&mut Attacking>), Without<Dead>>,
) {
    let globe = f.terrain.globe().clone();
    let (Some(life), Ok(robot)) = (globe.life(), f.robot.single()) else {
        return;
    };
    let (gravity, dt) = (globe.gravity() as f32, f.time.delta_secs().min(MAX_STEP));
    let here = f.anchor.to_frame(robot.translation);
    for (entity, mut beast, going) in &mut beasts {
        let Some(species) = life.species.get(usize::from(beast.index)) else {
            continue;
        };
        let foe = Foe {
            local: beast.spot.local(here),
            render: robot.translation,
            up: robot.up().as_vec3(),
        };
        match going {
            Some(a) if a.over() => {
                f.commands.entity(entity).remove::<Attacking>();
            }
            Some(mut a) => {
                let out = (&mut f.blows, &mut f.sounds);
                advance((&mut *beast, &mut *a), species, (&foe, dt), out);
            }
            None => open(&mut f.commands, (entity, &*beast), species, &foe, gravity),
        }
    }
}
