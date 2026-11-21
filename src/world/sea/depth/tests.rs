//! Tests of the depth map addressing, refresh sets and texel values.

use std::collections::HashSet;

use super::*;

/// Unpacks the depth in meters the way the shader does.
fn depth_of(bytes: [u8; 4]) -> f32 {
    f32::from(u16::from_be_bytes([bytes[0], bytes[1]])) / 65535.0 * (DEEP + LAND) - LAND
}

/// A texel keeps its slot when the map moves, and texels one map apart share a slot.
#[test]
fn offsets_wrap_on_both_axes() {
    assert_eq!(offset(IVec2::ZERO), 0);
    assert_eq!(offset(IVec2::new(1, 0)), 4);
    assert_eq!(offset(IVec2::new(0, 1)), 4 * SIZE as usize);
    assert_eq!(offset(IVec2::new(-1, -1)), 4 * (SIZE * SIZE - 1) as usize);
    assert_eq!(
        offset(IVec2::new(SIZE + 3, -SIZE + 5)),
        offset(IVec2::new(3, 5))
    );
}

/// Every texel of a map has its own slot, so the torus never overwrites a live texel.
#[test]
fn a_map_fills_every_slot_once() {
    let all = entered(None, IVec2::new(50_000, -70_001));
    let slots: HashSet<usize> = all.iter().map(|&t| offset(t)).collect();
    assert_eq!(all.len(), (SIZE * SIZE) as usize);
    assert_eq!(slots.len(), all.len());
    assert_eq!(all[0], IVec2::new(50_000, -70_001));
}

/// A step refreshes one column, a diagonal step one column and one row, a long jump everything.
#[test]
fn a_move_refreshes_only_what_scrolled_in() {
    let from = IVec2::new(10, -4);
    let side = SIZE as usize;
    assert!(entered(Some(from), from).is_empty());
    let step = entered(Some(from), from + IVec2::X);
    assert_eq!(step.len(), side);
    assert!(step.iter().all(|t| t.x == from.x + SIZE / 2));
    assert_eq!(
        entered(Some(from), from + IVec2::new(-1, 1)).len(),
        2 * side - 1
    );
    assert_eq!(entered(Some(from), from + IVec2::new(0, 3)).len(), 3 * side);
    assert_eq!(
        entered(Some(from), from + IVec2::new(SIZE, 0)).len(),
        side * side
    );
}

/// What scrolled in is inside the new map and outside the old one.
#[test]
fn entered_texels_are_new_to_the_map() {
    let (old, new) = (IVec2::new(3, 3), IVec2::new(-9, 20));
    let fresh = entered(Some(old), new);
    assert!(fresh.iter().all(|&t| inside(new, t) && !inside(old, t)));
    assert!(inside(new, new - IVec2::splat(SIZE / 2)));
    assert!(!inside(new, new + IVec2::splat(SIZE / 2)));
}

/// Depths survive packing to a millimeter, and out of range values clamp.
#[test]
fn depth_packs_into_two_bytes() {
    for depth in [-3.2, 0.0, 0.013, 0.5, 7.77, 15.9] {
        assert!(
            (depth_of(encode(depth, false)) - depth).abs() < 1e-3,
            "{depth}"
        );
    }
    assert_eq!(encode(40.0, false), BLANK);
    assert!((depth_of(encode(-90.0, true)) + LAND).abs() < 1e-3);
    assert_eq!(encode(1.0, true)[2], 255);
}

/// The spawn point is dry land, every texel matches the terrain and its ponds, the open sea is deep.
#[test]
fn texels_tell_sea_from_land_and_ponds() {
    let terrain = Terrain::new(1);
    let centre = centre_texel(terrain.find_spawn());
    assert!(depth_of(sample(&terrain, centre)) < 0.0);
    for texel in entered(None, centre) {
        let (p, bytes) = (texel.as_vec2() * TEXEL, sample(&terrain, texel));
        let depth = (-terrain.height(p.x, p.y)).clamp(-LAND, DEEP);
        assert!((depth_of(bytes) - depth).abs() < 1e-3, "{p}");
        assert_eq!(bytes[2] == 255, terrain.pond_at(p.x, p.y), "{p}");
    }
    let sea = (0..4000)
        .map(|i| IVec2::new(i, 0))
        .find(|t| terrain.height(t.x as f32 * TEXEL, 0.0) < -10.0);
    assert!(sea.is_some_and(|t| depth_of(sample(&terrain, t)) > 10.0));
}

/// A texel is wet where its bed lies under the sea and dry at the waterline and above it; a
/// texel still waiting for its sample reads as deep water.
#[test]
fn wet_texels_lie_under_the_sea() {
    assert!(wet(&encode(0.01, false)) && wet(&encode(9.0, true)) && wet(&BLANK));
    assert!(!wet(&encode(0.0, false)) && !wet(&encode(-2.0, false)));
    assert!(!wet(&[]));
}
