//! The warp drive system: the warp key, the spool, the cruise and what it tells the rest of the game.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::ShipEvent;
use super::body::Ship;
use super::helm::Helm;
use super::pilot::Autopilot;
use super::sky::Skies;
use super::warp::{Warp, advance, drop_out, request};
use crate::space::Frame;

/// Longest frame the drive moves the ship through, in seconds.
const MAX_FRAME: f64 = 0.1;

/// Switches the drive: spools it when it is off and may engage, stops it when it is on.
///
/// The drive never spools in the surface frame of a planet, which is inside its atmosphere.
fn switch(ship: &mut Ship, skies: &Skies) -> Option<ShipEvent> {
    let (state, surface) = (ship.warp, matches!(ship.frame, Frame::Surface(_)));
    match state {
        Warp::Off if surface || !request(ship, &skies.wells) => Some(ShipEvent::WarpRefused),
        Warp::Off => None,
        _ => {
            drop_out(ship, &skies.wells);
            None
        }
    }
}

/// Returns what a change of drive state means for the rest of the game.
fn change(before: Warp, after: Warp) -> Option<ShipEvent> {
    match (before, after) {
        (Warp::Off, Warp::Spooling { .. }) => Some(ShipEvent::WarpSpooling),
        (Warp::Spooling { .. }, Warp::Active { .. }) => Some(ShipEvent::WarpEngaged),
        (Warp::Spooling { .. }, Warp::Off) => Some(ShipEvent::WarpRefused),
        (Warp::Active { .. }, Warp::Off) => Some(ShipEvent::WarpDropped),
        _ => None,
    }
}

/// What the drive reads besides the ship and the helm.
#[derive(SystemParam)]
pub struct Bridge<'w> {
    /// Frame time.
    time: Res<'w, Time>,
    /// What surrounds the ship.
    skies: Res<'w, Skies>,
    /// The route, which sets the speed tier.
    autopilot: Res<'w, Autopilot>,
}

/// Runs the warp drive for the frame.
pub fn drive(
    bridge: Bridge,
    mut helm: ResMut<Helm>,
    mut ship: Single<&mut Ship>,
    mut events: MessageWriter<ShipEvent>,
) {
    let before = ship.warp;
    if before == Warp::Off && !helm.warp {
        return;
    }
    let skies = &*bridge.skies;
    let refused = std::mem::take(&mut helm.warp)
        .then(|| switch(&mut ship, skies))
        .flatten();
    advance(
        &mut ship,
        &skies.wells,
        bridge.autopilot.tier.cap(),
        bridge.time.delta_secs_f64().min(MAX_FRAME),
    );
    for event in refused.into_iter().chain(change(before, ship.warp)) {
        events.write(event);
    }
}
