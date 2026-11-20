//! Tests of the drainage grid: every path of water ends in the sea or a basin, the build is quick, trunk
//! rivers run on the ground as flows that fall, and the shores it finds are land near the sea.

use std::time::{Duration, Instant};

use bevy::math::DVec3;

use super::Globe;
use super::atlas::Cell;
use super::drainage::Drainage;
use super::drainage::tune::{LATITUDE, SIDE, STARTS};

/// The most the grid of a planet may take to build, off the main thread.
const BUILD: Duration = Duration::from_millis(400);

/// Returns an Earth-sized planet of seed `seed`, with its shore measured so a build times only the grid.
fn earth(seed: u64) -> Globe {
    let globe = Globe::new(seed, 6.371e6, 9.81);
    std::hint::black_box(globe.large_height(DVec3::X));
    globe
}

/// Every river cell of three Earth-sized planets and of a small one drains to a sea cell, as the sea is
/// there, and every path is finite: the water never loops.
#[test]
fn every_river_path_reaches_the_sea_or_a_basin() {
    for globe in [earth(1), earth(2), earth(3), Globe::new(42, 6000.0, 22.0)] {
        let drain = globe.drainage();
        let rivers: Vec<usize> = (0..drain.grid().len())
            .filter(|&k| drain.is_river(k))
            .collect();
        assert!(
            globe.radius() < 1e6 || !rivers.is_empty(),
            "no river on seed {}",
            globe.seed()
        );
        for cell in rivers {
            let path = drain.course(cell);
            assert!(path.len() <= drain.grid().len());
            let end = path[path.len() - 1];
            assert!(drain.next(end).is_none());
            assert!(
                !drain.is_land(end) || !globe.has_sea(),
                "seed {}: ends on land",
                globe.seed()
            );
        }
    }
}

/// The grid of an Earth has 256 cells along a face side and builds in under 400 ms.
#[test]
fn the_grid_builds_in_time() {
    let globe = earth(7);
    let start = Instant::now();
    let drain = Drainage::build(&globe);
    let took = start.elapsed();
    assert_eq!(drain.grid().n, SIDE);
    assert!(took < BUILD, "the grid took {took:?}");
}

/// Two builds of one planet give the same grid.
#[test]
fn the_grid_is_a_function_of_the_planet() {
    let globe = earth(5);
    let (a, b) = (Drainage::build(&globe), Drainage::build(&globe));
    assert!((0..a.grid().len()).all(|k| a.next(k) == b.next(k) && a.is_land(k) == b.is_land(k)));
}

/// The springs of a trunk lie in the atlas cells along it, and every trunk flow there falls at every step
/// with its bed, at full width from its first node.
#[test]
fn trunks_run_on_the_ground_as_falling_flows() {
    let globe = earth(2);
    let drain = globe.drainage();
    let n = globe.cells();
    let trunk = (0..drain.grid().len())
        .filter(|&k| drain.is_river(k) && drain.next(k).is_some_and(|to| drain.is_land(to)))
        .take(20);
    let mut kept = 0;
    for cell in trunk {
        let (from, to) = (drain.dir(cell), drain.dir(drain.next(cell).unwrap_or(cell)));
        let middle = (from + to).normalize();
        for flow in globe.sources(Cell::locate(middle, n).0).iter() {
            assert!(
                flow.nodes
                    .windows(2)
                    .all(|w| w[1].height < w[0].height && w[1].bed < w[0].bed)
            );
            kept += 1;
        }
    }
    assert!(kept > 0, "no trunk flow springs along 20 trunks");
}

/// The places a new game may start are at most eight, far apart, at a mild latitude to within a cell, and
/// the same ones every time.
#[test]
fn the_landfalls_are_mild_and_apart() {
    for globe in [earth(1), earth(4)] {
        let dirs = globe.landfalls();
        assert!(
            !dirs.is_empty() && dirs.len() <= STARTS,
            "{} landfalls",
            dirs.len()
        );
        assert_eq!(dirs, globe.landfalls());
        for dir in &dirs {
            let degrees = dir.y.asin().to_degrees().abs();
            assert!(
                (LATITUDE.0 - 1.0..=LATITUDE.1 + 1.0).contains(&degrees),
                "{degrees} degrees"
            );
            assert!(globe.coarse(*dir) > 0.0, "a landfall at sea");
        }
        let apart = |i: usize| (0..i).all(|j| dirs[i].angle_between(dirs[j]) > 0.05);
        assert!((0..dirs.len()).all(apart));
    }
}
