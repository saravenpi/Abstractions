//! Keeping the messages an app writes, for the tests to read.

use bevy::prelude::*;

/// The messages of type `M` written while the app ran, oldest first.
#[derive(Resource)]
pub struct Kept<M: Message + Clone>(pub Vec<M>);

/// Keeps every message of type `M` in [`Kept`].
fn keep<M: Message + Clone>(mut reader: MessageReader<M>, mut kept: ResMut<Kept<M>>) {
    kept.0.extend(reader.read().cloned());
}

/// Keeps every message of type `M` the app writes from now on.
pub fn watch<M: Message + Clone>(app: &mut App) {
    app.insert_resource(Kept::<M>(Vec::new()))
        .add_systems(Last, keep::<M>);
}

/// Returns the messages of type `M` kept so far.
pub fn kept<M: Message + Clone>(app: &App) -> Vec<M> {
    app.world().resource::<Kept<M>>().0.clone()
}
