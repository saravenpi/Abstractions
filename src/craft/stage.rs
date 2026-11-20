//! Shared test setups of the pack, the harvest and the workshop: a carried pack, home and a heavy
//! world, and the life of a real home planet.

use crate::craft::build::Place;
use crate::craft::matter::{Class, Grade, Mineral, Stuff, Treatments};
use crate::craft::pack::{Pack, Stack};
use crate::life::{self, Life};
use crate::universe::Universe;

/// Terrain seed of the planet the staged tissues come from, past i64::MAX on purpose.
pub const PLANET: u64 = 0x9E37_79B9_7F4A_7C15;

/// Returns Earth gravity and sea-level air.
pub fn earth() -> Place {
    Place {
        gravity: 9.81,
        air: 1.225,
    }
}

/// Returns a stack of `mass` kilograms of a clean tissue of `class` from species 3.
pub fn tissue(class: Class, tissue: u8, mass: f32) -> Stack {
    Stack {
        stuff: Stuff::Tissue {
            planet: PLANET,
            species: 3,
            tissue,
            class,
            grade: Grade::Clean,
            treated: Treatments::default(),
        },
        material: class.anchor(),
        mass,
    }
}

/// Returns `mass` kilograms of `mineral`.
pub fn stone(mineral: Mineral, mass: f32) -> Stack {
    Stack {
        stuff: Stuff::Mineral(mineral),
        material: mineral.material(),
        mass,
    }
}

/// Returns a pack of bone, wood and hide, flint and obsidian.
pub fn pack() -> Pack {
    let mut pack = Pack::default();
    for stack in [
        tissue(Class::Bone, 0, 3.0),
        tissue(Class::Wood, 1, 3.0),
        tissue(Class::Hide, 2, 1.0),
        stone(Mineral::Flint, 1.5),
        stone(Mineral::Obsidian, 1.5),
    ] {
        pack.stow(stack, f32::INFINITY);
    }
    pack
}

/// Returns the life of the first living home planet among the first universes, and its seed.
pub fn living() -> (Life, u64) {
    (0..64)
        .find_map(|seed| {
            let universe = Universe::new(seed);
            let body = universe.home();
            let planet = universe.planet(body);
            let life = life::of(&planet, &universe.system(body.star).star)?;
            Some((life, planet.seed))
        })
        .expect("a living home planet")
}
