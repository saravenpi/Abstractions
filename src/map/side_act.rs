//! The buttons of the side panel of the map: selecting a pin, going to it, renaming it and
//! removing it, and looking at a player. The rename and remove keys do the same as their buttons.

use bevy::ecs::system::SystemParam;
use bevy::math::DVec3;
use bevy::prelude::*;
use bevy::ui_widgets::Activate;

use super::level::Level;
use super::me::Me;
use super::places::Places;
use super::side::{Do, PinButton, PinRow, PlayerRow};
use super::surface::Surface;
use crate::computer::{Board, Go, Tab, Target, Typing};
use crate::menus::{MarkerKind, MarkerPlace, Markers};
use crate::saves::Pin;
use crate::space::{Anchor, Frame};
use crate::universe::BodyId;

/// Commits Go on `pin`, the pin at `index`, for a player on the planet `here`: on the planet of the pin it is
/// the active pin, which the compass leads to and which counts as reached on landing near it; from
/// anywhere else the course is the planet, and no pin is active, so that the course is done once the ship
/// is in orbit and never waits for a spot on the surface. Returns the course to set.
fn commit(
    places: &mut Places,
    (index, pin): (usize, &Pin),
    here: Option<BodyId>,
) -> Option<Target> {
    if here == Some(pin.body) {
        places.set_active(Some(index));
        return None;
    }
    places.set_active(None);
    Some(Target::Body(pin.body))
}

/// What the buttons of the panel change.
#[derive(SystemParam)]
pub struct Desk<'w, 's> {
    /// The pins.
    pub(super) places: ResMut<'w, Places>,
    /// Where the letters typed go.
    pub(super) typing: ResMut<'w, Typing>,
    /// The rows of the chart and the search.
    pub(super) board: ResMut<'w, Board>,
    /// The level shown and the flat map.
    pub(super) view: (ResMut<'w, Level>, ResMut<'w, Surface>),
    /// The markers, the render anchor and the player.
    pub(super) world: (Res<'w, Markers>, Option<Res<'w, Anchor>>, Me<'w, 's>),
    /// Starting a course and closing the map.
    pub(super) go: Go<'w, 's>,
}

impl Desk<'_, '_> {
    /// Opens the flat map of `body` looking at `dir`.
    pub(super) fn look(&mut self, body: BodyId, dir: DVec3) {
        let (level, surface) = &mut self.view;
        surface.focus = Some(dir);
        **level = Level::Planet(body);
    }

    /// Selects the pin at `index` and looks at it on the flat map.
    fn pick(&mut self, index: usize) {
        let Some(pin) = self.places.pins.get(index).cloned() else {
            return;
        };
        self.places.selected = Some(index);
        self.look(pin.body, DVec3::from_array(pin.dir));
    }

    /// Goes to the selected pin, see [`commit`]: on its planet it becomes the active pin and the
    /// map closes; from anywhere else [`Go::press`] handles the course, the seat, the route and
    /// flying home.
    fn go(&mut self) {
        let Some(index) = self.places.selected else {
            return;
        };
        let Some(pin) = self.places.pins.get(index).cloned() else {
            return;
        };
        let here = self.world.2.spot().map(|(body, _)| body);
        match commit(&mut self.places, (index, &pin), here) {
            Some(target) => self.go.press(Some(target)),
            None => self.go.close(),
        }
    }

    /// Runs the button `act` on the selected pin. Renaming one ends and clears a search, so
    /// Escape is left free to close the map.
    pub(super) fn pin(&mut self, act: Do) {
        let Some(index) = self.places.selected else {
            return;
        };
        match (act, self.places.pins.get(index).cloned()) {
            (Do::Go, _) => self.go(),
            (Do::Rename, Some(pin)) => {
                *self.typing = Typing::Name(index, pin.name);
                let board = &mut self.board;
                (board.filter, board.stale) = (String::new(), true);
            }
            (Do::Remove, _) => {
                self.places.remove(index);
                *self.typing = Typing::Off;
            }
            _ => {}
        }
    }

    /// Looks at the player at `index` of the list of friends: on the flat map of their planet,
    /// or on the system/universe chart.
    pub(super) fn player(&mut self, index: usize) {
        let (markers, anchor, _) = &self.world;
        let friends = markers
            .0
            .iter()
            .filter(|marker| marker.kind == MarkerKind::Friend);
        let Some(place) = friends.map(|marker| marker.place).nth(index) else {
            return;
        };
        let frame = anchor.as_deref().map(|anchor| anchor.frame);
        match (place, frame) {
            (MarkerPlace::Body(body, at), _) => self.look(body, at),
            (MarkerPlace::Anchor(at), Some(Frame::Surface(body))) => self.look(body, at),
            (MarkerPlace::Star(star, _), _) => {
                self.board.open(Tab::System, Some(Target::Star(star)));
                *self.view.0 = Level::Chart;
            }
            _ => *self.view.0 = Level::Chart,
        }
    }
}

/// The parts of a button of the panel that say what it does.
type Role = (
    Option<&'static PinRow>,
    Option<&'static PinButton>,
    Option<&'static PlayerRow>,
);

/// Runs a button of the panel.
pub(super) fn on_activate(event: On<Activate>, roles: Query<Role>, mut desk: Desk) {
    let Ok((row, button, player)) = roles.get(event.entity) else {
        return;
    };
    if let Some(PinRow(index)) = row {
        desk.pick(*index);
    }
    if let Some(PinButton(act)) = button {
        desk.pin(*act);
    }
    if let Some(PlayerRow(index)) = player {
        desk.player(*index);
    }
}

/// The key that renames the selected pin.
pub(super) const RENAME_KEY: KeyCode = KeyCode::KeyR;

/// Renames the selected pin on its key and removes it on Delete, unless a search or a name is typed.
pub(super) fn pin_keys(keys: Res<ButtonInput<KeyCode>>, mut desk: Desk) {
    if !desk.typing.is_off() {
        return;
    }
    if keys.just_pressed(RENAME_KEY) {
        desk.pin(Do::Rename);
    }
    if keys.just_pressed(KeyCode::Delete) {
        desk.pin(Do::Remove);
    }
}

#[cfg(test)]
mod tests_commit;
