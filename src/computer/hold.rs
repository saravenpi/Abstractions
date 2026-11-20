//! What Escape lets go of before it closes the map: the selection a click made on the level
//! shown. A row picked with the keys is let go of with the map, as before.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::pointer::MapView;
use crate::map::{Level, Surface};

/// The selections that Escape lets go of.
#[derive(SystemParam)]
pub(super) struct Hold<'w> {
    /// The chart and its picked row.
    view: ResMut<'w, MapView>,
    /// The level shown.
    level: Option<Res<'w, Level>>,
    /// The flat map of a planet and its selected marker.
    surface: Option<ResMut<'w, Surface>>,
}

impl Hold<'_> {
    /// Returns true while something is held that Escape lets go of before it closes the map: a
    /// selection on the level shown.
    pub(super) fn held(&self) -> bool {
        let planet = self
            .level
            .as_deref()
            .is_some_and(|level| *level != Level::Chart);
        match planet {
            true => self
                .surface
                .as_ref()
                .is_some_and(|surface| surface.chosen.is_some()),
            false => self.view.marked,
        }
    }

    /// Lets go of every selection.
    pub(super) fn release(&mut self) {
        (self.view.picked, self.view.marked) = (false, false);
        if let Some(surface) = self.surface.as_mut() {
            (surface.chosen, surface.picked) = (None, None);
        }
    }
}
