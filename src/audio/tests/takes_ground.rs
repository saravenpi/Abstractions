//! Takes of the ship near the ground for the offline renderer: the start on the pad, the
//! lift-off, the landing, the gear and the canopy.

use super::rig::parked;
use super::takes::ramp;
use crate::audio::ship::feed::{Ch, HullSound};

/// Returns a ship on its gear, engines running, 20 metres from a listener who stands beside it.
fn on_pad() -> HullSound {
    parked().with(Ch::Distance, 20.0).with(Ch::Aspect, 0.0)
}

/// A cold start on the pad: one second of a dark ship, then the engines start and settle to idle.
pub(super) fn start(t: f32) -> HullSound {
    on_pad().with(Ch::Power, f32::from(u8::from(t >= 1.0)))
}

/// A lift-off on the lift nozzles: idle, the nozzles come up from 2 to 4 seconds, the ship
/// climbs, and the gear folds 40 metres up.
pub(super) fn liftoff(t: f32) -> HullSound {
    let high = 0.8 * (t - 4.0).max(0.0).powi(2);
    on_pad()
        .with(
            Ch::Lift,
            0.55 * ramp(t, 2.0, 3.0) + 0.45 * ramp(t, 3.0, 4.0),
        )
        .with(Ch::Height, high)
        .with(Ch::Distance, (400.0 + high * high).sqrt())
        .with(Ch::Gear, 1.0 - ramp(high, 40.0, 40.0 + 1.6 * 12.0))
        .with(Ch::Glow, 0.6 * ramp(t, 2.0, 5.0))
        .with(Ch::Rcs, 0.3 * ramp(t, 4.0, 4.2))
}

/// A landing: the ship sinks on its lift nozzles, touches at 7 seconds, the nozzles shut, and
/// at 8.5 seconds the engines stop and the nozzles tick as they cool.
pub(super) fn landing(t: f32) -> HullSound {
    let high = 40.0 * (1.0 - ramp(t, 0.0, 7.0)).powi(2);
    on_pad()
        .with(Ch::Lift, 0.9 * (1.0 - ramp(t, 7.0, 7.6)))
        .with(Ch::Height, high)
        .with(Ch::Distance, (400.0 + high * high).sqrt())
        .with(Ch::Glow, 0.8 - 0.2 * ramp(t, 8.5, 12.0))
        .with(Ch::Rcs, 0.4 * (1.0 - ramp(t, 6.8, 7.0)))
        .with(Ch::Power, f32::from(u8::from(t < 8.5)))
}

/// The gear in flight at a low throttle: down from 1 to 3 seconds, up again from 5 to 7.
pub(super) fn gear(t: f32) -> HullSound {
    parked()
        .with(Ch::Height, 30.0)
        .with(Ch::Speed, 40.0)
        .with(Ch::Thrust, 0.2)
        .with(Ch::Gear, ramp(t, 1.0, 3.0) - ramp(t, 5.0, 7.0))
}

/// The canopy of a ship that is shut down on the pad: open, it shuts from 1 second, and opens
/// again from 5 seconds.
pub(super) fn canopy(t: f32) -> HullSound {
    on_pad()
        .with(Ch::Power, 0.0)
        .with(Ch::Canopy, 1.0 - ramp(t, 1.0, 2.4) + ramp(t, 5.0, 6.4))
}
