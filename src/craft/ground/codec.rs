//! A thing as the ground keeps it: the bytes of its material and of the rest the game rebuilds it
//! from, in the codec of the streams, beside its mass and its wear in plain numbers.

use abstractions_net::wire::{HEAP, LYING_BYTES, Lying, TOOL, decode, encode};
use bevy::math::DVec3;
use serde::Serialize;
use serde::de::DeserializeOwned;

use super::Thing;
use crate::craft::build::{Place, Recipe, mass};
use crate::craft::matter::{Material, Stuff};
use crate::craft::pack::{Item, Stack, called, noun};
use crate::sim::PlayerId;
use crate::universe::BodyId;

/// Returns the bytes of `value`, or nothing when it does not encode.
fn bytes<T: Serialize>(value: &T) -> Option<Vec<u8>> {
    encode(value).ok()
}

/// Returns the value of `bytes`, or nothing when they are not one.
fn read<T: DeserializeOwned>(bytes: &[u8]) -> Option<T> {
    decode(bytes, LYING_BYTES).ok()?.map(|(value, _)| value)
}

/// Returns the record of `thing` put down on `body` at `at`, metres in its turning frame, at
/// universe time `since`, worn as it is at `place`. Nothing when it does not fit a record.
pub fn lying(
    thing: &Thing,
    (body, at): (BodyId, DVec3),
    place: &Place,
    since: f64,
) -> Option<Lying> {
    let (kind, material, item, mass, wear) = match thing {
        Thing::Tool(item) => {
            let rest = bytes(&(&item.recipe, &item.materials))?;
            (TOOL, Vec::new(), rest, item.mass(), item.worn(place).0)
        }
        Thing::Heap(stack) => {
            let (stuff, numbers) = (bytes(&stack.stuff)?, bytes(&stack.material)?);
            (HEAP, stuff, numbers, stack.mass, 0.0)
        }
    };
    let lying = Lying {
        id: 0,
        by: PlayerId::LOCAL,
        tag: 0,
        body,
        at: at.to_array(),
        kind,
        material,
        mass,
        wear,
        item,
        since,
    };
    lying.sane().then_some(lying)
}

/// Returns the thing `lying` holds, or nothing when its bytes are not one. A tool comes back with
/// id 0: the pack gives it a new one.
pub fn thing(lying: &Lying) -> Option<Thing> {
    match lying.kind {
        TOOL => {
            let (recipe, materials): (Recipe, Vec<Material>) = read(&lying.item)?;
            Some(Thing::Tool(Item {
                id: 0,
                recipe,
                materials,
            }))
        }
        HEAP => Some(Thing::Heap(Stack {
            stuff: read::<Stuff>(&lying.material)?,
            material: read::<Material>(&lying.item)?,
            mass: lying.mass,
        })),
        _ => None,
    }
}

/// Returns the plain word for `thing`: the material of a heap, the shape of a tool.
pub fn word(thing: &Thing) -> &'static str {
    match thing {
        Thing::Heap(stack) => called(&stack.stuff),
        Thing::Tool(item) => item.build().map_or("tool", |build| {
            let share = build.head.mass / mass(&build.pieces()).max(f32::EPSILON);
            noun(item.intent(), share, build.reach())
        }),
    }
}
