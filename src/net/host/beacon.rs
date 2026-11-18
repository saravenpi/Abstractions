//! The number of players the beacon announces, kept up to date at most once a second.

use std::time::Duration;

use bevy::prelude::*;

use super::Hosted;
use crate::net::Players;

/// Least time between two updates of the beacon.
pub const UPDATE_EVERY: Duration = Duration::from_secs(1);

/// Announces how many players the world holds: the others and the host.
pub fn count(
    time: Res<Time<Real>>,
    mut every: Local<Option<Timer>>,
    players: Res<Players>,
    mut hosted: ResMut<Hosted>,
) {
    let timer = every.get_or_insert_with(|| Timer::new(UPDATE_EVERY, TimerMode::Repeating));
    if !timer.tick(time.delta()).just_finished() {
        return;
    }
    let held = u8::try_from(players.0.len() + 1).unwrap_or(u8::MAX);
    if hosted.beacon.players == held {
        return;
    }
    hosted.beacon.players = held;
    if let Some(announcer) = &hosted.announcer {
        announcer.update(hosted.beacon.clone());
    }
}
