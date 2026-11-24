//! The kinds the pack groups its rows by: made tools first, then each family of material, each with
//! its name and its icon, and the icon of each material.

use crate::craft::matter::{Class, Mineral, Stuff};
use crate::ui::{Icon, sign};

/// A group of the pack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Kind {
    /// Made tools.
    Tool,
    /// Stones from the ground, and glassy crystal tissue.
    Stone,
    /// Bone and antler.
    Bone,
    /// Horn sheaths and shells.
    Shell,
    /// Skin and sinew.
    Hide,
    /// Plant fibre and resin.
    Wood,
}

impl Kind {
    /// Every kind, in the order the pack shows them.
    pub const ALL: [Kind; 6] = [
        Kind::Tool,
        Kind::Stone,
        Kind::Bone,
        Kind::Shell,
        Kind::Hide,
        Kind::Wood,
    ];

    /// Returns the kind of a carried material.
    pub fn of(stuff: &Stuff) -> Kind {
        match stuff {
            Stuff::Wood(_) => Kind::Wood,
            Stuff::Mineral(_) | Stuff::Bullet(_) => Kind::Stone,
            Stuff::Charge(_) | Stuff::Placeable(_) => Kind::Wood,
            Stuff::Tissue { class, .. } => match class {
                Class::Glass => Kind::Stone,
                Class::Bone | Class::Antler => Kind::Bone,
                Class::Keratin | Class::Cuticle => Kind::Shell,
                Class::Hide => Kind::Hide,
                Class::Wood | Class::Resin => Kind::Wood,
            },
        }
    }

    /// Returns the name of the group, as its title shows it.
    pub fn name(self) -> &'static str {
        match self {
            Kind::Tool => "Tools",
            Kind::Stone => "Stone",
            Kind::Bone => "Bone",
            Kind::Shell => "Horn and shell",
            Kind::Hide => "Hide",
            Kind::Wood => "Wood",
        }
    }

    /// Returns the icon of the group.
    pub fn icon(self) -> Icon {
        match self {
            Kind::Tool => Icon::Workshop,
            Kind::Stone => sign::STONE,
            Kind::Bone => Icon::Skeleton,
            Kind::Shell => Icon::Shield,
            Kind::Hide => sign::HIDE,
            Kind::Wood => Icon::Tree,
        }
    }
}

/// Returns the fallback icon-font glyph for a carried material.
pub fn material_icon(stuff: &Stuff) -> Icon {
    match stuff {
        Stuff::Mineral(
            Mineral::Basalt
            | Mineral::Jade
            | Mineral::Ironstone
            | Mineral::Malachite
            | Mineral::Siderite
            | Mineral::Copper
            | Mineral::Galena
            | Mineral::Cobaltite
            | Mineral::Ilmenite,
        ) => sign::STONE,
        Stuff::Mineral(
            Mineral::Flint
            | Mineral::Obsidian
            | Mineral::Quartz
            | Mineral::Brimstone
            | Mineral::Pyrite
            | Mineral::Cinnabar
            | Mineral::Amethyst,
        ) => sign::SHARD,
        Stuff::Mineral(Mineral::Ice | Mineral::Cryolite) => sign::ICE,
        Stuff::Wood(_) => Icon::Tree,
        Stuff::Charge(_) | Stuff::Bullet(_) => sign::SHARD,
        Stuff::Placeable(_) => Icon::Workshop,
        Stuff::Tissue { class, .. } => match class {
            Class::Glass => sign::SHARD,
            Class::Bone | Class::Antler => Icon::Skeleton,
            Class::Keratin | Class::Cuticle => Icon::Shield,
            Class::Hide => sign::HIDE,
            Class::Wood => Icon::Tree,
            Class::Resin => sign::RESIN,
        },
    }
}
