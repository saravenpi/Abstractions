//! The flyover table: every speed, height, path, planet and render distance, one row each.

use super::{fly, rest, stage};

/// Speeds flown, in metres per second.
const SPEEDS: [f64; 4] = [30.0, 100.0, 300.0, 1000.0];

/// Heights flown, in metres: low over the ground and high over the sea.
const HEIGHTS: [f64; 2] = [30.0, 1000.0];

/// Seconds of each flight.
const SECONDS: f64 = 6.0;

/// Returns every flight of one world: height, whether it turns, and speed.
fn flights() -> Vec<(f64, bool, f64)> {
    let paths = HEIGHTS
        .into_iter()
        .flat_map(|height| [(height, false), (height, true)]);
    paths
        .flat_map(|(height, turns)| SPEEDS.map(|speed| (height, turns, speed)))
        .collect()
}

/// Returns the name of a flight: its world, height, path and speed.
fn label(world: &str, (height, turns, speed): (f64, bool, f64)) -> String {
    let path = ["line", "turn"][usize::from(turns)];
    format!("{world} {height:>4.0} m {path} {speed:>4.0} m/s")
}

/// Prints what a player sees load at every tested speed, height and path, on a 6 km and a 40 km
/// planet at Medium and High. Run it alone:
/// `cargo test --features dev flyover_table -- --ignored --nocapture`. The variable `FLY` keeps
/// the flights whose name holds its value, such as `FLY="6 km Medium   30 m line"`.
#[test]
#[ignore = "a benchmark of a quarter of an hour; run it alone"]
fn flyover_table() {
    let only = std::env::var("FLY").unwrap_or_default();
    let worlds = [6_000.0, 40_000.0]
        .into_iter()
        .flat_map(|radius| [(radius, "Medium", 4), (radius, "High", 8)]);
    for (radius, name, rings) in worlds {
        let world = format!("{:>2.0} km {name:>6}", radius / 1000.0);
        let kept = |flight: &(f64, bool, f64)| label(&world, *flight).contains(&only);
        let flights: Vec<_> = flights().into_iter().filter(kept).collect();
        if flights.is_empty() {
            continue;
        }
        let (mut app, mut route) = stage(radius, rings);
        for flight in flights {
            (route.height, route.turns, route.speed) = flight;
            rest(&mut app, &route);
            let row = fly(&mut app, &mut route, SECONDS);
            println!("fly {} | {}", label(&world, flight), row.text());
        }
    }
}
