//! Boss trophy cache drops when a solitary apex boss titan is slain.

use bevy::prelude::*;

use crate::craft::ground::{Note, Shed, Thing};
use crate::craft::matter::{
    ChargeKind, Class, Grade, Mineral, Stuff, Treatments, UNIT_MASS, bullet_mass, material,
};
use crate::craft::pack::Stack;
use crate::life::Species;
use crate::life::body::PartKind;
use crate::world::Terrain;

/// Apex minerals guaranteed in every boss trophy cache.
const APEX_TROPHIES: [Mineral; 5] = [
    Mineral::Wolframite,
    Mineral::Corundum,
    Mineral::Osmium,
    Mineral::Moissanite,
    Mineral::Lonsdaleite,
];

/// Emitted when a [`Species::boss`] creature dies so its trophy cache is dropped.
#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct BossSlain {
    /// Index of the boss species in the planet's life.
    pub species: u16,
    /// Proper name of the slain boss species.
    pub name: String,
    /// Terrain seed of the planet.
    pub planet: u64,
}

/// Builds the boosted [`Thing::Heap`] tissue trophy stack for `species` on `planet`.
fn trophy_tissue(species: &Species, planet: u64) -> Option<Thing> {
    let tissue = species.tissues.first()?;
    let class = Class::of(tissue.kind, PartKind::Torso).unwrap_or(Class::Bone);
    let mut mat = material(class, tissue, Grade::Clean)?;
    mat.strength *= 1.25;
    mat.toughness *= 1.2;
    mat.hardness = (mat.hardness + 1.0).min(10.0);
    let stuff = Stuff::Tissue {
        planet,
        species: species.id.index,
        tissue: 0,
        class,
        grade: Grade::Clean,
        treated: Treatments::default(),
    };
    Some(Thing::Heap(Stack {
        stuff,
        material: mat,
        mass: 3.0 * UNIT_MASS,
    }))
}

/// Appends the 5 apex minerals, 12 heavy bullets and 4 propellant charges for a slain boss.
fn append_cache(shed: &mut Vec<Thing>, species: u16) {
    for mineral in APEX_TROPHIES {
        shed.push(Thing::Heap(Stack {
            stuff: Stuff::Mineral(mineral),
            material: mineral.material(),
            mass: UNIT_MASS,
        }));
    }
    let ammo = [Mineral::Wolframite, Mineral::Osmium][usize::from(species & 1)];
    shed.push(Thing::Heap(Stack {
        stuff: Stuff::Bullet(ammo),
        material: ammo.material(),
        mass: 12.0 * bullet_mass(ammo),
    }));
    let charge = ChargeKind::BrimstonePowder;
    shed.push(Thing::Heap(Stack {
        stuff: Stuff::Charge(charge),
        material: charge.material(),
        mass: 4.0 * charge.unit_mass(),
    }));
}

/// Drops a generous boss trophy cache into [`Shed`] and announces the kill via [`Note`].
pub fn drop_trophy(
    mut slain: MessageReader<BossSlain>,
    terrain: Res<Terrain>,
    mut shed: ResMut<Shed>,
    mut notes: MessageWriter<Note>,
) {
    let life = terrain.globe().life();
    for ev in slain.read() {
        if let Some(species) = life.and_then(|l| l.species.get(usize::from(ev.species)))
            && let Some(tissue) = trophy_tissue(species, ev.planet)
        {
            shed.0.push(tissue);
        }
        append_cache(&mut shed.0, ev.species);
        notes.write(Note(format!(
            "Slain: {} — Boss trophy cache dropped!",
            ev.name
        )));
    }
}
