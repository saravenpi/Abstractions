//! The telegraphs of the attacks and the flinch of a hit part, drawn over the walk: the parts that
//! carry an attack are drawn back as it winds up and thrown out as it lands, so a bigger body gives
//! a longer warning, and a part that was hit is pushed along the blow and springs back.

use bevy::ecs::query::QueryData;
use bevy::prelude::*;

use super::kinds::Kind;
use super::timing::Attacking;
use super::tune::FLINCH;
use crate::fauna::beast::{Beast, Part};
use crate::life::Species;
use crate::life::body::{Body, PartKind};
use crate::world::Terrain;

/// A part that was hit: pushed by `push` in the frame of the animal, springing back over [`FLINCH`]
/// seconds.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Flinch {
    /// Index of the part hit.
    pub part: u8,
    /// Where the blow pushed it at first, in metres.
    pub push: Vec3,
    /// Seconds since the blow.
    pub age: f32,
}

/// Returns true when part `index` of `body` is on a front leg.
fn front(body: &Body, index: usize) -> bool {
    let at = |i: &u8| usize::from(*i) == index;
    body.legs
        .iter()
        .any(|l| l.rank == 0 && l.bones.iter().any(at))
}

/// Returns the way a part of `kind` moves, in body lengths per unit of swing, when the body makes an
/// attack of `attack`: +X right, +Y up, -Z ahead. Parts that do not carry it stay put.
pub fn aim(attack: Kind, part: PartKind, front: bool) -> Vec3 {
    let head = matches!(
        part,
        PartKind::Head | PartKind::Jaw | PartKind::Eye | PartKind::Antenna | PartKind::Horn
    );
    let leg = front && matches!(part, PartKind::Thigh | PartKind::Shin | PartKind::Foot);
    match attack {
        Kind::Bite if head => Vec3::new(0.0, 0.1, -0.3),
        Kind::Lunge if head => Vec3::new(0.0, 0.2, -1.0),
        Kind::Lunge if part == PartKind::Neck => Vec3::new(0.0, 0.1, -0.5),
        Kind::Swipe if leg => Vec3::new(0.0, -0.3, -0.8),
        Kind::Sweep if part == PartKind::Tail => Vec3::X,
        Kind::Charge if head || part == PartKind::Neck => Vec3::new(0.0, 0.3, -0.4),
        Kind::Trample if leg || head || part == PartKind::Neck => Vec3::new(0.0, -1.0, -0.3),
        _ => Vec3::ZERO,
    }
}

/// What the gestures read of an animal: its place, its attack and its flinch.
#[derive(QueryData)]
#[query_data(mutable)]
pub struct Gesturing {
    entity: Entity,
    beast: &'static Beast,
    going: Option<&'static Attacking>,
    flinch: Option<&'static mut Flinch>,
}

/// The animals that make an attack or flinch.
type Busy = Or<(With<Attacking>, With<Flinch>)>;

/// The drawn parts, the ones the gestures move.
type Limbs<'w, 's> = Query<'w, 's, &'static mut Transform, With<Part>>;

/// Returns how far part `index` of `beast` of `species` is moved by its attack `going` and by the
/// flinch `reeling`, in the frame of the animal.
fn shift(
    (beast, species): (&Beast, &Species),
    (going, reeling): (Option<&Attacking>, Option<Flinch>),
    index: usize,
) -> Vec3 {
    let body = &species.body;
    let mut out = Vec3::ZERO;
    if let Some(a) = going {
        let aim = aim(a.kind, body.parts[index].kind, front(body, index));
        out += beast.motion.root.turn * (aim * a.swing() * body.length);
    }
    if let Some(f) = reeling.filter(|f| usize::from(f.part) == index) {
        out += f.push * (1.0 - f.age / FLINCH).max(0.0);
    }
    out
}

/// Moves the parts of every animal that makes an attack or flinches from a blow.
pub fn gesture(
    mut commands: Commands,
    (time, terrain): (Res<Time>, Res<Terrain>),
    mut beasts: Query<Gesturing, Busy>,
    mut parts: Limbs,
) {
    let globe = terrain.globe().clone();
    let Some(life) = globe.life() else { return };
    for mut body in &mut beasts {
        let species = life.species.get(usize::from(body.beast.index));
        let reeling = body.flinch.as_mut().map(|f| {
            f.age += time.delta_secs();
            **f
        });
        for (index, part) in body.beast.parts.iter().enumerate() {
            let place = part.and_then(|p| parts.get_mut(p).ok());
            if let (Some(mut place), Some(species)) = (place, species) {
                let at = (body.beast, species);
                place.translation += shift(at, (body.going, reeling), index);
            }
        }
        if reeling.is_some_and(|f| f.age >= FLINCH) {
            commands.entity(body.entity).remove::<Flinch>();
        }
    }
}
