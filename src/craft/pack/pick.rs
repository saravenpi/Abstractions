//! Picking stones up: the pick key has the robot strike the rock under the crosshair, within its
//! reach, and the blow breaks a piece off it. The stone is the one the rock looks like: a basalt
//! column gives basalt, a flint nodule flint. A pick takes at most
//! [`HANDFUL`] kilograms and what the robot can still carry; the rock keeps the rest, shrinks as it
//! is quarried and is gone when its mass is used up.

mod aim;

use bevy::prelude::*;

use self::aim::Aim;
use super::notice::Notice;
use super::tune::{
    FIST_JOULES, FIST_MASS, FIST_STIFFNESS, HANDFUL, PICK_KEY, PIECE_SPEED, STONE_TONES,
};
use super::{Burden, Pack, Stack};
use crate::audio::{Body, Impact, Outer};
use crate::craft::build::{Place, swing};
use crate::craft::matter::{Mineral, Stuff};
use crate::craft::wield::Held;
use crate::effects::Fling;
use crate::player::{Interact, Verb};
use crate::space::{Anchor, Location, up};
use crate::world::Terrain;
use crate::world::rocks::tune::QUARRY_MAX;
use crate::world::rocks::{Rock, RockId, RockTaken, Taken, mineral_of};

/// The robot asks to strike `rock` at `at`, a point in render space, to break a piece off it.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct Knock {
    /// The rock to strike.
    pub rock: Rock,
    /// Where the crosshair meets it, in render space.
    pub at: Vec3,
}

/// The blow a [`Knock`] asked for reached the rock: the piece comes away now.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct Knocked {
    /// The rock struck.
    pub rock: Rock,
    /// Where the blow reached it, in render space.
    pub at: Vec3,
}

/// Returns the stone the rock `rock` is made of, the one its look shows, or nothing for a rock that
/// gives no stone to knap.
pub fn stone_of(rock: &Rock) -> Option<Mineral> {
    mineral_of(rock.plant)
}

/// Breaks a piece off `rock`, when there is one, and stows it: at most [`HANDFUL`] kilograms, at most
/// what is left of the rock, at most `room` kilograms. The piece is noted as taken from the rock, and
/// its mass and the rock's id are returned when the robot carried off any. A robot with no room
/// leaves the rock as it is.
pub(super) fn take(
    rock: Option<Rock>,
    room: f32,
    (pack, taken): (&mut Pack, &mut Taken),
) -> Option<(RockId, f32)> {
    let rock = rock?;
    let stone = stone_of(&rock)?;
    let material = stone.material();
    let whole = taken
        .whole(rock.id)
        .unwrap_or_else(|| rock.mass(material.density).min(QUARRY_MAX));
    let piece = HANDFUL.min(whole - taken.kilograms(rock.id));
    let stack = Stack {
        stuff: Stuff::Mineral(stone),
        material,
        mass: piece,
    };
    let kept = pack.stow(stack, room);
    (kept > 0.0).then_some(())?;
    taken.quarry(rock.id, kept, whole);
    Some((rock.id, kept))
}

/// Offers to break a piece off the rock under the crosshair, and asks the robot to strike it when
/// the pick key goes down: the piece comes away when the blow reaches the rock, in [`split`].
pub(crate) fn gather(
    (keys, aim): (Res<ButtonInput<KeyCode>>, Aim),
    mut interact: ResMut<Interact>,
    mut knocks: MessageWriter<Knock>,
) {
    let found = aim.rock().filter(|(rock, _)| stone_of(rock).is_some());
    let Some((rock, at)) = found else {
        return;
    };
    interact.offer(Verb::Stone, "");
    if keys.just_pressed(PICK_KEY) {
        knocks.write(Knock { rock, at });
    }
}

/// Returns the head that struck a rock and the energy of its blow on a world of `place`: the head
/// of the held tool swung at full speed, or the robot's fist.
pub(super) fn striker(held: &Held, place: &Place) -> (Body, f32) {
    let fist = Body {
        stiffness: FIST_STIFFNESS,
        mass: FIST_MASS,
    };
    held.0.as_ref().map_or((fist, FIST_JOULES), |hold| {
        let head = Body {
            stiffness: hold.build.head.material.stiffness,
            mass: hold.build.head.mass,
        };
        (head, swing(&hold.build, place).energy)
    })
}

/// Returns the crack of `stone` struck by `tool` with `joules` where the blow `knocked` reached it.
pub(super) fn crack(stone: Mineral, knocked: &Knocked, (tool, joules): (Body, f32)) -> Impact {
    let material = stone.material();
    Impact {
        at: knocked.at,
        joules,
        cut: false,
        outer: Outer::Ground,
        struck: Body {
            stiffness: material.stiffness,
            mass: knocked.rock.mass(material.density),
        },
        tool,
        deflected: false,
    }
}

/// Returns the piece of `kilograms` of `stone` broken off where the blow `knocked` reached it,
/// flying off the rock along `up`: a sphere of its own mass, the very stone the robot picks up.
pub(super) fn piece(stone: Mineral, knocked: &Knocked, (kilograms, up): (f32, Vec3)) -> Fling {
    let volume = kilograms / stone.material().density;
    let radius = (3.0 * volume / (4.0 * std::f32::consts::PI)).cbrt();
    Fling {
        at: knocked.at,
        velocity: up * PIECE_SPEED,
        radius,
        colour: STONE_TONES[stone as usize],
        wet: false,
        tag: None,
    }
}

/// Returns five angular chips fanned outward around `up` when a piece breaks off `stone`.
pub(super) fn chips(stone: Mineral, knocked: &Knocked, (kilograms, up): (f32, Vec3)) -> [Fling; 5] {
    let volume = kilograms / stone.material().density;
    let base = (3.0 * volume / (4.0 * std::f32::consts::PI)).cbrt();
    let (u, v) = up.any_orthonormal_pair();
    let tone = STONE_TONES[stone as usize];
    std::array::from_fn(|k| {
        let angle = k as f32 * std::f32::consts::TAU / 5.0 + 0.35;
        let out = u * angle.cos() + v * angle.sin();
        let dir = (up * (0.9 + 0.15 * (k % 2) as f32) + out * 0.85).normalize_or(up);
        let shade = 0.82 + 0.1 * (k as f32);
        Fling {
            at: knocked.at,
            velocity: dir * PIECE_SPEED * (1.15 + 0.16 * k as f32),
            radius: base * (0.35 + 0.04 * (k % 4) as f32),
            colour: tone.map(|c| c * shade),
            wet: false,
            tag: None,
        }
    })
}

/// What a broken piece is written to: the rock, the sound, the flecks and the notice.
type Told<'w> = (
    MessageWriter<'w, RockTaken>,
    MessageWriter<'w, Impact>,
    MessageWriter<'w, Fling>,
    ResMut<'w, Notice>,
);

/// The ground, the load, the place, the held tool and the anchor a split reads.
type Scene<'w> = (
    Res<'w, Terrain>,
    Res<'w, Burden>,
    Res<'w, Location>,
    Res<'w, Held>,
    Res<'w, Anchor>,
);

/// Breaks the piece off every rock a blow reached and stows it: the rock is noted smaller for
/// everyone, the stone cracks where it was struck, the piece and five chips fly off it, and the
/// prompt line says what was picked up.
pub(crate) fn split(
    mut knocked: MessageReader<Knocked>,
    (terrain, burden, location, held, anchor): Scene,
    (mut pack, mut taken): (ResMut<Pack>, ResMut<Taken>),
    (mut told, mut sounds, mut flings, mut notice): Told,
) {
    let globe = terrain.globe();
    let place = Place::on(globe.gravity(), globe.record());
    for blow in knocked.read() {
        let Some(stone) = stone_of(&blow.rock) else {
            continue;
        };
        let tools = (&mut *pack, &mut *taken);
        let Some((id, kilograms)) = take(Some(blow.rock), burden.room(), tools) else {
            continue;
        };
        told.write(RockTaken {
            planet: location.body,
            id,
            kilograms,
        });
        *notice = Notice::of(&Stuff::Mineral(stone), kilograms);
        sounds.write(crack(stone, blow, striker(&held, &place)));
        let normal = up(&anchor, blow.at);
        flings.write(piece(stone, blow, (kilograms, normal)));
        for chip in chips(stone, blow, (kilograms, normal)) {
            flings.write(chip);
        }
    }
}
