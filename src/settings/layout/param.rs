//! The labels of the keys as a system reads them.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::KeyLabels;

/// The labels of the keys for a system that shows them. An app without the labels, such as the
/// test of one area, reads those of QWERTY.
#[derive(SystemParam)]
pub struct Labels<'w> {
    /// The labels, when the app has them.
    known: Option<Res<'w, KeyLabels>>,
}

impl Labels<'_> {
    /// Returns the labels.
    pub fn get(&self) -> KeyLabels {
        self.known.as_deref().copied().unwrap_or_default()
    }

    /// Returns true when a label changed since the system ran last.
    pub fn is_changed(&self) -> bool {
        self.known.as_ref().is_some_and(|known| known.is_changed())
    }
}
