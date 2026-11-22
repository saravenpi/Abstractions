//! Tests of how the marks of the compass settle: every mark keeps its bearing, of two icons
//! that meet the one that ranks first, then the nearer, is drawn whole in front and the other
//! dimmed behind, and no two distances under the bar print over each other.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::show::Seen;
use super::space::{BASE, BEHIND, FRIENDS, HYSTERESIS, space_steady, wide};
use crate::menus::{Marker, MarkerKind, MarkerPlace, Markers};

/// Returns the markers and initial seen marks for `(kind, at, gap)` specs.
fn scene(specs: &[(MarkerKind, f32, f64)]) -> (Markers, Vec<Seen>) {
    let markers = specs
        .iter()
        .map(|(kind, _, _)| Marker {
            kind: *kind,
            place: MarkerPlace::Anchor(DVec3::ZERO),
            color: Color::WHITE,
            label: String::new(),
            owner: 0,
        })
        .collect();
    let seen = specs
        .iter()
        .enumerate()
        .map(|(index, &(_, at, gap))| Seen {
            index,
            at,
            alpha: 1.0,
            gap,
            more: 0,
            wide: 0.0,
            said: true,
        })
        .collect();
    (Markers(markers), seen)
}

/// Settles marks without prior frame history.
fn space(seen: &mut Vec<Seen>, markers: &Markers) {
    space_steady(seen, markers, &[]);
}

/// Of two icons on one spot, the course is drawn whole and last, so in front, and home behind
/// it at the reduced opacity, both at their own bearing.
#[test]
fn two_overlapping_icons_draw_the_first_in_rank_opaque() {
    let (all, mut seen) = scene(&[
        (MarkerKind::Home, 180.0, 300.0),
        (MarkerKind::Course, 184.0, 5.0e6),
    ]);
    space(&mut seen, &all);
    let of = |i| *seen.iter().find(|mark| mark.index == i).unwrap();
    assert_eq!(seen.last().unwrap().index, 1);
    assert_eq!(of(1).alpha, 1.0);
    assert!((of(0).alpha - BEHIND).abs() < 1e-6);
    assert_eq!((of(0).at, of(1).at), (180.0, 184.0));
}

/// Of two friends on one spot, the nearer is in front, and the other one dims with its
/// distance: its distance would print over the front one's, so it is left out.
#[test]
fn of_two_equals_the_nearer_is_in_front_and_says_its_distance() {
    let (all, mut seen) = scene(&[
        (MarkerKind::Friend, 200.0, 900.0),
        (MarkerKind::Friend, 200.0, 40.0),
    ]);
    space(&mut seen, &all);
    let of = |i| *seen.iter().find(|mark| mark.index == i).unwrap();
    let (front, back) = (of(1), of(0));
    assert_eq!(seen.last().unwrap().index, 1);
    assert_eq!((front.alpha, front.said), (1.0, true));
    assert_eq!((back.alpha, back.said), (BEHIND, false));
}

/// Icons apart stay whole and each says its distance under it.
#[test]
fn marks_apart_stay_whole_with_their_distance() {
    let (all, mut seen) = scene(&[
        (MarkerKind::Home, 60.0, 340.0),
        (MarkerKind::Ship, 300.0, 1200.0),
    ]);
    space(&mut seen, &all);
    assert!(seen.iter().all(|mark| mark.alpha == 1.0 && mark.said));
    assert!(seen.iter().all(|mark| mark.wide > BASE));
}

/// However the marks crowd the bar, no two distances that are said meet, every mark keeps its
/// bearing, and the mark drawn in front says its distance.
#[test]
fn distances_never_overlap() {
    let kinds = [MarkerKind::Waypoint, MarkerKind::Home, MarkerKind::Friend];
    let places: Vec<f32> = (0..12).map(|i| 20.0 + 27.0 * i as f32).collect();
    let specs: Vec<_> = (0..12)
        .map(|i| (kinds[i % 3], places[i], 50.0 * 3f64.powi(i as i32 % 7)))
        .collect();
    let (all, mut seen) = scene(&specs);
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
    let specs: Vec<_> = (0..FRIENDS + 2)
        .map(|i| (MarkerKind::Friend, 180.0, 100.0 + 10.0 * i as f64))
        .collect();
    let (all, mut seen) = scene(&specs);
    space(&mut seen, &all);
    let first = *seen.iter().find(|mark| mark.index == 0).unwrap();
    assert_eq!(seen.len(), FRIENDS);
    assert_eq!(first.more, 2);
    assert_eq!(seen.last().unwrap().index, 0);
}

/// Short distance labels clamp their collision box to at least five characters, and two marks of
/// the same rank whose distances cross by a few centimetres keep their order by index instead of
/// swapping front and back every frame.
#[test]
fn short_distances_and_nearly_equal_gaps_stay_steady() {
    assert_eq!(wide(8.0), wide(999.0));
    let (all, mut a) = scene(&[
        (MarkerKind::Waypoint, 200.0, 400.05),
        (MarkerKind::Waypoint, 200.0, 399.95),
    ]);
    let (_, mut b) = scene(&[
        (MarkerKind::Waypoint, 200.0, 399.95),
        (MarkerKind::Waypoint, 200.0, 400.05),
    ]);
    space(&mut a, &all);
    space(&mut b, &all);
    assert_eq!(a.last().unwrap().index, 0);
    assert_eq!(b.last().unwrap().index, 0);
}

/// A distance label crossing the overlap threshold keeps its prior state inside the
/// [`HYSTERESIS`] band so borderline motion does not flicker it on and off.
#[test]
fn distance_labels_keep_a_hysteresis_band_near_overlap() {
    let need = (wide(200.0) + wide(400.0)) * 0.5;
    let step = |offset: f32, was_said: &[usize]| {
        let (all, mut seen) = scene(&[
            (MarkerKind::Waypoint, 200.0, 200.0),
            (MarkerKind::Waypoint, 200.0 + need + offset, 400.0),
        ]);
        space_steady(&mut seen, &all, was_said);
        seen.iter().find(|mark| mark.index == 1).unwrap().said
    };
    let half = HYSTERESIS * 0.5;
    assert!(step(HYSTERESIS + 1.0, &[]));
    assert!(step(-half, &[0, 1]));
    assert!(!step(-HYSTERESIS - 1.0, &[0, 1]));
    assert!(!step(half, &[0]));
    assert!(step(HYSTERESIS + 1.0, &[0]));
}
