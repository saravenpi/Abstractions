//! Wire codec between local [`SlotEntry`] items, [`ChestItem`] payloads, ground [`Thing`]s,
//! and 3D UI [`ItemPreview`] thumbnails.

use abstractions_net::chest::{
    CHEST_ITEM_BYTES, ChestItem, ITEM_ARMOR, ITEM_GUN, ITEM_STACK, ITEM_TOOL,
    PlaceableKind as NetKind, STACK_MAX, valid_item,
};
use abstractions_net::wire::{decode, encode};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::craft::armor::Armor;
use crate::craft::build::Recipe;
use crate::craft::ground::Thing;
use crate::craft::gun::Gun;
use crate::craft::matter::{Material, PlaceableKind, Stuff};
use crate::craft::pack::{Item, Pack, SlotEntry, Stack};
use crate::craft::ui::models::ItemPreview;

/// Serializes `value` into a wire byte vector bounded by [`CHEST_ITEM_BYTES`].
fn write<T: Serialize>(value: &T) -> Option<Vec<u8>> {
    encode(value).ok().filter(|b| b.len() <= CHEST_ITEM_BYTES)
}

/// Deserializes `bytes` from a chest item payload.
fn read<T: DeserializeOwned>(bytes: &[u8]) -> Option<T> {
    decode(bytes, CHEST_ITEM_BYTES)
        .ok()?
        .map(|(value, _)| value)
}

/// Converts a game [`PlaceableKind`] into its network wire equivalent.
pub fn to_net_kind(kind: PlaceableKind) -> NetKind {
    match kind {
        PlaceableKind::Chest => NetKind::Chest,
        PlaceableKind::Torch => NetKind::Torch,
    }
}

/// Converts a network wire [`NetKind`] back into a game [`PlaceableKind`].
pub fn from_net_kind(kind: NetKind) -> PlaceableKind {
    match kind {
        NetKind::Chest => PlaceableKind::Chest,
        NetKind::Torch => PlaceableKind::Torch,
    }
}

/// Encodes `entry` from `pack` into a validated opaque [`ChestItem`].
pub fn encode_slot(entry: &SlotEntry, pack: &Pack) -> Option<ChestItem> {
    let item = match *entry {
        SlotEntry::Stack {
            stuff,
            material,
            count,
        } => ChestItem {
            kind: ITEM_STACK,
            count: count.clamp(1, STACK_MAX),
            data: write(&(stuff, material))?,
        },
        SlotEntry::Tool(id) => {
            let tool = pack.item(id)?;
            ChestItem {
                kind: ITEM_TOOL,
                count: 1,
                data: write(&(&tool.recipe, &tool.materials))?,
            }
        }
        SlotEntry::Gun(id) => {
            let gun = pack.gun(id)?;
            ChestItem {
                kind: ITEM_GUN,
                count: 1,
                data: write(gun)?,
            }
        }
        SlotEntry::Armor(id) => {
            let armor = pack.armor(id)?;
            ChestItem {
                kind: ITEM_ARMOR,
                count: 1,
                data: write(armor)?,
            }
        }
    };
    valid_item(&item).then_some(item)
}

/// Decodes `item` into a [`SlotEntry`], registering any tool or firearm inside `pack`.
pub fn decode_to_pack(item: &ChestItem, pack: &mut Pack) -> Option<SlotEntry> {
    if !valid_item(item) {
        return None;
    }
    match item.kind {
        ITEM_STACK => {
            let (stuff, material): (Stuff, Material) = read(&item.data)?;
            let count = item.count.clamp(1, STACK_MAX);
            Some(SlotEntry::Stack {
                stuff,
                material,
                count,
            })
        }
        ITEM_TOOL => {
            let (recipe, materials): (Recipe, Vec<Material>) = read(&item.data)?;
            let id = pack.next;
            pack.next = pack.next.wrapping_add(1);
            pack.items.push(Item {
                id,
                recipe,
                materials,
            });
            Some(SlotEntry::Tool(id))
        }
        ITEM_GUN => {
            let mut gun: Gun = read(&item.data)?;
            gun.id = pack.next;
            pack.next = pack.next.wrapping_add(1);
            let id = gun.id;
            pack.guns.push(gun);
            Some(SlotEntry::Gun(id))
        }
        ITEM_ARMOR => {
            let mut armor: Armor = read(&item.data)?;
            armor.id = pack.next;
            pack.next = pack.next.wrapping_add(1);
            let id = armor.id;
            pack.armors.push(armor);
            Some(SlotEntry::Armor(id))
        }
        _ => None,
    }
}

/// Decodes `item` into a ground [`Thing`] suitable for spilling into [`crate::craft::ground::Shed`].
pub fn decode_to_thing(item: &ChestItem) -> Option<Thing> {
    if !valid_item(item) {
        return None;
    }
    match item.kind {
        ITEM_STACK => {
            let (stuff, material): (Stuff, Material) = read(&item.data)?;
            let mass = f32::from(item.count) * stuff.unit_mass();
            Some(Thing::Heap(Stack {
                stuff,
                material,
                mass,
            }))
        }
        ITEM_TOOL => {
            let (recipe, materials): (Recipe, Vec<Material>) = read(&item.data)?;
            Some(Thing::Tool(Item {
                id: 0,
                recipe,
                materials,
            }))
        }
        ITEM_GUN => {
            let mut gun: Gun = read(&item.data)?;
            gun.id = 0;
            Some(Thing::Gun(gun))
        }
        _ => None,
    }
}

/// Decodes `item` into a 3D UI [`ItemPreview`] thumbnail descriptor.
pub fn preview(item: &ChestItem) -> Option<ItemPreview> {
    if !valid_item(item) {
        return None;
    }
    match item.kind {
        ITEM_STACK => {
            let (stuff, _): (Stuff, Material) = read(&item.data)?;
            Some(ItemPreview::from_stuff(&stuff))
        }
        ITEM_TOOL => {
            let (recipe, materials): (Recipe, Vec<Material>) = read(&item.data)?;
            let tool = Item {
                id: 0,
                recipe,
                materials,
            };
            tool.shape().map(ItemPreview::Tool)
        }
        ITEM_GUN => {
            let gun: Gun = read(&item.data)?;
            Some(ItemPreview::Gun(gun.kind, gun.barrel))
        }
        ITEM_ARMOR => {
            let armor: Armor = read(&item.data)?;
            Some(ItemPreview::Armor {
                slot: armor.slot,
                mineral: armor.mineral,
                shielded: armor.shielded,
            })
        }
        _ => None,
    }
}
