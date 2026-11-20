//! Map tests: a row of the side panel stands for what the map would find under the pointer,
//! with the same choices: a result for its place, a favourite and a player for themselves.

use bevy::math::DVec3;

use super::super::rig::{board, friend};
use super::super::{Choice, Under, choices};
use super::{Listed, SideRow, on_row};
use crate::menus::{MarkerPlace, Markers};
use crate::space::Frame;
use crate::universe::{BodyId, StarId};

/// Returns what each row offers with `markers` listed and the anchor in `frame`: the choices
/// of its menu, or nothing.
fn offered(row: SideRow, markers: &Markers, frame: Frame) -> Option<Vec<Choice>> {
    let board = board();
    let listed = Listed {
        board: &board,
        favourites: 2,
        markers,
        frame: Some(frame),
    };
    on_row(row, &listed).map(|under| choices(&under).to_vec())
}

/// Returns a planet that is not the default one.
fn other() -> BodyId {
    BodyId {
        index: 2,
        ..BodyId::default()
    }
}

/// A result offers what its place offers on the map, and a row past the list opens nothing.
#[test]
fn a_result_offers_what_its_place_offers() {
    use Choice::*;
    let (markers, frame) = (Markers::default(), Frame::Surface(other()));
    let result = |slot| offered(SideRow::Result(slot), &markers, frame);
    assert_eq!(result(0), Some(vec![Course, Focus, Waypoint, Favourite]));
    assert_eq!(result(1), Some(vec![Course, Focus]));
    assert_eq!(result(2), Some(vec![Course, Focus]));
    assert_eq!(result(3), None);
}

/// A favourite offers the choices of a favourite on the map, while it is listed.
#[test]
fn a_favourite_offers_what_a_favourite_offers() {
    use Choice::*;
    let (markers, frame) = (Markers::default(), Frame::Surface(other()));
    let favourite = |index| offered(SideRow::Favourite(index), &markers, frame);
    assert_eq!(favourite(1), Some(vec![Course, Focus, Rename, Delete]));
    assert_eq!(favourite(2), None);
}

/// A player offers the choices of a player on the planet they are on, given on it or in the
/// surface frame of the anchor. A player in open space and a row past the list open nothing.
#[test]
fn a_player_offers_what_a_player_offers() {
    let markers = Markers(vec![
        friend(MarkerPlace::Body(other(), DVec3::X)),
        friend(MarkerPlace::Anchor(DVec3::Y)),
    ]);
    let board = board();
    let listed = |frame| Listed {
        board: &board,
        favourites: 0,
        markers: &markers,
        frame: Some(frame),
    };
    let surface = listed(Frame::Surface(BodyId::default()));
    assert_eq!(
        on_row(SideRow::Player(0), &surface),
        Some(Under::Player(0, other()))
    );
    let anchored = Some(Under::Player(1, BodyId::default()));
    assert_eq!(on_row(SideRow::Player(1), &surface), anchored);
    let shown = offered(SideRow::Player(0), &markers, Frame::Surface(other()));
    assert_eq!(shown, Some(vec![Choice::Focus, Choice::Course]));
    let space = listed(Frame::System(StarId::default()));
    assert_eq!(on_row(SideRow::Player(1), &space), None);
    assert_eq!(on_row(SideRow::Player(2), &space), None);
}
