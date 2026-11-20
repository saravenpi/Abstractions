//! Shared test setups for builds: the worked example, home and heavy places, and a carried stock.

use super::*;
use crate::craft::matter::{Class, Grade, Material, Mineral, Stuff, Treatments};

/// Returns the home place of the worked example: Earth gravity and sea-level air.
pub(super) fn earth() -> Place {
    Place {
        gravity: 9.81,
        air: 1.225,
    }
}

/// Returns a heavy world: 22 m/s² and the same air.
pub(super) fn heavy() -> Place {
    Place {
        gravity: 22.0,
        ..earth()
    }
}

/// Returns the worked example: a 0.3 kg bone shaft 0.6 m long with a 0.8 kg head of `form` made of
/// `head`, no grip length behind the hand, bound with hide.
pub(super) fn example(form: Form, head: Material) -> Build {
    let bone = Class::Bone.anchor();
    let line: f32 = 0.3 / 0.6;
    Build {
        grip: 0.0,
        shaft: Shaft {
            length: 0.6,
            diameter: (4.0 * line / (std::f32::consts::PI * bone.density)).sqrt(),
            material: bone,
        },
        head: Head {
            form,
            mass: 0.8,
            material: head,
        },
        counterweight: None,
        binding: Some(Class::Hide.anchor()),
        hands: Hands::One,
    }
}

/// Returns a carried tissue of `class` from species 3 of a planet, `mass` kilograms of it.
pub(super) fn tissue(class: Class, tissue: u8, mass: f32) -> Stock {
    Stock {
        stuff: Stuff::Tissue {
            planet: 0x9E37_79B9_7F4A_7C15,
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

/// Returns a carried stock: bone, wood and hide from one species, and flint and obsidian.
pub(super) fn stock() -> Vec<Stock> {
    let stone = |mineral: Mineral| Stock {
        stuff: Stuff::Mineral(mineral),
        material: mineral.material(),
        mass: 1.5,
    };
    vec![
        tissue(Class::Bone, 0, 3.0),
        tissue(Class::Wood, 1, 3.0),
        tissue(Class::Hide, 2, 1.0),
        stone(Mineral::Flint),
        stone(Mineral::Obsidian),
    ]
}
