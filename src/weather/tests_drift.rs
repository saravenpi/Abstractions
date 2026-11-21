//! Tests of the drift of the puffs: they ride east round the spin axis at the speed of the wind,
//! smoothly at every second of play, with no jump where the old drift started over and no loss
//! of precision after years of universe time.

use bevy::math::{DQuat, DVec3, Vec3};

use super::puff::{CELL, SHEAR, WIND, density_at, drift, spot_at};

/// Noise cells along the radius of a shell 5 km over a planet of the size of the Earth.
const CELLS: f32 = 6_376_000.0 / CELL;

/// Seconds of universe time the drift is looked at: the start, an hour, either side of the 36 h
/// at which the old drift started over, a month, 10,000 hours of play, and 31 years.
const TIMES: [f64; 7] = [0.0, 3600.0, 131_071.5, 131_072.5, 2_592_000.0, 3.6e7, 1.0e9];

/// A puff seen at a place of the equator is found a minute later where the wind has carried it,
/// 240 m to the east, by both its broad octave and, a quarter faster, its finer ones.
#[test]
fn the_puffs_ride_east_with_the_wind() {
    let p = Vec3::Z * CELLS;
    let metres = WIND * 60.0;
    let east = |share: f64| {
        let angle = metres * share / (f64::from(CELLS) * f64::from(CELL));
        (DQuat::from_rotation_y(angle) * p.as_dvec3()).as_vec3()
    };
    let (before, base, fine) = (
        spot_at(p, drift(1000.0, CELLS)),
        spot_at(east(1.0), drift(1060.0, CELLS)),
        spot_at(east(1.0 + SHEAR), drift(1060.0, CELLS)),
    );
    assert!(
        base.base.distance(before.base) < 2.0e-3,
        "{base:?} {before:?}"
    );
    assert!(
        fine.fine.distance(before.fine) < 2.0e-3,
        "{fine:?} {before:?}"
    );
    assert!(east(1.0).x > p.x);
}

/// At every time of [`TIMES`], one second of play moves the noise under a fixed place by the
/// wind's 4 m and no more, to the 3 m the 32-bit turn holds: no jump.
#[test]
fn the_drift_never_jumps() {
    let p = Vec3::new(0.6, 0.0, 0.8) * CELLS;
    let step = (WIND / f64::from(CELL)) as f32;
    for t in TIMES {
        let (now, then) = (
            spot_at(p, drift(t, CELLS)),
            spot_at(p, drift(t + 1.0, CELLS)),
        );
        let moved = now.base.distance(then.base);
        assert!(
            (moved - step).abs() < 1.5e-3,
            "{moved} cells in a second at {t}"
        );
        let slid = now.fine.distance(then.fine);
        assert!(
            (slid - step * (1.0 + SHEAR) as f32).abs() < 1.5e-3,
            "{slid} at {t}"
        );
        let turn = drift(t, CELLS);
        let unit = |c: f32, s: f32| (c.hypot(s) - 1.0).abs() < 1.0e-6;
        assert!(
            unit(turn.x, turn.y) && unit(turn.z, turn.w),
            "{turn} at {t}"
        );
    }
}

/// After any time of play, a frame changes the cloud over a place by as little as it does in
/// the first minute: no precision jump after hours or years.
#[test]
fn a_cloud_holds_after_hours_of_play() {
    let dirs = (0..200).map(|i| {
        let a = f64::from(i) * 0.37;
        DVec3::new(a.cos(), 0.2 * a.sin(), a.sin()).normalize()
    });
    for dir in dirs {
        for t in TIMES {
            let frame = (density_at(dir, CELLS, t + 1.0 / 60.0, 0.6)
                - density_at(dir, CELLS, t, 0.6))
            .abs();
            assert!(frame < 0.02, "{frame} in a frame at {t}");
        }
    }
}
