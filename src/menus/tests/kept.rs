//! Keeping the messages a test app sends to the network, since a message lives two frames only.

use bevy::prelude::*;

/// Every message of one kind sent since the last call to [`kept`].
#[derive(Resource)]
struct Kept<M: Message>(Vec<M>);

/// Keeps the messages of the frame.
fn keep<M: Message + Clone>(mut sent: MessageReader<M>, mut kept: ResMut<Kept<M>>) {
    kept.0.extend(sent.read().cloned());
}

/// Starts keeping the messages of kind `M`.
pub(super) fn watch<M: Message + Clone>(app: &mut App) {
    app.insert_resource(Kept::<M>(Vec::new()))
        .add_systems(Last, keep::<M>);
}

/// Returns and forgets every message of kind `M` sent since the last call.
pub(super) fn kept<M: Message + Clone>(app: &mut App) -> Vec<M> {
    std::mem::take(&mut app.world_mut().resource_mut::<Kept<M>>().0)
}
