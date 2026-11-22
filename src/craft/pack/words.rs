//! Words for the pack: the plain names of materials and of tool shapes.

use crate::craft::build::{Figure, Intent};
use crate::craft::matter::{Mineral, Stuff};

/// Returns the plain name of what `stuff` is.
pub fn called(stuff: &Stuff) -> &'static str {
    match stuff {
        Stuff::Tissue { class, .. } => class.name(),
        Stuff::Mineral(Mineral::Basalt) => "basalt",
        Stuff::Mineral(Mineral::Obsidian) => "obsidian",
        Stuff::Mineral(Mineral::Flint) => "flint",
        Stuff::Mineral(Mineral::Ice) => "ice",
        Stuff::Mineral(Mineral::Quartz) => "quartz",
        Stuff::Mineral(Mineral::Jade) => "jade",
        Stuff::Mineral(Mineral::Ironstone) => "ironstone",
        Stuff::Mineral(Mineral::Malachite) => "malachite",
        Stuff::Mineral(Mineral::Brimstone) => "brimstone",
        Stuff::Mineral(Mineral::Siderite) => "siderite",
    }
}

/// Returns the word for a tool of `intent` whose head holds `share` of its mass, `reach` metres long.
pub fn noun(intent: Intent, share: f32, reach: f32) -> &'static str {
    Figure::of(intent, share, reach).word()
}
