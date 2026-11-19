//! Writing the lists of the side panel of the map: the names of the favourites, the one being
//! named with a caret, and the players with their colours. Rows past the end are hidden.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::favourite::Favourites;
use super::side::{FavName, FavRow, PlayerDot, PlayerName, PlayerRow};
use crate::computer::Typing;
use crate::menus::{Marker, MarkerKind, Markers};

/// The rows of the two lists.
type Rows<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Node,
        Option<&'static FavRow>,
        Option<&'static PlayerRow>,
    ),
    Or<(With<FavRow>, With<PlayerRow>)>,
>;

/// The texts and squares of the two lists.
#[derive(SystemParam)]
pub struct Faces<'w, 's> {
    /// The names of the favourites.
    names: Query<'w, 's, (&'static FavName, &'static mut Text), Without<PlayerName>>,
    /// The names of the players.
    players: Query<'w, 's, (&'static PlayerName, &'static mut Text), Without<FavName>>,
    /// The colours of the players.
    dots: Query<'w, 's, (&'static PlayerDot, &'static mut BackgroundColor)>,
}

/// Returns the name shown for the favourite at `index`: what is typed, with a caret, while
/// it is named.
fn shown(favourites: &Favourites, typing: &Typing, index: usize) -> Option<String> {
    let favourite = favourites.0.get(index)?;
    Some(match typing {
        Typing::Name(at, name) if *at == index => format!("{name}_"),
        _ => favourite.name.clone(),
    })
}

/// Writes the list of favourites when they or the name typed change.
fn favourites_text(favourites: &Favourites, typing: &Typing, faces: &mut Faces) {
    for (FavName(index), mut text) in &mut faces.names {
        if let Some(name) = shown(favourites, typing, *index) {
            text.set_if_neq(Text(name));
        }
    }
}

/// Writes the list of players, the markers at `friends` in `markers`: their names and their
/// colours.
fn players_text(markers: &[Marker], friends: &[usize], faces: &mut Faces) {
    let friend = |index: usize| friends.get(index).and_then(|at| markers.get(*at));
    for (PlayerName(index), mut text) in &mut faces.players {
        if let Some(friend) = friend(*index).filter(|friend| text.0 != friend.label) {
            text.0 = friend.label.clone();
        }
    }
    for (PlayerDot(index), mut fill) in &mut faces.dots {
        if let Some(friend) = friend(*index) {
            fill.set_if_neq(BackgroundColor(friend.color));
        }
    }
}

/// Writes both lists of the side panel and hides the rows past their ends.
pub(super) fn write(
    (favourites, typing, markers): (Res<Favourites>, Res<Typing>, Res<Markers>),
    mut rows: Rows,
    mut faces: Faces,
    mut friends: Local<Vec<usize>>,
) {
    friends.clear();
    let found = markers
        .0
        .iter()
        .enumerate()
        .filter(|(_, marker)| marker.kind == MarkerKind::Friend);
    friends.extend(found.map(|(index, _)| index));
    for (mut node, favourite, player) in &mut rows {
        let shown = match (favourite, player) {
            (Some(FavRow(index)), _) => *index < favourites.0.len(),
            (_, Some(PlayerRow(index))) => *index < friends.len(),
            _ => false,
        };
        let display = [Display::None, Display::Flex][usize::from(shown)];
        if node.display != display {
            node.display = display;
        }
    }
    if favourites.is_changed() || typing.is_changed() {
        favourites_text(&favourites, &typing, &mut faces);
    }
    players_text(&markers.0, &friends, &mut faces);
}
