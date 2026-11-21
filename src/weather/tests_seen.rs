//! Tests of the puffs as the shells draw them: a puff keeps its size and the sky its cover at any
//! distance, and the cover rises smoothly toward the horizon with no step or band.

use bevy::math::Vec3;

use super::puff::{CELL, Spot, drawn};

/// Radians one pixel spans at the default field of view of 75 degrees over 360 rows.
fn pixel() -> f32 {
    2.0 * 37.5f32.to_radians().tan() / 360.0
}

/// Returns whether the shell draws cloud at each of `points` under `cover`, seen with `facing`
/// and `footprint` cells per pixel.
fn cut(points: &[Vec3], cover: f32, view: (f32, f32)) -> Vec<bool> {
    points
        .iter()
        .map(|p| drawn(Spot::same(*p), cover, view) >= 0.5)
        .collect()
}

/// Returns the share of `cloud` that is true and the mean length of its runs of cloud, in steps.
fn runs(cloud: &[bool]) -> (f32, f32) {
    let share = cloud.iter().filter(|c| **c).count() as f32 / cloud.len() as f32;
    let starts = cloud
        .windows(2)
        .filter(|pair| !pair[0] && pair[1])
        .count()
        .max(1);
    (share, share * cloud.len() as f32 / starts as f32)
}

/// Seen overhead from 5 km to 400 km away, the sky holds as much cloud as near, and a puff along
/// a line through the field is never cut shorter: far detail fades, the puff it belongs to stays.
/// The fade of the detail to its mean shrank far puffs to specks, then to nothing.
#[test]
fn a_puff_keeps_its_size_with_distance() {
    let line = Vec3::new(0.6, 0.3, 0.74);
    let points: Vec<Vec3> = (0..20_000)
        .map(|i| Vec3::new(3.1, 7.7, 1.3) + line * (0.02 * i as f32))
        .collect();
    for cover in [0.3, 0.5, 0.7] {
        let (near, size) = runs(&cut(&points, cover, (1.0, 5000.0 * pixel() / CELL)));
        assert!(near > 0.01, "{cover}: no cloud near");
        for distance in [30_000.0, 120_000.0, 400_000.0] {
            let footprint = distance * pixel() / CELL;
            let (far, length) = runs(&cut(&points, cover, (1.0, footprint)));
            assert!(
                (far - near).abs() < 0.15 * near + 0.01,
                "{cover} at {distance} m: {far} for {near}"
            );
            assert!(
                length > 0.8 * size && length < 2.0 * size,
                "{cover} at {distance} m: a puff of {length} steps for {size}"
            );
        }
    }
}

/// Returns the sine of the angle between the shell and a line of sight from the ground at
/// `elevation` degrees, and the cells per pixel it sees there along its longest axis, for a
/// planet of the size of the Earth under a shell 5 km up.
fn sight(elevation: f32) -> (f32, f32) {
    let (ground, shell) = (6_371_000.0f32, 6_376_000.0f32);
    let (sin, cos) = elevation.to_radians().sin_cos();
    let b = ground * sin;
    let run = -b + (b * b + shell * shell - ground * ground).sqrt();
    let hit = Vec3::new(run * cos, ground + run * sin, 0.0);
    let facing = hit.normalize().dot(Vec3::new(cos, sin, 0.0));
    (facing, run * pixel() / CELL / facing.max(1.0e-3))
}

/// Looking from the zenith down to the horizon a quarter of a degree at a time, the cover of the
/// shell grows smoothly as the line of sight crosses more of the depth of the puffs: no step
/// between two rows, no band, and a clear sky stays clear to the horizon.
#[test]
fn the_horizon_has_no_band() {
    let points: Vec<Vec3> = (0..1000)
        .map(|i| {
            let i = i as f32;
            Vec3::new((i * 0.731) % 97.0, (i * 0.377) % 89.0, (i * 0.193) % 83.0) * 4.0
        })
        .collect();
    for cover in [0.0, 0.3, 0.5, 0.8] {
        let shares: Vec<f32> = (0..=360)
            .map(|step| runs(&cut(&points, cover, sight(90.0 - 0.25 * step as f32))).0)
            .collect();
        let worst = shares
            .windows(2)
            .map(|pair| (pair[1] - pair[0]).abs())
            .fold(0.0, f32::max);
        assert!(worst < 0.05, "{cover}: the cover steps by {worst}");
        assert!(
            shares[360] >= shares[0],
            "{cover}: {} at the horizon",
            shares[360]
        );
    }
    let clear: Vec<bool> = cut(&points, 0.0, sight(0.0));
    assert!(runs(&clear).0 < 0.02);
}
