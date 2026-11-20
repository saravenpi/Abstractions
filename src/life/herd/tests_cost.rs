//! The cost of herds: rolling a watch, and asking it for herds and hunts every frame.

use std::cell::Cell;
use std::time::{Duration, Instant};

use bevy::math::DVec3;

use super::fixtures::{light, meadow_life, sample};
use super::{Ground, Search, Watch};

/// Returns the shortest of `runs` timed runs of `work`.
fn best(runs: u32, mut work: impl FnMut()) -> Duration {
    (0..runs)
        .map(|_| {
            let start = Instant::now();
            work();
            start.elapsed()
        })
        .min()
        .unwrap_or_default()
}

/// Returns the search of `radius` metres around a point on land of the test planet at 4321 s.
fn search(radius: f64) -> Search {
    Search {
        at: DVec3::new(1.0, 0.3, 0.2).normalize(),
        radius,
        planet: 6000.0,
        time: 4321.0,
    }
}

/// Rolling a watch for a 1 km search, hunts planned, samples at most a few hundred sites and takes
/// under 2 ms in an optimised build.
#[test]
fn rolling_a_watch_is_cheap() {
    let life = meadow_life();
    let calls = Cell::new(0u32);
    let counted = |dir: DVec3| {
        calls.set(calls.get() + 1);
        sample(dir, 1.0)
    };
    let ground = Ground {
        sample: &counted,
        light: &light,
    };
    let each = best(20, || {
        std::hint::black_box(Watch::new(&life, &ground, &search(1000.0)));
    });
    let samples = calls.get() / 20;
    assert!(samples <= 400, "{samples} samples");
    let limit = [2_000, 20_000][usize::from(cfg!(debug_assertions))];
    assert!(each.as_micros() < limit, "{each:?}, {samples} samples");
}

/// Over two seconds of frames, asking a watch for the herds within 300 m, or for one herd, takes under
/// 20 microseconds per herd, and its hunts and carcasses under 20 microseconds a frame.
#[test]
fn asking_a_watch_is_cheap() {
    let life = meadow_life();
    let ground = Ground {
        sample: &|dir| sample(dir, 1.0),
        light: &light,
    };
    let mut watch = Watch::new(&life, &ground, &search(300.0));
    let frames: Vec<f64> = (0..120).map(|f| 4321.0 + f64::from(f) / 60.0).collect();
    let herds = watch.herds(&life, &ground, frames[0]);
    assert!(herds.len() >= 2, "{} herds", herds.len());
    let all = best(5, || {
        frames.iter().for_each(|t| {
            std::hint::black_box(watch.herds(&life, &ground, *t));
        });
    }) / (120 * herds.len() as u32);
    let one = best(5, || {
        frames.iter().for_each(|t| {
            std::hint::black_box(watch.herd(&life, &ground, herds[0].id, *t));
        });
    }) / 120;
    let shown = best(5, || {
        frames.iter().for_each(|t| {
            std::hint::black_box((watch.hunts(&life, *t), watch.carcasses(&life, *t)));
        });
    }) / 120;
    let limit = Duration::from_micros(20);
    assert!(
        all < limit && one < limit,
        "{all:?} per herd, {one:?} for one herd"
    );
    assert!(shown < limit, "{shown:?} for hunts and carcasses");
}
