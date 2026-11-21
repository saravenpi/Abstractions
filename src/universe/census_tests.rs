//! Census of life among the planets, and the cost of the home search over many seeds.

use std::time::Instant;

use super::range_tests::sample;
use super::*;
use crate::life;

/// Seeds the cost of the home search is timed over.
const SEEDS: u64 = 100;

/// Prints how many of the planets of the sampled systems hold life, and how many are homely, and
/// asserts that life stays a minority.
#[test]
fn life_among_all_planets() {
    let (mut planets, mut alive, mut wet) = (0.0, 0.0, 0.0);
    for (u, star) in sample() {
        for p in u.system(star.id).planets {
            planets += 1.0;
            alive += f64::from(u8::from(life::of(&p, &star).is_some()));
            wet += f64::from(u8::from(p.record.liquids()[0] > 0.0));
        }
    }
    let (alive, wet) = (100.0 * alive / planets, 100.0 * wet / planets);
    println!("planets with life {alive:.1} %, with liquid water {wet:.1} %, of {planets}");
    assert!(alive < 40.0, "{alive}");
}

/// Prints the slowest home search of the first seeds, and asserts that it stays under 100 ms.
#[test]
fn the_slowest_home_search() {
    let run = |seed: u64| {
        let start = Instant::now();
        let _ = Universe::new(seed);
        (start.elapsed(), seed)
    };
    let worst = (0..SEEDS).map(run).max().expect("seeds");
    println!(
        "slowest home search of {SEEDS} seeds: seed {} in {:?}",
        worst.1, worst.0
    );
    assert!(worst.0.as_millis() < 100, "{worst:?}");
}
