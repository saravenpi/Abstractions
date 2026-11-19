//! A remote robot whose records take 100 ms to arrive: the buffer measures that delay on the
//! universe clock and draws the robot on its path, that much further in the past.

use abstractions_net::io::ClientEvent;
use abstractions_net::wire::Frame;
use bevy::math::DVec3;

use super::stage::{app, frame_at, ground, linked, puppets, robot};
use crate::net::state::track::Remotes;
use crate::sim::PlayerId;

/// Universe time the walk starts at.
const START: f64 = 800.0;

/// Seconds every record takes to arrive.
const LATENCY: f64 = 0.1;

/// Returns where the player is at `time` and how fast it goes: east from the ground spot at 3 m/s.
fn walk(time: f64) -> (DVec3, DVec3) {
    let (_, spot) = ground();
    let east = DVec3::Y.cross(spot).normalize() * 3.0;
    (spot + east * (time - START), east)
}

/// Each record of a walk arrives 100 ms after it was taken; the robot is drawn on the walk
/// within 2 cm, at the time the buffer says, and the buffer measures the 100 ms.
#[test]
fn a_late_robot_is_drawn_on_its_path() {
    let mut app = app("late");
    let script = linked(&mut app);
    let (home, _) = ground();
    let at = |step: u32| START + f64::from(step) / 30.0;
    for step in 0..90 {
        let batch = robot(PlayerId(9), Frame::Surface(home), at(step), walk(at(step)));
        script
            .lock()
            .unwrap()
            .events
            .push_back(ClientEvent::Datagram(batch));
        frame_at(&mut app, at(step) + LATENCY);
    }
    let buffer = &app.world().resource::<Remotes>().robots[&PlayerId(9)].buffer;
    let (latency, delay) = (buffer.latency(), buffer.delay());
    assert!((latency - LATENCY).abs() < 1e-3, "measured {latency} s");
    let drawn = at(89) + LATENCY - latency - delay;
    let seen = puppets(&mut app);
    let off = seen[0].pos.distance(walk(drawn).0);
    assert!(off < 0.02, "{off} m off the walk");
}
