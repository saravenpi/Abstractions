//! The sound of a blow on an animal: which layer rang, how stiff it is and how much of the animal
//! rings with it.

use bevy::prelude::*;

use super::open::stack;
use crate::audio::{Body, Impact, Outer};
use crate::craft::matter::Class;
use crate::life::Species;
use crate::life::body::PartKind;

/// Returns the outer layer a part of `kind` shows when its outer layer is of `class`, or flesh when
/// it has none.
pub fn outer(class: Option<Class>, kind: PartKind) -> Outer {
    match class {
        None => Outer::Flesh,
        Some(Class::Hide | Class::Wood | Class::Resin) => Outer::Hide,
        Some(Class::Cuticle) => Outer::Shell,
        Some(Class::Keratin) if kind == PartKind::Plate => Outer::Plate,
        Some(Class::Keratin | Class::Antler) => Outer::Horn,
        Some(Class::Bone) => Outer::Bone,
        Some(Class::Glass) => Outer::Plate,
    }
}

/// Returns the mass of part `index` of `species` in kilograms: its share of the volume of the body
/// times the mass of the animal.
pub fn part_mass(species: &Species, index: usize) -> f32 {
    let volume = |p: &crate::life::body::Part| p.size.x * p.size.y * p.size.z;
    let all: f32 = species.body.parts.iter().map(volume).sum();
    let part = species.body.parts.get(index).map_or(0.0, volume);
    species.history.mass * part / all.max(f32::EPSILON)
}

/// Returns the impact of a blow of `joules` that `tool` struck at `at` on part `index` of `species`,
/// `cut` when it opened it: the outer layer that rang and the part that rings.
pub fn impact(
    species: &Species,
    index: usize,
    (at, joules, cut): (Vec3, f32, bool),
    tool: Body,
) -> Impact {
    let (layers, skin) = stack(species, index);
    let first = layers[0];
    let class = (skin > 0).then_some(first.class);
    Impact {
        at,
        joules,
        cut,
        outer: outer(class, species.body.parts[index].kind),
        struck: Body {
            stiffness: first.material.stiffness,
            mass: part_mass(species, index),
        },
        tool,
        deflected: false,
    }
}
