//! What Escape lets go of before it closes the map: the menu of a right click, then the
//! selection a click made on the level shown. A row picked with the keys is let go of with the
//! map, as before.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::pointer::MapView;
use crate::map::{ContextMenu, Level, Surface};

/// The menu and the selections that Escape lets go of one at a time.
#[derive(SystemParam)]
pub(super) struct Hold<'w> {
    /// The chart and its picked row.
    view: ResMut<'w, MapView>,
    /// The level shown.
    level: Option<Res<'w, Level>>,
    /// The flat map of a planet and its selected marker.
    surface: Option<ResMut<'w, Surface>>,
    /// The menu of a right click.
    menu: Option<ResMut<'w, ContextMenu>>,
}

impl Hold<'_> {
    /// Returns true while something is held that Escape lets go of before it closes the map:
    /// an open menu, or a selection on the level shown.
    pub(super) fn held(&self) -> bool {
        let menu = self.menu.as_ref().is_some_and(|menu| menu.0.is_some());
        let planet = self
            .level
            .as_deref()
            .is_some_and(|level| *level != Level::Chart);
        let chosen = match planet {
            true => self
                .surface
                .as_ref()
                .is_some_and(|surface| surface.chosen.is_some()),
            false => self.view.marked,
        };
        menu || chosen
    }

    /// Lets go of the menu when it is open, or else of every selection.
    pub(super) fn release(&mut self) {
        if let Some(menu) = self.menu.as_mut().filter(|menu| menu.0.is_some()) {
            menu.0 = None;
            return;
        }
        (self.view.picked, self.view.marked) = (false, false);
        if let Some(surface) = self.surface.as_mut() {
            (surface.chosen, surface.picked) = (None, None);
        }
    }
}
