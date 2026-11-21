//! Takes of the ship in flight for the offline renderer: what the ship does over a few seconds,
//! as its state at each moment. `render` plays them and writes them down.

use super::rig::parked;
use crate::audio::ship::feed::{Ch, HullSound};

/// Returns how far `t` is from `from` to `to`, held between 0 and 1.
pub(super) fn ramp(t: f32, from: f32, to: f32) -> f32 {
    ((t - from) / (to - from)).clamp(0.0, 1.0)
}

/// Returns a ship in level flight 500 metres up, gear folded, 60 metres from a listener
/// outside, at the angle its jets are loudest at.
fn flying() -> HullSound {
    parked()
        .with(Ch::Height, 500.0)
        .with(Ch::Gear, 0.0)
        .with(Ch::Speed, 90.0)
}

/// Throttle in thick air, twice the air of home: idle, up to full, the boost, and back to idle.
pub(super) fn throttle(t: f32) -> HullSound {
    let up = ramp(t, 2.0, 4.0) - ramp(t, 9.5, 11.0);
    let boost = ramp(t, 6.5, 7.0) - ramp(t, 8.5, 9.0);
    let thrust = up + 0.73 * boost;
    flying()
        .with(Ch::Air, 2.0)
        .with(Ch::Thrust, thrust)
        .with(Ch::Glow, thrust.min(1.0))
        .with(Ch::Speed, 90.0 + 60.0 * thrust)
}

/// A pass at 150 metres per second, 60 metres up, 80 metres to the side of a standing
/// listener, from the left to the right. The ship is abeam at 4 seconds.
pub(super) fn flyby(t: f32) -> HullSound {
    let along = 150.0 * (t - 4.0);
    let far = (along * along + 80.0 * 80.0 + 60.0 * 60.0).sqrt();
    flying()
        .with(Ch::Thrust, 0.7)
        .with(Ch::Speed, 150.0)
        .with(Ch::Height, 60.0)
        .with(Ch::Distance, far)
        .with(Ch::Side, along / far)
        .with(Ch::Aspect, along / far)
}

/// Flight in thin cold air, a seventh of the air of home, where sound makes 240 metres per
/// second: the lever moves between a cruise and full, at Mach 1.7.
pub(super) fn thin(t: f32) -> HullSound {
    let thrust = 0.5 + 0.5 * (ramp(t, 2.0, 3.0) - ramp(t, 5.5, 6.5));
    flying()
        .with(Ch::Air, 0.15)
        .with(Ch::Sound, 240.0)
        .with(Ch::Speed, 400.0)
        .with(Ch::Height, 9000.0)
        .with(Ch::Thrust, thrust)
        .with(Ch::Rough, 0.15)
}

/// A burn in a vacuum: idle, full, idle, with two bursts of the attitude jets.
pub(super) fn vacuum(t: f32) -> HullSound {
    let thrust = ramp(t, 2.0, 3.5) - ramp(t, 6.5, 8.0);
    let jets = f32::from(u8::from((0.5..0.9).contains(&t) || (8.6..9.2).contains(&t)));
    flying()
        .with(Ch::Air, 0.0)
        .with(Ch::Speed, 0.0)
        .with(Ch::Height, 1.0e5)
        .with(Ch::Thrust, thrust)
        .with(Ch::Glow, thrust)
        .with(Ch::Rcs, jets)
}

/// The warp drive in a vacuum: it spools from 1 to 5 seconds, engages, cruises until 10 seconds
/// and drops out.
pub(super) fn warp(t: f32) -> HullSound {
    let drive = ramp(t, 1.0, 5.0) - ramp(t, 10.0, 10.8);
    vacuum(0.0).with(Ch::Thrust, 0.2).with(Ch::Warp, drive)
}
