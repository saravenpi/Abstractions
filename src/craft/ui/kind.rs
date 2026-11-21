//! The kinds the pack groups its rows by: made tools first, then each family of material, each with
//! its name and its icon.

use crate::craft::matter::{Class, Stuff};
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
        let Stuff::Tissue { class, .. } = stuff else {
            return Kind::Stone;
        };
        match class {
            Class::Glass => Kind::Stone,
            Class::Bone | Class::Antler => Kind::Bone,
            Class::Keratin | Class::Cuticle => Kind::Shell,
            Class::Hide => Kind::Hide,
            Class::Wood | Class::Resin => Kind::Wood,
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
