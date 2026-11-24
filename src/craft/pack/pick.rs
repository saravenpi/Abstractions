//! Picking stones up: the pick key has the robot strike the rock under the crosshair, within its
//! reach, and the blow breaks a piece off it. The stone is the one the rock looks like: a basalt
//! column gives basalt, a flint nodule flint. A pick takes at most
//! [`HANDFUL`] kilograms and what the robot can still carry; the rock keeps the rest, shrinks as it
//! is quarried and is gone when its mass is used up.

mod aim;
pub(crate) mod chop;
pub(crate) mod dig;

use bevy::prelude::*;

use self::aim::Aim;
use super::notice::Notice;
use super::tune::{
    FIST_JOULES, FIST_MASS, FIST_STIFFNESS, HANDFUL, PICK_KEY, PIECE_SPEED, STONE_TONES,
};
use super::{Pack, Stack};
use crate::audio::{Body, Impact, Outer};
use crate::craft::build::{Intent, Place, swing};
use crate::craft::matter::{Mineral, Stuff};
use crate::craft::wield::Held;
use crate::effects::Fling;
use crate::player::{Interact, Verb};
use crate::space::{Anchor, Location, up};
use crate::world::Terrain;
use crate::world::rocks::tune::QUARRY_MAX;
use crate::world::rocks::{Rock, RockId, RockTaken, Taken, mineral_of};
use crate::world::trees::Tree;

/// Non-rock harvest target for the knap swing animation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HarvestTarget {
    /// Standing choppable tree.
    Tree(Tree),
    /// Terrain surface to dig into an underground vein.
    Ground,
}

/// Request to swing at a tree or ground patch under the crosshair.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct Forage {
    /// Whether chopping a tree or digging terrain.
    pub target: HarvestTarget,
    /// Hit point in render space.
    pub at: Vec3,
}

/// Emitted when the [`Forage`] swing lands on target.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct Foraged {
    /// Whether chopping a tree or digging terrain.
    pub target: HarvestTarget,
    /// Hit point in render space.
    pub at: Vec3,
}

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

/// Multiplier for rock breaking and digging: `Intent::Crush` (pickaxe/hammer) boosts speed and yield.
pub(super) fn crush_scale(held: &Held) -> f32 {
    match held.0.as_ref().map(|h| h.intent) {
        Some(Intent::Crush) => 2.5,
        Some(_) => 1.25,
        None => 1.0,
    }
}

/// Breaks a scaled piece off `rock`, stowing up to `room` kg and recording the mass removed.
pub(super) fn take_scaled(
    rock: Option<Rock>,
    room: f32,
    scale: f32,
    (pack, taken): (&mut Pack, &mut Taken),
) -> Option<(RockId, f32)> {
    let rock = rock?;
    let stone = stone_of(&rock)?;
    let material = stone.material();
    let whole = taken
        .whole(rock.id)
        .unwrap_or_else(|| rock.mass(material.density).min(QUARRY_MAX));
    let remaining = whole - taken.kilograms(rock.id);
    let piece = (HANDFUL * scale).min(remaining);
    let stack = Stack {
        stuff: Stuff::Mineral(stone),
        material,
        mass: piece,
    };
    let unit = stack.stuff.unit_mass();
    let kept = pack.stow(stack, room);
    (kept > 0.0).then_some(())?;
    let removed = if remaining - kept < unit && room >= remaining {
        remaining
    } else {
        kept
    };
    taken.quarry(rock.id, removed, whole);
    Some((rock.id, removed))
}

/// Breaks a standard handful off `rock`, stowing up to `room` kilograms.
pub(super) fn take(
    rock: Option<Rock>,
    room: f32,
    tools: (&mut Pack, &mut Taken),
) -> Option<(RockId, f32)> {
    take_scaled(rock, room, 1.0, tools)
}

/// Offers to harvest the rock, tree, or terrain under the crosshair on [`PICK_KEY`].
pub(crate) fn gather(
    (keys, aim): (Res<ButtonInput<KeyCode>>, Aim),
    mut interact: ResMut<Interact>,
    (mut knocks, mut forages): (MessageWriter<Knock>, MessageWriter<Forage>),
) {
    if let Some((rock, at)) = aim.rock().filter(|(rock, _)| stone_of(rock).is_some()) {
        interact.offer(Verb::Stone, "rock");
        if keys.just_pressed(PICK_KEY) {
            knocks.write(Knock { rock, at });
        }
    } else if let Some((tree, at)) = aim.tree() {
        interact.offer(Verb::Stone, "tree");
        if keys.just_pressed(PICK_KEY) {
            forages.write(Forage {
                target: HarvestTarget::Tree(tree),
                at,
            });
        }
    } else if let Some(at) = aim.ground() {
        interact.offer(Verb::Stone, "ground");
        if keys.just_pressed(PICK_KEY) {
            forages.write(Forage {
                target: HarvestTarget::Ground,
                at,
            });
        }
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

/// The ground, the place, the held tool and the anchor a split reads.
type Scene<'w> = (
    Res<'w, Terrain>,
    Res<'w, Location>,
    Res<'w, Held>,
    Res<'w, Anchor>,
);

/// Breaks the piece off every rock a blow reached and stows it: the rock is noted smaller for
/// everyone, the stone cracks where it was struck, the piece and five chips fly off it, and the
/// prompt line says what was picked up.
pub(crate) fn split(
    mut knocked: MessageReader<Knocked>,
    (terrain, location, held, anchor): Scene,
    (mut pack, mut taken): (ResMut<Pack>, ResMut<Taken>),
    (mut told, mut sounds, mut flings, mut notice): Told,
) {
    let globe = terrain.globe();
    let place = Place::on(globe.gravity(), globe.record());
    let scale = crush_scale(&held);
    for blow in knocked.read() {
        let Some(stone) = stone_of(&blow.rock) else {
            continue;
        };
        let stuff = Stuff::Mineral(stone);
        let room = pack.room_for(&stuff) as f32 * stuff.unit_mass();
        let tools = (&mut *pack, &mut *taken);
        let Some((id, kilograms)) = take_scaled(Some(blow.rock), room, scale, tools) else {
            continue;
        };
        told.write(RockTaken {
            planet: location.body,
            id,
            kilograms,
        });
        *notice = Notice::of(&stuff, kilograms);
        sounds.write(crack(stone, blow, striker(&held, &place)));
        let normal = up(&anchor, blow.at);
        flings.write(piece(stone, blow, (kilograms, normal)));
        for chip in chips(stone, blow, (kilograms, normal)) {
            flings.write(chip);
        }
    }
}
