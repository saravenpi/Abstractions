//! The pack and the things it lets go of or takes back: tools, firearms, or discrete heaps.

use abstractions_net::wire::{GUN, Lying, TOOL};

use super::tune::CRUMB;
use super::{Shed, Thing};
use crate::craft::gun::Gun;
use crate::craft::pack::{Pack, SlotEntry, Stack};
use crate::ui::units::mass as weight;

/// Returns true if `pack` has at least one empty slot in the 24+6 grid.
fn has_empty_slot(pack: &Pack) -> bool {
    pack.grid.iter().any(Option::is_none)
}

/// Stores a picked-up firearm with a fresh pack-unique id, returning false if no slot is free.
fn keep_picked_gun(pack: &mut Pack, mut gun: Gun) -> bool {
    pack.ensure_grid();
    let Some(slot) = pack.grid.iter().position(Option::is_none) else {
        return false;
    };
    let id = pack.next;
    pack.next = pack.next.wrapping_add(1);
    gun.id = id;
    pack.guns.push(gun);
    pack.grid[slot] = Some(SlotEntry::Gun(id));
    pack.sync_views();
    true
}

/// Lets go of the tool `id`: out of its slot and the pack, into `shed`.
pub fn shed_item(pack: &mut Pack, shed: &mut Shed, id: u32) {
    pack.ensure_grid();
    let Some(at) = pack.items.iter().position(|item| item.id == id) else {
        return;
    };
    for slot in &mut pack.grid {
        if matches!(slot, Some(SlotEntry::Tool(has)) if *has == id) {
            *slot = None;
        }
    }
    shed.0.push(Thing::Tool(pack.items.remove(at)));
    pack.sync_views();
}

/// Lets go of `share`, 0 to 1, of the grouped stack at `at` into `shed`.
pub fn shed_stack(pack: &mut Pack, shed: &mut Shed, at: usize, share: f32) {
    pack.ensure_grid();
    let Some(stack) = pack.stacks.get(at).copied() else {
        return;
    };
    let units = pack.count_of(&stack.stuff) as f32;
    let drop_n = (units * share.clamp(0.0, 1.0)).round().max(1.0) as u16;
    if drop_n > 0 && pack.take_count(&stack.stuff, drop_n) {
        let mass = f32::from(drop_n) * stack.stuff.unit_mass();
        shed.0.push(Thing::Heap(Stack { mass, ..stack }));
    }
}

/// Drops the exact slot entry at `slot` into `shed`.
pub fn shed_slot(pack: &mut Pack, shed: &mut Shed, slot: usize) {
    pack.ensure_grid();
    let Some(entry) = pack.grid.get_mut(slot).and_then(Option::take) else {
        return;
    };
    match entry {
        SlotEntry::Tool(id) => {
            if let Some(i) = pack.items.iter().position(|it| it.id == id) {
                shed.0.push(Thing::Tool(pack.items.remove(i)));
            }
        }
        SlotEntry::Gun(id) => {
            if let Some(i) = pack.guns.iter().position(|g| g.id == id) {
                shed.0.push(Thing::Gun(pack.guns.remove(i)));
            }
        }
        SlotEntry::Armor(id) => {
            if let Some(cell) = pack.grid.get_mut(slot) {
                *cell = Some(SlotEntry::Armor(id));
            }
        }
        SlotEntry::Stack {
            stuff,
            material,
            count,
        } => {
            let mass = f32::from(count) * stuff.unit_mass();
            shed.0.push(Thing::Heap(Stack {
                stuff,
                material,
                mass,
            }));
        }
    }
    pack.sync_views();
}

/// Puts `thing` back into `pack` whatever it weighs.
pub fn restore(pack: &mut Pack, thing: Thing) {
    pack.ensure_grid();
    match thing {
        Thing::Tool(item) => _ = pack.keep(item.recipe, item.materials),
        Thing::Gun(gun) => _ = keep_picked_gun(pack, gun),
        Thing::Heap(stack) => _ = pack.stow(stack, f32::MAX),
    }
}

/// Puts `thing` into `pack` within `room` kilograms and slot availability; overflow goes into `shed`.
pub(super) fn stow(thing: Thing, room: f32, pack: &mut Pack, shed: &mut Shed) {
    pack.ensure_grid();
    match thing {
        Thing::Tool(item) if item.mass() > room + CRUMB || !has_empty_slot(pack) => {
            shed.0.push(Thing::Tool(item));
        }
        Thing::Tool(item) => _ = pack.keep(item.recipe, item.materials),
        Thing::Gun(gun) if gun.mass() > room + CRUMB || !has_empty_slot(pack) => {
            shed.0.push(Thing::Gun(gun));
        }
        Thing::Gun(gun) => _ = keep_picked_gun(pack, gun),
        Thing::Heap(stack) => {
            let left = stack.mass - pack.stow(stack, room);
            if left > CRUMB {
                shed.0.push(Thing::Heap(Stack {
                    mass: left,
                    ..stack
                }));
            }
        }
    }
}

/// Returns the kilograms of `lying`, called `name`, that a pack with `room` kilograms free takes.
pub(super) fn wanted(lying: &Lying, room: f32, name: &str) -> Result<f32, String> {
    match lying.kind {
        TOOL | GUN if lying.mass <= room + CRUMB => return Ok(lying.mass),
        TOOL | GUN => {}
        _ if room >= CRUMB => return Ok(lying.mass.min(room)),
        _ => {}
    }
    let (needs, free) = (weight(f64::from(lying.mass)), weight(f64::from(room)));
    Err(format!(
        "Pack full: the {name} weighs {needs}, room for {free}"
    ))
}
