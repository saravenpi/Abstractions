//! Picking stones up: the pick key has the robot strike the rock under the crosshair, within its
//! reach, and the blow breaks a piece off it. The stone is one of the stones of the planet, chosen
//! by the id of the rock, so the same rock always gives the same one. A pick takes at most
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
use crate::universe::PlanetRecord;
use crate::world::Terrain;
use crate::world::rocks::{Rock, RockId, RockTaken, Taken};

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

/// Returns the stones that lie on a planet of `record`.
pub fn stones(record: &PlanetRecord) -> Vec<Mineral> {
    Mineral::ALL
        .into_iter()
        .filter(|m| m.lies_on(record))
        .collect()
}

/// Returns the stone of the planet of `record` that the rock `id` is made of, or nothing when no
/// stone lies there.
pub fn stone_of(record: &PlanetRecord, id: RockId) -> Option<Mineral> {
    let found = stones(record);
    found.get(id.0 as usize % found.len().max(1)).copied()
}

/// Breaks a piece off `rock`, when there is one, and stows it: at most [`HANDFUL`] kilograms, at most
/// what is left of the rock, at most `room` kilograms. The piece is noted as taken from the rock, and
/// its mass and the rock's id are returned when the robot carried off any. A robot with no room
/// leaves the rock as it is.
pub(super) fn take(
    rock: Option<Rock>,
    (record, room): (&PlanetRecord, f32),
    (pack, taken): (&mut Pack, &mut Taken),
) -> Option<(RockId, f32)> {
    let rock = rock?;
    let stone = stone_of(record, rock.id)?;
    let material = stone.material();
    let whole = taken
        .whole(rock.id)
        .unwrap_or_else(|| rock.mass(material.density));
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
    terrain: Res<Terrain>,
    mut interact: ResMut<Interact>,
    mut knocks: MessageWriter<Knock>,
) {
    let record = terrain.globe().record();
    let found = aim
        .rock()
        .filter(|(rock, _)| stone_of(record, rock.id).is_some());
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

/// What a broken piece is written to: the rock, the sound, the flecks and the notice.
type Told<'w> = (
    MessageWriter<'w, RockTaken>,
    MessageWriter<'w, Impact>,
    MessageWriter<'w, Fling>,
    ResMut<'w, Notice>,
);

/// Breaks the piece off every rock a blow reached and stows it: the rock is noted smaller for
/// everyone, the stone cracks where it was struck, the piece flies off it, and the prompt line
/// says what was picked up.
pub(crate) fn split(
    mut knocked: MessageReader<Knocked>,
    (terrain, burden, location, held, anchor): (
        Res<Terrain>,
        Res<Burden>,
        Res<Location>,
        Res<Held>,
        Res<Anchor>,
    ),
    (mut pack, mut taken): (ResMut<Pack>, ResMut<Taken>),
    (mut told, mut sounds, mut flings, mut notice): Told,
) {
    let globe = terrain.globe();
    let place = Place::on(globe.gravity(), globe.record());
    for blow in knocked.read() {
        let Some(stone) = stone_of(globe.record(), blow.rock.id) else {
            continue;
        };
        let tools = (&mut *pack, &mut *taken);
        let Some((id, kilograms)) = take(Some(blow.rock), (globe.record(), burden.room()), tools)
        else {
            continue;
        };
        told.write(RockTaken {
            planet: location.body,
            id,
            kilograms,
        });
        *notice = Notice::of(&Stuff::Mineral(stone), kilograms);
        sounds.write(crack(stone, blow, striker(&held, &place)));
        flings.write(piece(stone, blow, (kilograms, up(&anchor, blow.at))));
    }
}
