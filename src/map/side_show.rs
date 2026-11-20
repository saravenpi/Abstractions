//! Writing the lists of the side panel of the map: the names of the favourites, the one being
//! named with a caret, the players with their colours, and what the save button saves. Rows
//! past the end are hidden.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::favourite::Favourites;
use super::level::Level;
use super::side::{FavName, FavRow, PlayerDot, PlayerName, PlayerRow, Players, SAVE, SaveWords};
use crate::computer::{Board, Typing};
use crate::menus::{Marker, MarkerKind, Markers};

/// The rows of the two lists.
type Rows<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Node,
        Option<&'static FavRow>,
        Option<&'static PlayerRow>,
        Has<Players>,
    ),
    Or<(With<FavRow>, With<PlayerRow>, With<Players>)>,
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

/// What the lists are written from: the favourites, the name typed, the markers and the board,
/// which says how many rows of each the window has room for.
type Lists<'w> = (
    Res<'w, Favourites>,
    Res<'w, Typing>,
    Res<'w, Markers>,
    Res<'w, Board>,
);

/// Writes both lists of the side panel and hides the rows past their ends or past the room of
/// the window, and the players while none is shown.
pub(super) fn write(
    (favourites, typing, markers, board): Lists,
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
    let room = board.lists.unwrap_or((usize::MAX, usize::MAX));
    let (saved, met) = (favourites.0.len().min(room.0), friends.len().min(room.1));
    for (mut node, favourite, player, block) in &mut rows {
        let shown = match (favourite, player) {
            (Some(FavRow(index)), _) => *index < saved,
            (_, Some(PlayerRow(index))) => *index < met,
            _ => block && met > 0,
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

/// Writes what the button that saves a favourite saves, when the level changes or the panel
/// was just spawned: the place picked on a planet, where the player is on a chart.
pub(super) fn save_words(
    level: Res<Level>,
    fresh: Query<(), Added<SaveWords>>,
    mut texts: Query<&mut Text, With<SaveWords>>,
) {
    if !(level.is_changed() || !fresh.is_empty()) {
        return;
    }
    let said = SAVE[usize::from(*level == Level::Chart)];
    for mut text in &mut texts {
        text.set_if_neq(Text(said.to_string()));
    }
}
