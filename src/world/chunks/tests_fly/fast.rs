//! Fast flight: at 300 m/s the ground ahead is ready before the fog lets it show.

use super::rig::Slow;
use super::{fly, rest, stage};

/// Speed of the flight in metres per second: a ship low over the ground.
const SPEED: f64 = 300.0;

/// Seconds measured.
const SECONDS: f64 = 4.0;

/// Most things that may still appear at once within the fog per second. The streamer popped
/// about sixty per second on this flight before it looked ahead.
const POPS: f64 = 12.0;

/// Largest share of the frames that may have unsettled ground ahead within the fog. It was
/// more than a third before the streamer looked ahead.
const LATE: f64 = 0.05;

/// Wall-clock frames per simulated frame: the test runs beside every other test, which share
/// its worker threads. What it checks does not depend on how fast they are: before the streamer
/// looked ahead the ground arrived inside the fog however fast it was built.
const WALL: f64 = 4.0;

/// A ship that flies low at 300 m/s over the home-sized planet at Medium meets ground that is
/// already settled when the fog lets it show, and sees almost nothing pop in.
#[test]
fn fast_flight_loads_beyond_the_fog() {
    let (mut app, mut route) = stage(6000.0, 4);
    app.insert_resource(Slow(WALL));
    route.speed = SPEED;
    rest(&mut app, &route);
    let row = fly(&mut app, &mut route, SECONDS);
    assert!(row.late <= LATE, "late: {}", row.text());
    assert!(row.popped() <= POPS, "pops: {}", row.text());
}
