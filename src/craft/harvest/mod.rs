//! Harvesting: what a body part of a dead animal gives, and how the tool decides it.
//!
//! The right tool gives the right yield (roadmap 0.13): skinning needs an edge harder than the hide,
//! cracking out shell and bone needs a crusher, drawing resin needs a careful pierce. A right tool
//! with too soft an edge tears half the tissue and drops its grade one step; the wrong tool, or bare
//! hands, ruins three quarters and leaves the rest poor.
//!
//! Another system sends a [`Harvest`] message for a part of a carcass; [`reap`] answers it from the
//! life underfoot and the held tool, into the pack.

mod tune;

#[cfg(test)]
mod tests;

use bevy::prelude::*;

use crate::craft::build::Intent;
use crate::craft::matter::{Class, Grade, Stuff, Treatments, material};
use crate::craft::pack::{Burden, Pack, Stack};
use crate::craft::wield::Held;
use crate::life::Life;
use crate::world::Terrain;
use tune::{BLUNT, FILL, RUINED, SKIN};

/// One part of one dead animal to take.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cut {
    /// Index of the species in the life of the planet underfoot.
    pub species: u16,
    /// Index of the part in the species' body.
    pub part: u8,
    /// How cleanly the animal was killed.
    pub kill: Grade,
}

/// Asks to harvest a part of a dead animal with the held tool.
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Harvest(pub Cut);

/// The tool a part is taken with: what it is for and the Mohs hardness of its head.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tool {
    /// What it is for.
    pub intent: Intent,
    /// Mohs hardness of its edge, point or head.
    pub hardness: f32,
}

/// Returns the intent that takes a tissue of `class` whole.
pub fn need(class: Class) -> Intent {
    match class {
        Class::Hide | Class::Keratin | Class::Wood => Intent::Cut,
        Class::Bone | Class::Antler | Class::Cuticle | Class::Glass => Intent::Crush,
        Class::Resin => Intent::Pierce,
    }
}

/// Returns the grade one step under `grade`.
fn worse(grade: Grade) -> Grade {
    match grade {
        Grade::Clean => Grade::Good,
        Grade::Good | Grade::Poor => Grade::Poor,
    }
}

/// Returns the share of the part `tool` takes whole from a tissue of `class` and Mohs `hardness`,
/// and the grade left from a kill of `kill`.
pub fn fit(class: Class, hardness: f32, tool: Option<Tool>, kill: Grade) -> (f32, Grade) {
    let Some(tool) = tool else {
        return (RUINED, Grade::Poor);
    };
    let used = match tool.intent {
        Intent::Throw => Intent::Pierce,
        intent => intent,
    };
    let wanted = need(class);
    let sharp = wanted == Intent::Crush || tool.hardness > hardness;
    match (used == wanted, sharp) {
        (true, true) => (1.0, kill),
        (true, false) => (BLUNT, worse(kill)),
        (false, _) => (RUINED, Grade::Poor),
    }
}

/// Returns what `cut` gives from `life` on the planet of terrain seed `planet` with `tool`, or
/// nothing for a part that is not there or too soft to build with (gel, glands).
pub fn harvest(life: &Life, planet: u64, cut: Cut, tool: Option<Tool>) -> Option<Stack> {
    let species = life.species.get(usize::from(cut.species))?;
    let part = species.body.parts.get(usize::from(cut.part))?;
    let tissue = species.tissues.get(usize::from(part.tissue))?;
    let class = Class::of(tissue.kind, part.kind)?;
    let fresh = material(class, tissue, cut.kill)?;
    let (share, grade) = fit(class, fresh.hardness, tool, cut.kill);
    let skin = if class == Class::Hide { SKIN } else { 1.0 };
    let volume = part.size.x * part.size.y * part.size.z * FILL;
    let stuff = Stuff::Tissue {
        planet,
        species: cut.species,
        tissue: part.tissue,
        class,
        grade,
        treated: Treatments::default(),
    };
    Some(Stack {
        stuff,
        material: material(class, tissue, grade)?,
        mass: volume * tissue.density * skin * share,
    })
}

/// Returns the tool the held item makes.
fn tool(held: &Held) -> Option<Tool> {
    held.0.as_ref().map(|hold| Tool {
        intent: hold.intent,
        hardness: hold.build.head.material.hardness,
    })
}

/// Answers every [`Harvest`] with the held tool, from the life underfoot, into the pack as far as
/// the robot can carry it.
pub(crate) fn reap(
    mut asked: MessageReader<Harvest>,
    (terrain, held): (Option<Res<Terrain>>, Res<Held>),
    burden: Res<Burden>,
    mut pack: ResMut<Pack>,
) {
    let Some(terrain) = terrain else {
        asked.clear();
        return;
    };
    let mut room = burden.room();
    for Harvest(cut) in asked.read() {
        let life = terrain.globe().life();
        let Some(stack) = life.and_then(|l| harvest(l, terrain.seed(), *cut, tool(&held))) else {
            continue;
        };
        room -= pack.stow(stack, room);
    }
}
