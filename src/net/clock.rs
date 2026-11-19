//! Universe time in a session: a ping once a second, each pong fed to the estimate, and the
//! universe clock steered toward the server's each frame, right after it moved.

use std::time::Duration;

use abstractions_net::clock::{Clock as Estimate, Sample};
use abstractions_net::wire::{ClientMsg, ServerMsg};
use bevy::prelude::*;

use super::link::{FromServer, Link};
use crate::space::Clock;

/// Time between two pings.
pub const PING_EVERY: Duration = Duration::from_secs(1);

/// The estimate of the server's universe time and the number of the next ping.
#[derive(Resource, Debug, Default)]
pub struct SessionClock {
    /// The offset estimate and its slew.
    pub estimate: Estimate,
    /// Number of the next ping.
    next: u32,
}

/// What the HUD may show of the connection.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct NetStats {
    /// Smoothed round trip of the pings in milliseconds, once one came back.
    pub rtt_ms: Option<u32>,
}

/// Sends a ping every [`PING_EVERY`], carrying the local monotonic time.
pub fn ping(
    time: Res<Time<Real>>,
    mut every: Local<Option<Timer>>,
    mut link: ResMut<Link>,
    mut clock: ResMut<SessionClock>,
) {
    let timer = every.get_or_insert_with(|| Timer::new(PING_EVERY, TimerMode::Repeating));
    if !timer.tick(time.delta()).just_finished() {
        return;
    }
    let sent = link.now();
    link.send(ClientMsg::Ping {
        id: clock.next,
        sent,
    });
    clock.next = clock.next.wrapping_add(1);
}

/// Feeds every pong of the frame to the estimate and keeps the round trip for the HUD.
pub fn pong(
    mut from: MessageReader<FromServer>,
    link: Res<Link>,
    mut clock: ResMut<SessionClock>,
    mut stats: ResMut<NetStats>,
) {
    for FromServer(msg) in from.read() {
        let ServerMsg::Pong {
            sent, clock: at, ..
        } = *msg
        else {
            continue;
        };
        let sample = Sample {
            sent,
            received: link.now(),
            server: at,
        };
        clock.estimate.pong(sample);
    }
    let rtt = clock.estimate.offsets().rtt_smoothed();
    let rtt_ms = rtt.map(|seconds| (seconds * 1000.0).round() as u32);
    if stats.rtt_ms != rtt_ms {
        stats.rtt_ms = rtt_ms;
    }
}

/// Shows the server's universe time: it snaps on the first estimate and slews by at most 2
/// percent after. It runs after the universe clock moved, so every system of the frame reads
/// one time.
pub fn sync(
    time: Res<Time>,
    link: Res<Link>,
    mut estimate: ResMut<SessionClock>,
    mut clock: ResMut<Clock>,
) {
    let shown = clock.seconds;
    let seconds = estimate
        .estimate
        .advance(shown, time.delta_secs_f64(), link.now());
    if seconds != shown {
        clock.seconds = seconds;
    }
}
