//! The pack and the things it lets go of or takes back: a tool whole, a heap whole or in part.

use abstractions_net::wire::{Lying, TOOL};

use super::tune::CRUMB;
use super::{Shed, Thing};
use crate::craft::pack::{Pack, Stack};
use crate::ui::units::mass as weight;

/// Lets go of the tool `id`: out of its hotbar slot and the pack, into `shed`.
pub fn shed_item(pack: &mut Pack, shed: &mut Shed, id: u32) {
    let Some(at) = pack.items.iter().position(|item| item.id == id) else {
        return;
    };
    pack.bar.forget(id);
    shed.0.push(Thing::Tool(pack.items.remove(at)));
}

/// Lets go of `share`, 0 to 1, of the stack at `at` into `shed`: the whole stack at 1.
pub fn shed_stack(pack: &mut Pack, shed: &mut Shed, at: usize, share: f32) {
    let Some(stack) = pack.stacks.get(at).copied() else {
        return;
    };
    let mass = stack.mass * share.clamp(0.0, 1.0);
    if mass > 0.0 && pack.take(&stack.stuff, mass) {
        shed.0.push(Thing::Heap(Stack { mass, ..stack }));
    }
}

/// Puts `thing` back into `pack` whatever it weighs: a tool gets a new id and a free slot.
pub fn restore(pack: &mut Pack, thing: Thing) {
    match thing {
        Thing::Tool(item) => _ = pack.keep(item.recipe, item.materials),
        Thing::Heap(stack) => _ = pack.stow(stack, f32::MAX),
    }
}

/// Puts `thing` into `pack` within `room` kilograms. What does not fit goes into `shed`, to be put
/// down again.
pub(super) fn stow(thing: Thing, room: f32, pack: &mut Pack, shed: &mut Shed) {
    match thing {
        Thing::Tool(item) if item.mass() > room + CRUMB => shed.0.push(Thing::Tool(item)),
        Thing::Tool(item) => _ = pack.keep(item.recipe, item.materials),
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

/// Returns the kilograms of `lying`, called `name`, that a pack with `room` kilograms free takes:
/// a tool whole or not at all, a heap as much as fits. Else the line that says the pack is full.
pub(super) fn wanted(lying: &Lying, room: f32, name: &str) -> Result<f32, String> {
    match lying.kind {
        TOOL if lying.mass <= room + CRUMB => return Ok(lying.mass),
        TOOL => {}
        _ if room >= CRUMB => return Ok(lying.mass.min(room)),
        _ => {}
    }
    let (needs, free) = (weight(f64::from(lying.mass)), weight(f64::from(room)));
    Err(format!(
        "Pack full: the {name} weighs {needs}, room for {free}"
    ))
}
