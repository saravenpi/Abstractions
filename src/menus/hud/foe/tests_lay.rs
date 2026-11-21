//! Tests of the pool of labels: every animal in view within the far range has a label, none past
//! it, never more than the pool, brighter up close and for the animal fought, and a bunched herd
//! keeps its bars while the farther names step aside.

use bevy::prelude::*;

use super::lay::{DIM, FULL, faint};
use super::{FAR, Found, Label, SLOTS, lay};

/// The screen of the tests in interface pixels.
const SCREEN: Vec2 = Vec2::new(1280.0, 720.0);

/// Returns an animal at `distance` metres whose head shows at `pos`.
fn found(distance: f32, pos: Vec2) -> Found {
    Found {
        species: 1,
        health: 0.8,
        pos,
        distance,
        keen: false,
    }
}

/// Returns the labels laid for `herd`.
fn laid(herd: &mut [Found]) -> Vec<Option<Label>> {
    let (mut labels, mut taken) = (vec![None; SLOTS], Vec::new());
    lay(herd, SCREEN, &mut labels, &mut taken);
    labels
}

/// Animals spread over the screen at 5 to 55 m each get a label; one at 70 m gets none.
#[test]
fn every_animal_in_view_within_range_has_a_label() {
    let mut herd: Vec<Found> = (0..6)
        .map(|n| {
            found(
                5.0 + 10.0 * n as f32,
                Vec2::new(120.0 + 200.0 * n as f32, 200.0),
            )
        })
        .collect();
    herd.push(found(FAR + 10.0, Vec2::new(640.0, 600.0)));
    let labels = laid(&mut herd);
    let shown = labels.iter().flatten().count();
    assert_eq!(shown, 6, "{labels:?}");
    assert!(
        labels
            .iter()
            .flatten()
            .all(|label| label.named && label.alpha > 0.0)
    );
}

/// A herd of a hundred bunched in the middle of the screen fills the pool and no more: every label
/// keeps its bar, and the names behind the nearest step aside.
#[test]
fn a_bunched_herd_fills_the_pool_and_keeps_its_bars() {
    let mut herd: Vec<Found> = (0..100)
        .map(|n| {
            found(
                6.0 + n as f32 * 0.3,
                Vec2::new(500.0 + (n % 5) as f32 * 8.0, 300.0),
            )
        })
        .collect();
    let labels = laid(&mut herd);
    let shown: Vec<Label> = labels.iter().flatten().copied().collect();
    assert_eq!(shown.len(), SLOTS);
    assert!(shown[0].named, "the nearest keeps its name");
    assert!(shown.iter().skip(1).all(|label| !label.named), "{shown:?}");
}

/// A label is full up close, fades out to nothing at the far range, and the animal fought or looked
/// at shows brighter than the rest.
#[test]
fn labels_fade_with_distance() {
    assert_eq!(faint(FULL - 1.0, true), 1.0);
    assert!((faint(FULL - 1.0, false) - DIM).abs() < 1e-6);
    assert!(faint(0.5 * (FULL + FAR), false) < DIM);
    assert_eq!(faint(FAR + 1.0, true), 0.0);
    assert!(faint(30.0, true) > faint(30.0, false));
}
