//! Tests of the cover map: it holds the cloud field, what the shaders read from it is what the
//! weather reads, and it never steps when a snapshot is replaced.

use bevy::math::DVec3;

use super::map::{CHANNELS, CoverMap, budget, toward_second};
use crate::universe::{HOME_RADIUS, PlanetRecord};
use crate::weather::{FACE, Field, Patch, SNAPSHOT, TEXELS, texel};

/// Returns the wetness the tests give the ground in direction `dir`.
fn wet(dir: DVec3) -> f32 {
    (0.6 + 0.4 * dir.y) as f32
}

/// Returns the field of a planet like home and its empty map over the ground of [`wet`].
fn map() -> (Field, CoverMap) {
    let field = Field::new(&PlanetRecord::TEMPERATE, 5, HOME_RADIUS);
    let ground = (0..TEXELS).map(|index| wet(texel(index))).collect();
    (field, CoverMap::new(field, ground))
}

/// Returns the cover the shaders read from `map` at `seconds` over `patch`, which lies inside
/// one face: the four texels around the place, each read between its two channels.
fn drawn(map: &CoverMap, patch: &Patch, seconds: f64) -> f32 {
    let side = FACE as usize;
    let [column, row] = patch.corner.map(|c| c as usize);
    let at = |dx: usize, dy: usize| {
        let texel = patch.face * side * side + (row + dy) * side + column + dx;
        let held = &map.bytes()[texel * CHANNELS..][..CHANNELS];
        let (first, second) = (f32::from(held[0]) / 255.0, f32::from(held[1]) / 255.0);
        first + (second - first) * toward_second(seconds)
    };
    patch.blend([at(0, 0), at(1, 0), at(0, 1), at(1, 1)])
}

/// Returns directions well inside the faces of the grid.
fn places() -> impl Iterator<Item = DVec3> {
    let lean = [(0.3, -0.2), (-0.55, 0.4), (0.1, 0.62)];
    let axes = [DVec3::X, DVec3::NEG_Y, DVec3::Z, DVec3::NEG_X];
    axes.into_iter().flat_map(move |axis| {
        let (a, b) = (
            axis.any_orthonormal_vector(),
            axis.cross(axis.any_orthonormal_vector()),
        );
        lean.map(|(x, y)| (axis + a * x + b * y).normalize())
    })
}

/// What the shaders read from the map is the cover the weather reads at the camera, at every
/// place and second tried, to within the 8 bits of the texture.
#[test]
fn the_map_draws_the_cover_the_weather_reads() {
    let (field, mut map) = map();
    for seconds in [3.0, 15.9, 16.0, 40.5, 1000.0, 1015.99] {
        map.advance(seconds, 0);
        for dir in places() {
            let patch = Patch::around(dir);
            let read = field.cover(&patch, patch.dirs().map(wet), seconds);
            let shown = drawn(&map, &patch, seconds);
            assert!((read - shown).abs() < 0.004, "{read} {shown} at {seconds}");
        }
    }
}

/// Followed frame by frame over four snapshot periods, a few texels of the next snapshot at
/// a time, the map ends up byte for byte the map built at once, and what it draws never
/// moves by more than a sliver between two frames: replacing a snapshot does not show.
#[test]
fn replacing_a_snapshot_never_steps() {
    let (_, mut map) = map();
    let patches: Vec<Patch> = places().map(|dir| Patch::around(dir)).collect();
    let frame = 1.0 / 60.0;
    map.advance(0.0, 0);
    let mut last: Vec<f32> = patches.iter().map(|p| drawn(&map, p, 0.0)).collect();
    let mut changes = 0;
    for step in 1..=(4.2 * SNAPSHOT / frame) as u32 {
        let seconds = f64::from(step) * frame;
        changes += u32::from(map.advance(seconds, budget(frame as f32)));
        for (patch, before) in patches.iter().zip(&mut last) {
            let now = drawn(&map, patch, seconds);
            assert!((now - *before).abs() < 0.002, "at {seconds}");
            *before = now;
        }
    }
    assert_eq!(changes, 4);
    let (_, mut fresh) = super::tests::map();
    fresh.advance(4.2 * SNAPSHOT, 0);
    assert_eq!(map.bytes(), fresh.bytes());
}

/// A jump in universe time rebuilds both snapshots at once, and a frame budget always
/// finishes a snapshot well within its period, however slow the frames.
#[test]
fn a_jump_in_time_rebuilds_the_map() {
    let (_, mut map) = map();
    assert!(map.advance(100.0, 0));
    assert!(!map.advance(101.0, 64));
    assert!(map.advance(90_000.0, 0));
    let (_, mut fresh) = super::tests::map();
    fresh.advance(90_000.0, 0);
    assert_eq!(map.bytes(), fresh.bytes());
    for rate in [4.0_f32, 30.0, 144.0] {
        let frames = SNAPSHOT as f32 * rate;
        assert!(
            budget(1.0 / rate) as f32 * frames > 4.0 * TEXELS as f32,
            "{rate}"
        );
    }
}
