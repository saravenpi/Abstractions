//! The board and the friends the tests of the menu of the side panel run on.

use bevy::prelude::*;

use crate::computer::{Board, Line, Target};
use crate::menus::{Marker, MarkerKind, MarkerPlace};
use crate::universe::{BodyId, StarId};

/// Returns a row of the search for `target`.
pub fn line(target: Target) -> Line {
    Line {
        title: "Kolva".to_string(),
        detail: String::new(),
        target,
        distance: 1000.0,
        visited: false,
        at: Default::default(),
    }
}

/// Returns a board listing a planet, a star and a galaxy, in that order.
pub fn board() -> Board {
    let star = StarId::default();
    let mut board = Board::default();
    let targets = [
        Target::Body(BodyId::default()),
        Target::Star(star),
        Target::Galaxy(star.galaxy),
    ];
    board.fill("Varena system".to_string(), targets.map(line).to_vec());
    board
}

/// Returns a friend at `place`.
pub fn friend(place: MarkerPlace) -> Marker {
    Marker {
        kind: MarkerKind::Friend,
        place,
        color: Color::WHITE,
        label: "Ada".to_string(),
        owner: 7,
    }
}
