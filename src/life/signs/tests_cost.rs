//! Cost tests of the signs: how many one search returns, and how long reading them takes.

use std::time::Instant;

use super::rig::{find, on, search, travels};
use super::tune::MOST;
use super::{Mark, near};
use crate::life::herd::Watch;
use crate::life::herd::fixtures::meadow_life;

/// A search around a herd returns at most [`MOST`] signs, and reading them from a warm watch takes
/// a few milliseconds at most, in a debug build.
#[test]
fn signs_per_search_are_bounded_and_cheap() {
    let life = meadow_life();
    let (herd, time) = find(&life, travels).expect("a travelling herd");
    let (count, prints, best) = on(|g| {
        let mut watch = Watch::new(&life, g, &search(herd.centre, 400.0, time));
        let mut out = Vec::with_capacity(MOST);
        let mut best = f64::MAX;
        for k in 0..5 {
            let start = Instant::now();
            near(
                &life,
                g,
                &mut watch,
                &search(herd.centre, 60.0, time + f64::from(k)),
                &mut out,
            );
            best = best.min(start.elapsed().as_secs_f64());
        }
        let prints = out.iter().filter(|s| s.mark == Mark::Print).count();
        (out.len(), prints, best)
    });
    println!(
        "{count} signs, {prints} prints, read in {:.2} ms",
        best * 1000.0
    );
    assert!(count <= MOST);
    assert!(best < 0.05, "signs took {:.1} ms", best * 1000.0);
}
