//! Fracture: which parts of a body can break, and how much energy it takes.

use crate::craft::matter::{Class, Grade, material};
use crate::life::Species;
use crate::life::body::PartKind;

/// Returns true when a part of kind `kind` can break at all: the ones that stick out.
pub fn brittle(kind: PartKind) -> bool {
    matches!(
        kind,
        PartKind::Horn
            | PartKind::Tail
            | PartKind::Thigh
            | PartKind::Shin
            | PartKind::Foot
            | PartKind::Plate
            | PartKind::Wing
            | PartKind::Fin
            | PartKind::Antenna
            | PartKind::Jaw
    )
}

/// Returns true when a part of kind `kind` falls off when it breaks: the ones on a stalk of their own.
pub fn detaches(kind: PartKind) -> bool {
    matches!(kind, PartKind::Horn | PartKind::Plate | PartKind::Antenna)
}

/// Returns true when a blow of `joules` breaks part `index` of `species`: when it puts in more
/// energy than the strain the tissue stores when it cracks, over the volume of the part.
pub fn breaks(species: &Species, index: usize, joules: f32) -> bool {
    let Some(part) = species.body.parts.get(index).filter(|p| brittle(p.kind)) else {
        return false;
    };
    let tissue = species.tissues.get(usize::from(part.tissue));
    let class = tissue.and_then(|t| Class::of(t.kind, part.kind));
    let stuff = tissue
        .zip(class)
        .and_then(|(t, c)| material(c, t, Grade::Good));
    let volume = part.size.x * part.size.y * part.size.z;
    stuff.is_some_and(|m| joules >= m.resilience() * volume)
}
