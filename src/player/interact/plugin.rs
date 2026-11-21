//! The plugin of the interact key: the resource and the system that settles the offers of each
//! frame before the systems of the next one read them.

use bevy::prelude::*;

use super::Interact;

/// Keeps the action offered last frame that comes first, for the prompt and the keys of this
/// frame.
fn settle(mut interact: ResMut<Interact>) {
    let mut next = interact.clone();
    if next.settle() {
        *interact = next;
    }
}

/// Registers the interact resource and the system that settles it. Every area that offers or
/// acts on an action adds it, so that each works in a test app of its own.
pub struct Interacting;

impl Plugin for Interacting {
    fn build(&self, app: &mut App) {
        app.init_resource::<Interact>().add_systems(
            PreUpdate,
            settle.run_if(resource_exists_and_changed::<Interact>),
        );
    }
}

/// Adds [`Interacting`] to `app` unless an area already did.
pub fn interacting(app: &mut App) {
    if !app.is_plugin_added::<Interacting>() {
        app.add_plugins(Interacting);
    }
}
