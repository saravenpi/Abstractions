//! What the instruments of a ship read: the numbers of its telemetry, as the dashboard shows
//! them. A reading that has nothing true to show is `None`, and its instrument goes dark.

use bevy::prelude::*;

use super::pixels::Ink;
use super::tune::ALTITUDE_SPAN;
use crate::ship::cruise::top_speed;
use crate::ship::telemetry::{ShipTelemetry, WarpState};
use crate::ship::warp::fraction;
use crate::space::Frame;

/// Number of bars on the dashboard.
pub const BARS: usize = 5;

/// Number of lamps on the dashboard.
pub const LAMPS: usize = 6;

/// A bar of the dashboard. Each one shows one real number of the ship as a lit strip, in the
/// order of [`Readings::bars`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dial {
    /// The speed against the top speed the lever can ask for here, or in warp the warp speed
    /// against the fastest jump.
    Speed,
    /// The throttle lever. Dark in warp, where the lever asks for nothing.
    Throttle,
    /// The height above the ground on a logarithmic scale. Dark away from a surface.
    Altitude,
    /// How far the warp drive has spooled, full while the drive is engaged.
    Charge,
    /// How far the landing gear is down. Dark away from a surface, where no gear helps.
    Gear,
}

impl Dial {
    /// Every bar, in the order of [`Readings::bars`].
    pub const ALL: [Dial; BARS] = [
        Dial::Speed,
        Dial::Throttle,
        Dial::Altitude,
        Dial::Charge,
        Dial::Gear,
    ];
}

/// A lamp of the dashboard. It lights for one real state of the ship.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lamp {
    /// The flight assist is on.
    Assist,
    /// The ship rests on the ground or on the sea.
    Landed,
    /// The ship is inside the lock sphere of a body: the drive cannot spool.
    Close,
    /// The nose glows from an entry.
    Hot,
    /// The frame is under half of its integrity.
    Frame,
    /// The frame is under a quarter of its integrity.
    Failing,
}

impl Lamp {
    /// Every lamp, in the order of [`Readings::lamps`].
    pub const ALL: [Lamp; LAMPS] = [
        Lamp::Assist,
        Lamp::Landed,
        Lamp::Close,
        Lamp::Hot,
        Lamp::Frame,
        Lamp::Failing,
    ];

    /// Returns the legend of the lamp on the status display.
    pub fn legend(self) -> &'static str {
        ["STB", "LND", "CLS", "HOT", "FRM", "ERR"][self as usize]
    }

    /// Returns the ink the lamp is lit in: the lamps of the state of the ship are white, a
    /// warning is amber, a danger is red.
    pub fn ink(self) -> Ink {
        match self {
            Lamp::Assist | Lamp::Landed => Ink::Lit,
            Lamp::Close | Lamp::Hot | Lamp::Frame => Ink::Amber,
            Lamp::Failing => Ink::Red,
        }
    }
}

/// Everything the instruments of one ship measure right now.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Readings {
    /// What each bar measures from 0 to 1, in the order of [`Dial::ALL`].
    pub bars: [Option<f32>; BARS],
    /// Which lamps are lit, in the order of [`Lamp::ALL`].
    pub lamps: [bool; LAMPS],
    /// The speed in metres per second the speed display writes.
    pub speed: Option<f64>,
    /// The height in metres the altitude display writes.
    pub height: Option<f64>,
    /// Pitch up and roll to the right of the ship against the local up, in radians. Nothing
    /// where there is no horizon.
    pub attitude: Option<Vec2>,
    /// Unit direction from the ship to the destination of the course, in body axes.
    pub course: Option<Vec3>,
    /// Unit direction from the ship to the body it is beside, in body axes.
    pub body: Option<Vec3>,
    /// The acceleration the crew feels, in body axes, in metres per second squared.
    pub accel: Vec3,
}

/// Returns `value` as a share from 0 to 1, or nothing when it is not a number.
pub fn share(value: f64) -> Option<f32> {
    value.is_finite().then(|| value.clamp(0.0, 1.0) as f32)
}

/// Returns how full the altitude bar is at `height` metres: the logarithm of the height over
/// the logarithm of the full height. Nothing for a height that is not a number.
pub fn height_share(height: f64) -> Option<f32> {
    let height = height.is_finite().then_some(height.max(0.0))?;
    share((1.0 + height).ln() / (1.0 + ALTITUDE_SPAN).ln())
}

/// Returns the speed in metres per second that the speed display writes: the warp speed in
/// warp, else the speed over the surface, which in space is the orbital speed.
fn speed_of(t: &ShipTelemetry) -> f64 {
    match t.warp {
        WarpState::Active(warp) => warp,
        _ => t.surface_speed,
    }
}

/// Returns what the bars read from the telemetry `t` and the extension of the `gear`.
fn bars_of(t: &ShipTelemetry, gear: f64) -> [Option<f32>; BARS] {
    let surface = matches!(t.frame, Frame::Surface(_));
    let (load, flying) = match t.warp {
        WarpState::Active(warp) => (fraction(warp), false),
        _ => (t.surface_speed / top_speed(t.altitude_ground), true),
    };
    let charge = match t.warp {
        WarpState::Off => 0.0,
        WarpState::Spooling(ready) => ready,
        WarpState::Active(_) => 1.0,
    };
    let mut bars = [None; BARS];
    bars[Dial::Speed as usize] = share(load);
    bars[Dial::Throttle as usize] = share(t.throttle).filter(|_| flying);
    bars[Dial::Altitude as usize] = height_share(t.altitude_ground).filter(|_| surface);
    bars[Dial::Charge as usize] = share(charge);
    bars[Dial::Gear as usize] = share(gear).filter(|_| surface);
    bars
}

impl Readings {
    /// Returns what the instruments read from the telemetry of the ship and the extension of
    /// its `gear`: the bars, the speed and the height, and the lamps of the assist and of
    /// the ground. The speed is the one the HUD writes.
    pub fn of(telemetry: &ShipTelemetry, gear: f64) -> Self {
        let t = telemetry;
        let surface = matches!(t.frame, Frame::Surface(_));
        let mut lamps = [false; LAMPS];
        lamps[Lamp::Assist as usize] = t.assist;
        lamps[Lamp::Landed as usize] = t.landed;
        Self {
            bars: bars_of(t, gear),
            lamps,
            speed: Some(speed_of(t)).filter(|speed| speed.is_finite()),
            height: Some(t.altitude_ground).filter(|h| h.is_finite() && surface),
            ..Self::default()
        }
    }
}
