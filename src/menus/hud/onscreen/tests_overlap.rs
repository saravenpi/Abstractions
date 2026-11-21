//! Tests of markers that meet on the screen: the pool takes them by rank, the first in rank
//! keeps its room, the next is reduced, and one that would still print over another is drawn
//! behind it, dimmed, without words.

use bevy::prelude::*;

use super::overlap::settle;
use super::place::{MAX, choose};
use super::rule::Face;
use crate::menus::hud::compass::BEHIND;

/// A whole marker: its icon and its words.
const WHOLE: Face = Face {
    glyph: 1.0,
    label: 1.0,
    small: false,
};

/// Two markers far apart both stay whole.
#[test]
fn markers_apart_stay_whole() {
    let mut taken = Vec::new();
    let first = settle(WHOLE, Vec2::new(300.0, 300.0), &mut taken);
    let second = settle(WHOLE, Vec2::new(600.0, 300.0), &mut taken);
    assert_eq!((first, second), (Some(WHOLE), Some(WHOLE)));
    assert_eq!(taken.len(), 2);
}

/// Of two markers that would overlap, the one asked first, which ranks first, stays whole and
/// the other becomes a small icon without words.
#[test]
fn the_first_in_rank_keeps_its_room() {
    let mut taken = Vec::new();
    let course = settle(WHOLE, Vec2::new(300.0, 300.0), &mut taken);
    let home = settle(WHOLE, Vec2::new(380.0, 300.0), &mut taken).expect("a small icon");
    assert_eq!(course, Some(WHOLE));
    assert!(home.small && home.label <= 0.0 && home.glyph > 0.0 && home.glyph < 1.0);
    assert_eq!(taken.len(), 2);
}

/// A marker whose small icon would still sit on another is drawn behind it: a small icon at
/// the reduced opacity, without words, and it takes its room.
#[test]
fn a_marker_right_on_another_is_dimmed_behind() {
    let mut taken = Vec::new();
    settle(WHOLE, Vec2::new(300.0, 300.0), &mut taken);
    let back = settle(WHOLE, Vec2::new(304.0, 302.0), &mut taken).expect("a dimmed icon");
    assert_eq!((back.glyph, back.label, back.small), (BEHIND, 0.0, true));
    assert_eq!(taken.len(), 2);
}

/// Three markers in one place: the first whole, the others small and dimmed behind it, none of
/// them with words, whatever their number.
#[test]
fn a_crowd_leaves_one_whole() {
    let mut taken = Vec::new();
    let drawn: Vec<Option<Face>> = (0..3)
        .map(|_| settle(WHOLE, Vec2::new(500.0, 400.0), &mut taken))
        .collect();
    let back = Face {
        glyph: BEHIND,
        label: 0.0,
        small: true,
    };
    assert_eq!(drawn, [Some(WHOLE), Some(back), Some(back)]);
}

/// A marker that is not drawn, hidden by nearness, takes no room from the next.
#[test]
fn a_hidden_marker_takes_no_room() {
    let gone = Face {
        glyph: 0.0,
        label: 0.0,
        small: true,
    };
    let mut taken = Vec::new();
    assert_eq!(settle(gone, Vec2::new(300.0, 300.0), &mut taken), None);
    assert_eq!(
        settle(WHOLE, Vec2::new(300.0, 300.0), &mut taken),
        Some(WHOLE)
    );
}

/// The name of a near friend, which has no icon, keeps its room like any words do, and is
/// left out rather than reduced when the course is on it.
#[test]
fn a_name_alone_keeps_its_room() {
    let tag = Face {
        glyph: 0.0,
        label: 1.0,
        small: false,
    };
    let mut taken = Vec::new();
    assert_eq!(settle(tag, Vec2::new(300.0, 300.0), &mut taken), Some(tag));
    let mut taken = Vec::new();
    settle(WHOLE, Vec2::new(300.0, 300.0), &mut taken);
    assert_eq!(settle(tag, Vec2::new(300.0, 300.0), &mut taken), None);
}

/// The pool never exceeds six, and the lowest rank comes first, then the nearest.
#[test]
fn the_pool_holds_six_and_keeps_the_course_first() {
    let rank = |i: i32| [3, 0][usize::from(i == 17)];
    let items: Vec<(u8, f64)> = (0..20).map(|i| (rank(i), 100.0 - f64::from(i))).collect();
    let mut chosen = Vec::new();
    choose(&items, &mut chosen);
    assert_eq!(chosen.len(), MAX);
    assert_eq!(chosen[0], 17);
    assert_eq!(chosen[1], 19);
    choose(&items[..2], &mut chosen);
    assert_eq!(chosen.len(), 2);
}
