//! Aiming a cut: the part of a carcass the robot looks at, whether it is worth taking, and the
//! name the prompt gives it.

use bevy::camera::primitives::Aabb;
use bevy::prelude::*;

use super::Dead;
use super::tune::{CUT_RANGE, CUT_SLACK};
use crate::craft::matter::Class;
use crate::fauna::beast::{Beast, Part};
use crate::life::body::PartKind;
use crate::life::{Life, Species};

/// The parts of the animals as the cut sees them.
pub type Parts<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static GlobalTransform,
        &'static Aabb,
        &'static ChildOf,
    ),
    With<Part>,
>;

/// The dead animals.
pub type Carcasses<'w, 's> = Query<'w, 's, &'static mut Beast, With<Dead>>;

/// A part of a carcass the robot may cut.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Target {
    /// The entity of the part.
    pub part: Entity,
    /// The dead animal it belongs to.
    pub body: Entity,
    /// Index of the part in the body of the species.
    pub index: usize,
}

/// Returns how far off the line of sight from `from` along the unit `ahead` a part centred at
/// `centre`, `size` metres across from its middle, lies, when it is ahead of the eyes and within
/// [`CUT_RANGE`] of the robot at `robot`, or of the eyes with no robot: in third person the camera
/// stands metres behind the robot, and the reach is the robot's.
pub fn sighted(
    centre: Vec3,
    size: f32,
    (from, ahead): (Vec3, Vec3),
    robot: Option<Vec3>,
) -> Option<f32> {
    let rel = centre - from;
    let along = rel.dot(ahead);
    let off = (rel - ahead * along).length();
    let near = robot.map_or(along <= CUT_RANGE, |robot| {
        robot.distance(centre) <= CUT_RANGE + size
    });
    (along >= 0.0 && near && off <= size + CUT_SLACK).then_some(off)
}

/// Returns the part of a carcass nearest the line of sight `view` within reach of the robot at
/// `robot`, with the carcass.
fn aimed(
    parts: &Parts,
    dead: &Carcasses,
    view: (Vec3, Vec3),
    robot: Option<Vec3>,
) -> Option<(Entity, Entity)> {
    let near = parts.iter().filter_map(|(part, placed, aabb, of)| {
        dead.contains(of.parent()).then_some(())?;
        let centre = placed.transform_point(Vec3::from(aabb.center));
        let size = Vec3::from(aabb.half_extents).length() * placed.scale().max_element();
        Some((sighted(centre, size, view, robot)?, part, of.parent()))
    });
    near.min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, p, b)| (p, b))
}

/// Returns true when part `index` of `species` has tissue worth taking.
fn worth(species: &Species, index: usize) -> bool {
    let part = species.body.parts.get(index);
    let tissue = part.and_then(|p| species.tissues.get(usize::from(p.tissue)).map(|t| (t, p)));
    tissue.is_some_and(|(t, p)| Class::of(t.kind, p.kind).is_some())
}

/// Returns the part of a carcass that an eye at `eye` looks at and that is worth cutting, among the
/// animals of `life`.
pub fn target(
    (eye, robot): (&GlobalTransform, Option<Vec3>),
    (parts, dead): (&Parts, &Carcasses),
    life: &Life,
) -> Option<Target> {
    let view = (eye.translation(), eye.forward().as_vec3());
    let (part, body) = aimed(parts, dead, view, robot)?;
    let beast = dead.get(body).ok()?;
    let index = beast.parts.iter().position(|p| *p == Some(part))?;
    let species = life.species.get(usize::from(beast.index))?;
    worth(species, index).then_some(Target { part, body, index })
}

/// Returns the name the prompt gives part `index` of `species`, in lower case.
pub fn named(species: &Species, index: usize) -> &'static str {
    let kind = species.body.parts.get(index).map(|part| part.kind);
    match kind {
        Some(PartKind::Torso | PartKind::Blob | PartKind::Segment) => "body",
        Some(PartKind::Pelvis) => "hips",
        Some(PartKind::Neck) => "neck",
        Some(PartKind::Head) => "head",
        Some(PartKind::Jaw) => "jaw",
        Some(PartKind::Tail) => "tail",
        Some(PartKind::Thigh) => "thigh",
        Some(PartKind::Shin) => "shin",
        Some(PartKind::Foot) => "foot",
        Some(PartKind::Wing) => "wing",
        Some(PartKind::Fin) => "fin",
        Some(PartKind::Bell) => "bell",
        Some(PartKind::Tentacle) => "tentacle",
        Some(PartKind::Horn) => "horn",
        Some(PartKind::Plate) => "plate",
        Some(PartKind::Eye) => "eye",
        Some(PartKind::Antenna) => "antenna",
        Some(PartKind::Crystal) => "crystal",
        None => "part",
    }
}
