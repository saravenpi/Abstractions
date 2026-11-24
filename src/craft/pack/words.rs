//! Words for the pack: the plain names of materials, wood, ammo, placeables, and tool shapes.

use crate::craft::build::{Figure, Intent};
use crate::craft::matter::{Mineral, Stuff};

/// Returns the lowercase name of `mineral`.
pub fn mineral_name(mineral: Mineral) -> &'static str {
    match mineral {
        Mineral::Basalt => "basalt",
        Mineral::Obsidian => "obsidian",
        Mineral::Flint => "flint",
        Mineral::Ice => "ice",
        Mineral::Quartz => "quartz",
        Mineral::Jade => "jade",
        Mineral::Ironstone => "ironstone",
        Mineral::Malachite => "malachite",
        Mineral::Brimstone => "brimstone",
        Mineral::Siderite => "siderite",
        Mineral::Copper => "copper",
        Mineral::Pyrite => "pyrite",
        Mineral::Galena => "galena",
        Mineral::Cobaltite => "cobaltite",
        Mineral::Cinnabar => "cinnabar",
        Mineral::Amethyst => "amethyst",
        Mineral::Ilmenite => "ilmenite",
        Mineral::Cryolite => "cryolite",
    }
}

/// Returns the plain name of what `stuff` is.
pub fn called(stuff: &Stuff) -> &'static str {
    match stuff {
        Stuff::Tissue { class, .. } => class.name(),
        Stuff::Mineral(m) => mineral_name(*m),
        Stuff::Wood(wood) => wood.name(),
        Stuff::Charge(charge) => charge.name(),
        Stuff::Placeable(placeable) => placeable.name(),
        Stuff::Bullet(m) => bullet_label(*m),
    }
}

/// Returns a descriptive name for mineral bullets.
fn bullet_label(mineral: Mineral) -> &'static str {
    match mineral {
        Mineral::Basalt => "basalt slug",
        Mineral::Obsidian => "obsidian slug",
        Mineral::Flint => "flint slug",
        Mineral::Ice => "ice slug",
        Mineral::Quartz => "quartz slug",
        Mineral::Jade => "jade slug",
        Mineral::Ironstone => "ironstone slug",
        Mineral::Malachite => "malachite slug",
        Mineral::Brimstone => "brimstone slug",
        Mineral::Siderite => "siderite slug",
        Mineral::Copper => "copper bullet",
        Mineral::Pyrite => "pyrite bullet",
        Mineral::Galena => "galena bullet",
        Mineral::Cobaltite => "cobaltite bullet",
        Mineral::Cinnabar => "cinnabar bullet",
        Mineral::Amethyst => "amethyst bullet",
        Mineral::Ilmenite => "ilmenite bullet",
        Mineral::Cryolite => "cryolite bullet",
    }
}

/// Returns the word for a tool of `intent` whose head holds `share` of its mass, `reach` metres long.
pub fn noun(intent: Intent, share: f32, reach: f32) -> &'static str {
    Figure::of(intent, share, reach).word()
}
