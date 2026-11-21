//! Tests of how the marks of the compass settle: every mark keeps its bearing, of two icons
//! that meet the one that ranks first, then the nearer, is drawn whole in front and the other
//! dimmed behind, and no two distances under the bar print over each other.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::show::Seen;
use super::space::{BASE, BEHIND, FRIENDS, space};
use crate::menus::{Marker, MarkerKind, MarkerPlace, Markers};

/// Returns the markers of the cases, one of each `kinds`, the game's own.
fn markers(kinds: &[MarkerKind]) -> Markers {
    let marker = |kind: &MarkerKind| Marker {
        kind: *kind,
        place: MarkerPlace::Anchor(DVec3::ZERO),
        color: Color::WHITE,
        label: String::new(),
        owner: 0,
    };
    Markers(kinds.iter().map(marker).collect())
}

/// Returns the mark of marker `index`, `at` pixels along the bar and `gap` metres away, whole.
fn mark(index: usize, at: f32, gap: f64) -> Seen {
    Seen {
        index,
        at,
        alpha: 1.0,
        gap,
        more: 0,
        wide: 0.0,
        said: true,
    }
}

/// Returns the mark of marker `index` among `seen`.
fn of(seen: &[Seen], index: usize) -> Seen {
    *seen.iter().find(|mark| mark.index == index).unwrap()
}

/// Of two icons on one spot, the course is drawn whole and last, so in front, and home behind
/// it at the reduced opacity, both at their own bearing.
#[test]
fn two_overlapping_icons_draw_the_first_in_rank_opaque() {
    let all = markers(&[MarkerKind::Home, MarkerKind::Course]);
    let mut seen = vec![mark(0, 180.0, 300.0), mark(1, 184.0, 5.0e6)];
    space(&mut seen, &all);
    assert_eq!(seen.last().unwrap().index, 1);
    assert_eq!(of(&seen, 1).alpha, 1.0);
    assert!((of(&seen, 0).alpha - BEHIND).abs() < 1e-6);
    assert_eq!((of(&seen, 0).at, of(&seen, 1).at), (180.0, 184.0));
}

/// Of two friends on one spot, the nearer is in front, and the other one dims with its
/// distance: its distance would print over the front one's, so it is left out.
#[test]
fn of_two_equals_the_nearer_is_in_front_and_says_its_distance() {
    let all = markers(&[MarkerKind::Friend, MarkerKind::Friend]);
    let mut seen = vec![mark(0, 200.0, 900.0), mark(1, 200.0, 40.0)];
    space(&mut seen, &all);
    let (front, back) = (of(&seen, 1), of(&seen, 0));
    assert_eq!(seen.last().unwrap().index, 1);
    assert_eq!((front.alpha, front.said), (1.0, true));
    assert_eq!((back.alpha, back.said), (BEHIND, false));
}

/// Icons apart stay whole and each says its distance under it.
#[test]
fn marks_apart_stay_whole_with_their_distance() {
    let all = markers(&[MarkerKind::Home, MarkerKind::Ship]);
    let mut seen = vec![mark(0, 60.0, 340.0), mark(1, 300.0, 1200.0)];
    space(&mut seen, &all);
    assert!(seen.iter().all(|mark| mark.alpha == 1.0 && mark.said));
    assert!(seen.iter().all(|mark| mark.wide > BASE));
}

/// However the marks crowd the bar, no two distances that are said meet, every mark keeps its
/// bearing, and the mark drawn in front says its distance.
#[test]
fn distances_never_overlap() {
    let kinds = [MarkerKind::Waypoint, MarkerKind::Home, MarkerKind::Friend];
    let all = markers(&kinds.repeat(4));
    let places: Vec<f32> = (0..12).map(|i| 20.0 + 27.0 * i as f32).collect();
    let mut seen: Vec<Seen> = (0..12)
        .map(|i| mark(i, places[i], 50.0 * 3f64.powi(i as i32 % 7)))
        .collect();
    space(&mut seen, &all);
    let said: Vec<&Seen> = seen.iter().filter(|mark| mark.said).collect();
    for (i, a) in said.iter().enumerate() {
        for b in &said[i + 1..] {
            let room = (a.wide + b.wide) * 0.5;
            assert!((a.at - b.at).abs() >= room, "{} and {}", a.index, b.index);
        }
    }
    assert!(seen.iter().all(|mark| mark.at == places[mark.index]));
    assert!(seen.last().unwrap().said);
}

/// Friends past the limit fold into the nearest one, which counts them.
#[test]
fn friends_past_the_limit_fold_into_the_nearest() {
    let all = markers(&[MarkerKind::Friend; FRIENDS + 2]);
    let mut seen: Vec<Seen> = (0..FRIENDS + 2)
        .map(|i| mark(i, 180.0, 100.0 + 10.0 * i as f64))
        .collect();
    space(&mut seen, &all);
    assert_eq!(seen.len(), FRIENDS);
    assert_eq!(of(&seen, 0).more, 2);
    assert_eq!(seen.last().unwrap().index, 0);
}
