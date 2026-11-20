//! Shared test setups for strikes: tools built from anchor materials on an Earth-like place.

use super::held::Held;
use crate::craft::build::{Design, Intent, Place, Recipe, Wear};
use crate::craft::matter::{Class, Grade, Material, Mineral, Stuff, Treatments};

/// Returns a place with Earth gravity and sea-level air.
pub fn earth() -> Place {
    Place {
        gravity: 9.81,
        air: 1.225,
    }
}

/// Returns a tissue of `class` from species 0 of a made-up planet, as stuff.
fn tissue(class: Class, tissue: u8) -> Stuff {
    Stuff::Tissue {
        planet: 7,
        species: 0,
        tissue,
        class,
        grade: Grade::Clean,
        treated: Treatments::default(),
    }
}

/// Returns a tool for `intent` with a wooden shaft of `length` metres, a flint head of `head`
/// kilograms and a hide binding, held at Earth.
pub fn tool(intent: Intent, length: f32, head: f32) -> Held {
    let stuff = vec![
        tissue(Class::Wood, 0),
        Stuff::Mineral(Mineral::Flint),
        tissue(Class::Hide, 1),
    ];
    let materials: Vec<Material> = vec![
        Class::Wood.anchor(),
        Mineral::Flint.material(),
        Class::Hide.anchor(),
    ];
    let design = Design::of(intent, length, head, [0, 1, 2]);
    let build = design.build(&materials).expect("every material is listed");
    let recipe = Recipe {
        design,
        stuff,
        wear: Wear::default(),
    };
    Held::new(recipe, build, earth())
}

/// Returns a slashing blade: a 0.8 m shaft with a 0.1 kg edge.
pub fn blade() -> Held {
    tool(Intent::Cut, 0.8, 0.1)
}

/// Returns a very light knife: a 0.2 m shaft with a 0.1 kg edge.
pub fn knife() -> Held {
    tool(Intent::Cut, 0.2, 0.1)
}

/// Returns a spear made to be thrown: a 1.2 m shaft with a 0.2 kg point.
pub fn spear() -> Held {
    tool(Intent::Throw, 1.2, 0.2)
}

/// Returns a club: a 0.6 m shaft with a 1.5 kg block.
pub fn club() -> Held {
    tool(Intent::Crush, 0.6, 1.5)
}
