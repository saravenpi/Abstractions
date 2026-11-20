//! Opening the menu of a right click on a row of the side panel: a search result, a favourite
//! or a player. The menu is the one a right click on the same thing on the map opens, with the
//! same choices, at the pointer and inside the window.

use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use bevy::window::PrimaryWindow;

use super::{ContextMenu, Opened, Under, choices, place};
use crate::computer::{Board, Slot};
use crate::map::favourite::Favourites;
use crate::map::side::{FavRow, PlayerRow};
use crate::menus::{MarkerKind, MarkerPlace, Markers};
use crate::space::{Anchor, Frame};

/// A row of the side panel a right click can land on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SideRow {
    /// The search result drawn in this slot, counted from the top of the list.
    Result(usize),
    /// The favourite at this index of the list.
    Favourite(usize),
    /// The player at this index of the list of friends.
    Player(usize),
}

/// What the rows stand for: the results, the favourites, and the friends with the frame of the
/// render anchor, which places a friend given in it.
pub struct Listed<'a> {
    /// The rows of the search.
    pub board: &'a Board,
    /// How many favourites are listed.
    pub favourites: usize,
    /// Every marker, the friends among them.
    pub markers: &'a Markers,
    /// The frame of the render anchor, if any.
    pub frame: Option<Frame>,
}

/// Returns what the row `row` stands for, as the map would find it under the pointer: the place
/// of a result, a favourite, or a player on the planet they are on. A row past the end of its
/// list, and a player off any planet, stand for nothing.
pub fn on_row(row: SideRow, listed: &Listed) -> Option<Under> {
    match row {
        SideRow::Result(slot) => listed.board.at(slot).map(|line| Under::Place(line.target)),
        SideRow::Favourite(index) => (index < listed.favourites).then_some(Under::Favourite(index)),
        SideRow::Player(index) => {
            let friends = listed.markers.0.iter();
            let friend = friends
                .filter(|marker| marker.kind == MarkerKind::Friend)
                .nth(index)?;
            let body = match (friend.place, listed.frame) {
                (MarkerPlace::Body(body, _), _) => body,
                (MarkerPlace::Anchor(_), Some(Frame::Surface(body))) => body,
                _ => return None,
            };
            Some(Under::Player(index, body))
        }
    }
}

/// Returns the row a node is, from its parts.
fn kind(
    slot: Option<&Slot>,
    favourite: Option<&FavRow>,
    player: Option<&PlayerRow>,
) -> Option<SideRow> {
    match (slot, favourite, player) {
        (Some(Slot(at)), _, _) => Some(SideRow::Result(*at)),
        (_, Some(FavRow(at)), _) => Some(SideRow::Favourite(*at)),
        (_, _, Some(PlayerRow(at))) => Some(SideRow::Player(*at)),
        _ => None,
    }
}

/// The rows of the side panel, with whether the pointer is over each.
type Rows<'w, 's> = Query<
    'w,
    's,
    (
        &'static RelativeCursorPosition,
        Option<&'static Slot>,
        Option<&'static FavRow>,
        Option<&'static PlayerRow>,
    ),
    Or<(With<Slot>, With<FavRow>, With<PlayerRow>)>,
>;

/// What the rows stand for.
type Lists<'w> = (
    Res<'w, Board>,
    Res<'w, Favourites>,
    Res<'w, Markers>,
    Option<Res<'w, Anchor>>,
);

/// Opens the menu at the pointer on a right click on a row of the side panel, for what the row
/// stands for.
pub(super) fn summon(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    rows: Rows,
    (board, favourites, markers, anchor): Lists,
    mut menu: ResMut<ContextMenu>,
) {
    if !buttons.just_pressed(MouseButton::Right) || menu.0.is_some() {
        return;
    }
    let (Some(window), Some(row)) = (
        windows.iter().next(),
        rows.iter()
            .filter(|(over, ..)| over.cursor_over)
            .find_map(|(_, slot, favourite, player)| kind(slot, favourite, player)),
    ) else {
        return;
    };
    let listed = Listed {
        board: &board,
        favourites: favourites.0.len(),
        markers: &markers,
        frame: anchor.map(|anchor| anchor.frame),
    };
    let (Some(at), Some(under)) = (window.cursor_position(), on_row(row, &listed)) else {
        return;
    };
    let at = place(at, window.size(), choices(&under).len());
    menu.0 = Some(Opened { under, at });
}

#[cfg(test)]
#[path = "rows_tests.rs"]
mod tests;
#[cfg(test)]
#[path = "rows_open_tests.rs"]
mod tests_open;
