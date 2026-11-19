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
use crate::computer::{Destination, Target, Typing};
use crate::menus::{MarkerKind, MarkerPlace, Markers};
use crate::space::{Anchor, Frame};
use crate::universe::BodyId;

/// What the buttons of the panel change.
#[derive(SystemParam)]
pub struct Desk<'w, 's> {
    favourites: ResMut<'w, Favourites>,
    typing: ResMut<'w, Typing>,
    destination: ResMut<'w, Destination>,
    view: (ResMut<'w, Level>, ResMut<'w, Surface>),
    world: (Res<'w, Markers>, Option<Res<'w, Anchor>>, Me<'w, 's>),
}

impl Desk<'_, '_> {
    /// Opens the flat map of `body` looking at `dir`.
    fn look(&mut self, body: BodyId, dir: DVec3) {
        let (level, surface) = &mut self.view;
        surface.focus = Some(dir);
        **level = Level::Planet(body);
    }

    /// Runs the button `act` of the favourite at `index`.
    fn favourite(&mut self, index: usize, act: Do) {
        let Some(favourite) = self.favourites.0.get(index).cloned() else {
            return;
        };
        match act {
            Do::Go => self.look(favourite.body, DVec3::from_array(favourite.dir)),
            Do::Course => self.destination.0 = Some(Target::Body(favourite.body)),
            Do::Rename => *self.typing = Typing::Name(index, favourite.name),
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
        let Some((body, dir)) = picked.or_else(|| self.world.2.spot()) else {
            return;
        };
        let index = self.favourites.save(body, dir);
        *self.typing = Typing::Name(index, self.favourites.0[index].name.clone());
    }

    /// Looks at the player at `index` of the list of friends: on the flat map of their planet,
    /// or on the chart.
    fn player(&mut self, index: usize) {
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
