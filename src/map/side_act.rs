//! The buttons of the side panel of the map: looking at a favourite or a player, setting the
//! course to a favourite, renaming it, deleting it and saving a new one.

use bevy::ecs::system::SystemParam;
use bevy::math::DVec3;
use bevy::prelude::*;
use bevy::ui_widgets::Activate;

use super::favourite::Favourites;
use super::level::Level;
use super::me::Me;
use super::side::{Do, FavButton, PlayerRow, SaveFavourite};
use super::surface::Surface;
use crate::computer::{Board, Destination, Target, Typing};
use crate::menus::{MarkerKind, MarkerPlace, Markers};
use crate::space::{Anchor, Frame};
use crate::universe::BodyId;

/// What the buttons of the panel change.
#[derive(SystemParam)]
pub struct Desk<'w, 's> {
    /// The favourites.
    pub(super) favourites: ResMut<'w, Favourites>,
    /// Where the letters typed go.
    pub(super) typing: ResMut<'w, Typing>,
    /// The course.
    pub(super) destination: ResMut<'w, Destination>,
    /// The rows of the chart and the search.
    pub(super) board: ResMut<'w, Board>,
    /// The level shown and the flat map.
    pub(super) view: (ResMut<'w, Level>, ResMut<'w, Surface>),
    /// The markers, the render anchor and the player.
    pub(super) world: (Res<'w, Markers>, Option<Res<'w, Anchor>>, Me<'w, 's>),
}

impl Desk<'_, '_> {
    /// Opens the flat map of `body` looking at `dir`.
    pub(super) fn look(&mut self, body: BodyId, dir: DVec3) {
        let (level, surface) = &mut self.view;
        surface.focus = Some(dir);
        **level = Level::Planet(body);
    }

    /// Runs the button `act` of the favourite at `index`. Renaming one ends and clears a search,
    /// so Escape is left free to close the map.
    pub(super) fn favourite(&mut self, index: usize, act: Do) {
        let Some(favourite) = self.favourites.0.get(index).cloned() else {
            return;
        };
        match act {
            Do::Go => self.look(favourite.body, DVec3::from_array(favourite.dir)),
            Do::Course => self.destination.0 = Some(Target::Body(favourite.body)),
            Do::Rename => {
                *self.typing = Typing::Name(index, favourite.name);
                let board = &mut self.board;
                (board.filter, board.stale) = (String::new(), true);
            }
            Do::Delete => {
                self.favourites.0.remove(index);
                *self.typing = Typing::Off;
            }
        }
    }

    /// Saves the place picked on the flat map, or where the player is, and starts naming it.
    fn save(&mut self) {
        let picked = match *self.view.0 {
            Level::Planet(body) => self.view.1.picked.map(|dir| (body, dir)),
            Level::Chart => None,
        };
        if let Some((body, dir)) = picked.or_else(|| self.world.2.spot()) {
            self.keep(body, dir);
        }
    }

    /// Saves the place `dir` of `body` as a favourite and starts naming it.
    pub(super) fn keep(&mut self, body: BodyId, dir: DVec3) {
        let index = self.favourites.save(body, dir);
        *self.typing = Typing::Name(index, self.favourites.0[index].name.clone());
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
    Option<&'static FavButton>,
    Option<&'static PlayerRow>,
    Has<SaveFavourite>,
);

/// Runs a button of the panel.
pub(super) fn on_activate(event: On<Activate>, roles: Query<Role>, mut desk: Desk) {
    let Ok((favourite, player, save)) = roles.get(event.entity) else {
        return;
    };
    if let Some(FavButton(index, act)) = favourite {
        desk.favourite(*index, *act);
    }
    if let Some(PlayerRow(index)) = player {
        desk.player(*index);
    }
    if save {
        desk.save();
    }
}
