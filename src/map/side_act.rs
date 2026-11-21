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
use crate::computer::{Board, Computer, Destination, Engage, RouteStatus, Target, Typing};
use crate::menus::{MarkerKind, MarkerPlace, Markers};
use crate::space::{Anchor, Frame};
use crate::universe::BodyId;

/// What the buttons of the panel change.
#[derive(SystemParam)]
pub struct Desk<'w, 's> {
    /// The pins.
    pub(super) places: ResMut<'w, Places>,
    /// Where the letters typed go.
    pub(super) typing: ResMut<'w, Typing>,
    /// The course, and the route that can take it.
    pub(super) course: (ResMut<'w, Destination>, Option<Res<'w, RouteStatus>>),
    /// The rows of the chart and the search.
    pub(super) board: ResMut<'w, Board>,
    /// The level shown and the flat map.
    pub(super) view: (ResMut<'w, Level>, ResMut<'w, Surface>),
    /// The markers, the render anchor and the player.
    pub(super) world: (Res<'w, Markers>, Option<Res<'w, Anchor>>, Me<'w, 's>),
    /// Closing the map and handing a course to the autopilot.
    pub(super) screen: (ResMut<'w, NextState<Computer>>, Commands<'w, 's>),
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

    /// Makes the selected pin the active one and closes the map. A pin on another planet also
    /// sets the course to that planet, handed to the autopilot when it can take it.
    fn go(&mut self) {
        let Some(index) = self.places.selected else {
            return;
        };
        let Some(pin) = self.places.pins.get(index).cloned() else {
            return;
        };
        self.places.set_active(Some(index));
        let here = self.world.2.spot().map(|(body, _)| body);
        if here != Some(pin.body) {
            let target = Target::Body(pin.body);
            self.course.0.0 = Some(target);
            if self.course.1.as_deref() == Some(&RouteStatus::Idle) {
                self.screen.1.trigger(Engage(target));
            }
        }
        self.screen.0.set(Computer::Closed);
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
    /// or on the chart.
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
