//! What the instruments of a ship that someone else flies read: the numbers its records carry.

use super::readings::{BARS, Dial, Readings, height_share, share};

/// What the records of a remote ship say about its flight.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Seen {
    /// Speed in metres per second the speed display writes. Nothing in warp, where the record
    /// carries no speed in metres.
    pub speed: Option<f64>,
    /// The speed as a share of what the speed bar spans, from 0 to 1.
    pub load: f32,
    /// The throttle lever, from 0 to 1.
    pub throttle: f32,
    /// Metres above the ground, infinite away from a surface.
    pub clearance: f64,
    /// How far the warp drive has spooled, 1 in warp.
    pub drive: f32,
    /// How far the landing gear is down.
    pub gear: f32,
}

impl Readings {
    /// Returns what the instruments of a ship read from its `seen` record. It carries no
    /// attitude, no course and no frame, so those instruments stay dark.
    pub fn seen(seen: Seen) -> Self {
        let near = seen.clearance.is_finite();
        let warping = seen.drive >= 1.0;
        let mut bars = [None; BARS];
        bars[Dial::Speed as usize] = share(f64::from(seen.load));
        bars[Dial::Throttle as usize] = share(f64::from(seen.throttle)).filter(|_| !warping);
        bars[Dial::Altitude as usize] = height_share(seen.clearance).filter(|_| near);
        bars[Dial::Charge as usize] = share(f64::from(seen.drive));
        bars[Dial::Gear as usize] = share(f64::from(seen.gear)).filter(|_| near);
        Self {
            bars,
            speed: seen.speed.filter(|speed| speed.is_finite()),
            height: Some(seen.clearance).filter(|_| near),
            ..Self::default()
        }
    }
}
