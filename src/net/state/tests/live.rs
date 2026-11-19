//! A live run: the headless app joins a real server on this machine while the bot example
//! walks three players there, follows the server clock and draws three robot puppets that
//! move.
//!
//! Start the server and the bots first, then run this test by name with `--ignored`. The
//! address is `ABSTRACTIONS_LIVE`, `127.0.0.1:24760` without it.

use std::time::{Duration, Instant};

use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::stage::{app, puppets};
use crate::net::{JoinServer, NetStats, Session};
use crate::space::{Anchor, Clock, Frame, Location, Travel};
use crate::universe::BodyId;

/// Longest the test waits for the server and the bots.
const PATIENCE: Duration = Duration::from_secs(20);

/// Runs frames 10 ms apart until `done` holds, or fails after [`PATIENCE`]. Each frame moves
/// universe time on by the time it took, as the clock of a game being played does; the
/// session clock then steers it toward the server's.
fn until(app: &mut App, what: &str, done: impl Fn(&mut App) -> bool) {
    let end = Instant::now() + PATIENCE;
    let mut last = Instant::now();
    while !done(app) {
        assert!(Instant::now() < end, "timed out waiting for {what}");
        let now = Instant::now();
        app.world_mut().resource_mut::<Clock>().seconds += (now - last).as_secs_f64();
        last = now;
        app.update();
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// Puts the player on foot on `body`, with the anchor at its centre.
fn stand_on(app: &mut App, body: BodyId) {
    let anchor = Anchor {
        frame: Frame::Surface(body),
        origin: DVec3::ZERO,
        rotation: DQuat::IDENTITY,
        flat: false,
    };
    let location = Location {
        body,
        travel: Travel::OnFoot,
    };
    app.insert_resource(anchor).insert_resource(location);
}

/// Joins the live server, stands on the body the bots walk on, and sees three robots move.
#[test]
#[ignore = "needs a server and three bots on this machine"]
fn three_bots_walk_on_a_live_server() {
    let address = std::env::var("ABSTRACTIONS_LIVE").unwrap_or("127.0.0.1:24760".to_string());
    let mut app = app("live");
    let join = JoinServer {
        address,
        expected_key: None,
        password: None,
    };
    app.world_mut().write_message(join);
    until(&mut app, "the welcome", |app| {
        matches!(app.world().resource::<Session>(), Session::Joined { .. })
    });
    stand_on(&mut app, BodyId::default());
    until(&mut app, "the server clock", |app| {
        app.world()
            .get_resource::<NetStats>()
            .is_some_and(|stats| stats.rtt_ms.is_some())
    });
    until(&mut app, "three robots", |app| puppets(app).len() == 3);
    let before = puppets(&mut app);
    let end = Instant::now() + Duration::from_secs(1);
    until(&mut app, "a second", |_| Instant::now() > end);
    let after = puppets(&mut app);
    assert_eq!(after.len(), 3);
    for robot in &after {
        let gaps = before.iter().map(|was| was.pos.distance(robot.pos));
        let moved = gaps.fold(f64::INFINITY, f64::min);
        println!("robot at {:.2?} moved {moved:.2} m in a second", robot.pos);
        assert!(moved > 0.5, "a robot moved only {moved} m");
    }
}
