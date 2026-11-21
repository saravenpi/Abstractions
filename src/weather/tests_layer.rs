//! Tests of the depth of the cloud band: the slices the layer draws and the volume the whiteout
//! and the fly-through read are one profile, and there is no volume where no slice is drawn.

use super::puff::{SLICES, SOFT, drawn_at, reached, slice, volume};

/// The shader that draws the slices.
const SHADER: &str = include_str!("../render/clouds.wesl");

/// Returns bodies of the field from 0 to 1 in steps of a thousandth.
fn bodies() -> impl Iterator<Item = f32> {
    (0..=1000).map(|i| i as f32 / 1000.0)
}

/// The slices run from the base of the band to its top, and a slice draws a column exactly where
/// the volume holds cloud at the height of that slice.
#[test]
fn the_slices_draw_the_volume() {
    assert_eq!((slice(0), slice(SLICES - 1)), (0.0, 1.0));
    for body in bodies() {
        for k in 0..SLICES {
            let full = volume(slice(k), body) > 0.5;
            assert_eq!(drawn_at(k, body), full, "slice {k} over a body of {body}");
        }
    }
    assert!(SHADER.contains("lone(cloud, cover) - 0.5 * clouds.slice.x"));
}

/// Returns whether the volume over a column that reaches the slice at `top`, or none, must be
/// empty at `share` of the band.
fn empty(top: Option<f32>, share: f32) -> bool {
    top.is_none_or(|top| share < -SOFT || share > top + SOFT)
}

/// No whiteout and no fly-through puff under the base slice, over the top slice, or over the
/// highest slice a column reaches; and none over a column no slice draws.
#[test]
fn there_is_no_volume_where_no_slice_is_drawn() {
    for body in bodies() {
        let top = reached(body);
        for share in (-200..=1200).map(|i| i as f32 / 1000.0) {
            if empty(top, share) {
                assert_eq!(volume(share, body), 0.0, "{share} over a body of {body}");
            }
        }
        let drawn = (0..SLICES).filter(|k| drawn_at(*k, body)).map(slice);
        let highest = drawn.fold(None, |most: Option<f32>, h| {
            Some(most.map_or(h, |m| m.max(h)))
        });
        assert_eq!(highest, top, "{body}");
    }
}
