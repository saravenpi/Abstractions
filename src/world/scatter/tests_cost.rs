//! The cost of scatter on a globe against the legacy plane, and of a living planet against the rules of home,
//! each measured in one run.

use std::sync::Arc;
use std::time::Instant;

use super::zones::{LUSH, Tile, Zone, chunks, hold};
use super::*;
use crate::life::flora::fixture::garden;
use crate::world::globe::Globe;

/// Returns `want` chunks of the legacy plane of `t` whose 25 sample points are all land.
fn flat_land(t: &Terrain, want: usize) -> Vec<IVec2> {
    let dry = |c: &IVec2| {
        let o = chunk_center(*c);
        (0..25)
            .map(|i| o + Vec2::new((i % 5) as f32 - 2.0, (i / 5) as f32 - 2.0) * 15.0)
            .all(|p| t.height(p.x, p.y) > 0.5)
    };
    let all = (-40..40).flat_map(|x| (-40..40).map(move |y| IVec2::new(x, y)));
    all.filter(dry).take(want).collect()
}

/// Returns the best of five timings of `work`, in seconds, and what it returns.
fn timed(work: impl Fn() -> usize) -> (f64, usize) {
    let once = || {
        let start = Instant::now();
        let n = work();
        (start.elapsed().as_secs_f64(), n)
    };
    (0..5).map(|_| once()).fold(
        (f64::MAX, 0),
        |best, run| if run.0 < best.0 { run } else { best },
    )
}

/// Returns how many props `scatter` holds.
fn count(scatter: &Scatter) -> usize {
    scatter.props.len() + scatter.grass.len()
}

/// Scatter of one land chunk costs less than 1.2 times as much on a globe as on the legacy plane, measured in
/// the same run on the home globe, at a face centre where its chunks are largest, and so does the call of a
/// chunk builder that hands in the heights it already has.
#[test]
fn globe_scatter_costs_no_more_than_flat() {
    let _book = hold();
    let flat = Terrain::new(5);
    let globe = Arc::new(Globe::new(5, 6000.0, 22.0));
    let (plane, tiles) = (flat_land(&flat, 12), chunks(&globe, Zone::Centre, 25, 12));
    assert_eq!((plane.len(), tiles.len()), (12, 12));
    let grids = tiles.iter().map(|t| chunk_heights(&t.terrain, t.coord, 1));
    let grids: Vec<_> = tiles.iter().zip(grids).collect();
    let on_globe = |t: &Tile| count(&scatter_chunk(&t.terrain, t.coord));
    let over = |(t, h): &(&Tile, Vec<Vec<f32>>)| count(&scatter_over(&t.terrain, t.coord, h));
    let legacy = timed(|| plane.iter().map(|c| count(&scatter_chunk(&flat, *c))).sum());
    let alone = timed(|| tiles.iter().map(on_globe).sum());
    let handed = timed(|| grids.iter().map(over).sum());
    let (ratio, least) = (alone.0 / legacy.0, handed.0 / legacy.0);
    println!(
        "COST flat {legacy:?} globe {alone:?} with heights {handed:?}: {ratio:.2}, {least:.2}"
    );
    assert!(legacy.1 > 2000 && alone.1 > 2000, "too few props to time");
    let fast = ratio < 1.2 && least < 1.2;
    assert!(
        fast,
        "globe scatter costs {ratio} times the flat one, {least} with heights"
    );
}

/// Scatter of a planet that plants its own producers of all eight architectures costs less than 1.5 times the
/// same globe under the rules of home, per chunk, measured in the same run on its largest chunks. The best of
/// five runs finds every grown mesh in the book, as each later chunk does.
#[test]
fn living_scatter_costs_no_more_than_home_rules() {
    let _book = hold();
    let all = garden(9, &[0, 1, 2, 3, 4, 5, 6, 7], vec![]);
    let (plain, living) = (
        Arc::new(Globe::new(LUSH, 5000.0, 9.81)),
        Arc::new(Globe::new(LUSH, 5000.0, 9.81).with_life(Some(all))),
    );
    let (home, alien) = (
        chunks(&plain, Zone::Centre, 15, 12),
        chunks(&living, Zone::Centre, 15, 12),
    );
    assert_eq!((home.len(), alien.len()), (12, 12));
    let on = |tiles: &[Tile]| {
        timed(|| {
            tiles
                .iter()
                .map(|t| count(&scatter_chunk(&t.terrain, t.coord)))
                .sum()
        })
    };
    let (rules, life) = (on(&home), on(&alien));
    let ratio = life.0 / rules.0;
    println!("COST home rules {rules:?} living {life:?}: {ratio:.2}");
    assert!(rules.1 > 2000 && life.1 > 1000, "too few props to time");
    assert!(
        ratio < 1.5,
        "living scatter costs {ratio} times the rules of home"
    );
}
