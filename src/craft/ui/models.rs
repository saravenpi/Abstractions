//! Canonical 3D thumbnail identity covering every craftable tool, gun, mineral, and supply.

use super::models_geom::build_tris;
use crate::craft::build::Look;
use crate::craft::gun::GunKind;
use crate::craft::gun::mesh::Tri;
use crate::craft::matter::{ChargeKind, Class, Mineral, PlaceableKind, Stuff, WoodKind};
use crate::craft::pack::{Pack, SlotEntry};

/// High-bit namespace mask preventing collisions with UI bitmap handles.
const KIND_MASK: u64 = 0x3C00_0000_0000_0000;

/// Canonical 3D item variant rendered into isometric UI preview images.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemPreview {
    /// Procedural hand tool silhouette rendered in 3D.
    Tool(Look),
    /// Crafted flintlock pistol or mineral rifle with mineral barrel accent.
    Gun(GunKind, Mineral),
    /// Surface or underground mineral ore crystal.
    Mineral(Mineral),
    /// Timber, milled plank, stick, or charcoal piece.
    Wood(WoodKind),
    /// Propellant powder pouch.
    Charge(ChargeKind),
    /// Mineral-tipped bullet cartridge.
    Bullet(Mineral),
    /// Placeable storage chest or torch.
    Placeable(PlaceableKind),
    /// Organic hide, bone, antler, shell, or horn.
    Tissue(Class),
}

impl ItemPreview {
    /// Resolves a 3D preview descriptor from a [`Stuff`] stack variant.
    pub fn from_stuff(stuff: &Stuff) -> Self {
        match *stuff {
            Stuff::Mineral(m) => ItemPreview::Mineral(m),
            Stuff::Wood(w) => ItemPreview::Wood(w),
            Stuff::Charge(c) => ItemPreview::Charge(c),
            Stuff::Bullet(m) => ItemPreview::Bullet(m),
            Stuff::Placeable(p) => ItemPreview::Placeable(p),
            Stuff::Tissue { class, .. } => ItemPreview::Tissue(class),
        }
    }

    /// Resolves a 3D preview descriptor for inventory `entry` in `pack`.
    pub fn from_slot(pack: &Pack, entry: &SlotEntry) -> Option<Self> {
        match *entry {
            SlotEntry::Tool(id) => Some(ItemPreview::Tool(pack.item(id)?.shape()?)),
            SlotEntry::Gun(id) => {
                let gun = pack.gun(id)?;
                Some(ItemPreview::Gun(gun.kind, gun.barrel))
            }
            SlotEntry::Stack { stuff, .. } => Some(Self::from_stuff(&stuff)),
        }
    }

    /// Returns a collision-free 48-bit discriminant + payload key under `KIND_MASK`.
    pub fn code(&self) -> u64 {
        let tag = match *self {
            ItemPreview::Tool(look) => (1 << 36) | u64::from(look.code()),
            ItemPreview::Gun(k, m) => (2 << 36) | ((k as u64) << 16) | (m as u64),
            ItemPreview::Mineral(m) => (3 << 36) | (m as u64),
            ItemPreview::Wood(w) => (4 << 36) | (w as u64),
            ItemPreview::Charge(c) => (5 << 36) | (c as u64),
            ItemPreview::Bullet(m) => (6 << 36) | (m as u64),
            ItemPreview::Placeable(p) => (7 << 36) | (p as u64),
            ItemPreview::Tissue(c) => (8 << 36) | (c as u64),
        };
        KIND_MASK | tag
    }

    /// Generates the flat-shaded 3D triangles for this item.
    pub fn tris(self) -> Vec<Tri> {
        build_tris(self)
    }
}
