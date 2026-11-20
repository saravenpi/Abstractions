//! The held tool: the item of the held hotbar slot, as the swing system reads it, and the wear its
//! blows leave.
//!
//! [`Held`] is the one place the swing code reads the held tool from. It sends [`Worn`] once a blow
//! of that tool lands, and the joints of the item count it.

use bevy::prelude::*;

use super::build::{Build, Intent};
use super::pack::{Pack, SLOTS};

/// The number keys of the hotbar slots, in slot order.
pub const KEYS: [KeyCode; SLOTS] = [
    KeyCode::Digit1,
    KeyCode::Digit2,
    KeyCode::Digit3,
    KeyCode::Digit4,
    KeyCode::Digit5,
    KeyCode::Digit6,
];

/// The held tool, resolved.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hold {
    /// Its id in the pack.
    pub id: u32,
    /// What it is for.
    pub intent: Intent,
    /// Its parts and materials.
    pub build: Build,
}

/// The tool in the robot's hand, or nothing for bare hands.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Default)]
pub struct Held(pub Option<Hold>);

/// Says one blow of the item with this id landed.
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Worn(pub u32);

/// Returns the hotbar slot whose number key was pressed this frame.
pub fn pressed(keys: &ButtonInput<KeyCode>) -> Option<usize> {
    KEYS.iter().position(|key| keys.just_pressed(*key))
}

/// Holds the hotbar slot whose number key was pressed.
pub(crate) fn choose(keys: Res<ButtonInput<KeyCode>>, mut pack: ResMut<Pack>) {
    if let Some(slot) = pressed(&keys).filter(|slot| *slot != pack.bar.held) {
        pack.bar.select(slot);
    }
}

/// Writes the held tool from the pack when the pack changes.
pub(crate) fn hold(pack: Res<Pack>, mut held: ResMut<Held>) {
    let hold = pack.held().and_then(|item| {
        Some(Hold {
            id: item.id,
            intent: item.intent(),
            build: item.build()?,
        })
    });
    held.set_if_neq(Held(hold));
}

/// Counts every landed blow on the joints of the item that struck it.
pub(crate) fn wear(mut blows: MessageReader<Worn>, mut pack: ResMut<Pack>) {
    for Worn(id) in blows.read() {
        if let Some(item) = pack.items.iter_mut().find(|item| item.id == *id) {
            item.recipe.wear.hit();
        }
    }
}
