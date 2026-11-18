//! The flight instruments: what each one reads from the ship, as text.

use bevy::prelude::*;

use super::units::{height, percent, speed};
use crate::computer::{RouteStatus, distance};
use crate::ship::{ShipTelemetry, WarpState};
use crate::space::Frame;

/// One instrument. Its row is hidden while it has nothing to show.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Gauge {
    /// Height above the ground.
    Altitude,
    /// Speed over the surface in the air, orbital speed in space.
    Speed,
    /// Main engine lever.
    Throttle,
    /// Flight assist.
    Assist,
    /// What the ship is measured against.
    Frame,
    /// Highest point of the orbit.
    Apoapsis,
    /// Lowest point of the orbit.
    Periapsis,
    /// Warp drive.
    Warp,
    /// Where the board computer is set to go.
    Destination,
}

impl Gauge {
    /// Every instrument, in display order, with its label.
    pub(super) const ALL: [(Gauge, &'static str); 9] = [
        (Gauge::Altitude, "ALT"),
        (Gauge::Speed, "SPD"),
        (Gauge::Throttle, "THR"),
        (Gauge::Assist, "ASSIST"),
        (Gauge::Frame, "FRAME"),
        (Gauge::Apoapsis, "APO"),
        (Gauge::Periapsis, "PERI"),
        (Gauge::Warp, "WARP"),
        (Gauge::Destination, "DEST"),
    ];
}

/// What the instruments are read from.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Flight {
    /// Instruments of the ship.
    pub telemetry: ShipTelemetry,
    /// How far along its route the ship is.
    pub route: RouteStatus,
    /// Name of the destination and distance to it in metres, when one is set.
    pub destination: Option<(String, f64)>,
}

/// Returns the text of the speed instrument: the warp speed in warp, the speed over the surface
/// in the frame of a planet, the orbital speed in space.
fn speed_text(flight: &Flight) -> String {
    let t = &flight.telemetry;
    match (t.warp, t.frame) {
        (WarpState::Active(warp), _) => format!("{} warp", speed(warp)),
        (_, Frame::Surface(_)) => format!("{} surface", speed(t.surface_speed)),
        (_, Frame::System(_)) => format!("{} orbital", speed(t.orbital_speed)),
        (_, Frame::Galaxy(_)) => speed(t.orbital_speed),
    }
}

/// Returns the text of the destination instrument.
fn destination_text(flight: &Flight) -> Option<String> {
    let (name, gap) = flight.destination.as_ref()?;
    Some(match flight.route {
        RouteStatus::Idle => format!("{name}  {}", distance(*gap)),
        route => format!("{name}  {}  {}", distance(*gap), route.label()),
    })
}

/// Returns what an instrument shows, or nothing when it has nothing to show: no orbit outside
/// space, no warp line while the drive is off, no destination until one is set.
pub fn reading(gauge: Gauge, flight: &Flight) -> Option<String> {
    let t = &flight.telemetry;
    let in_orbit = matches!(t.frame, Frame::System(_)) && !t.landed;
    let near = !matches!(t.frame, Frame::Galaxy(_)) && !matches!(t.warp, WarpState::Active(_));
    match gauge {
        Gauge::Altitude => near.then(|| height(t.altitude_ground)),
        Gauge::Speed => Some(speed_text(flight)),
        Gauge::Throttle => Some(percent(t.throttle)),
        Gauge::Assist => Some(assist_text(t).to_string()),
        Gauge::Frame => Some(frame_name(t.frame).to_string()),
        Gauge::Apoapsis => t.apoapsis.filter(|_| in_orbit).map(height),
        Gauge::Periapsis => t.periapsis.filter(|_| in_orbit).map(height),
        Gauge::Warp => warp_text(t.warp),
        Gauge::Destination => destination_text(flight),
    }
}

/// Returns the text of the assist instrument. A landed ship under the assist only takes off on
/// the rise key, so the instrument says it.
fn assist_text(t: &ShipTelemetry) -> &'static str {
    match (t.assist, t.landed) {
        (true, true) => "on - rise to take off",
        (true, false) => "on",
        (false, _) => "off",
    }
}

/// Returns the word for the frame the ship is measured in.
fn frame_name(frame: Frame) -> &'static str {
    match frame {
        Frame::Surface(_) => "surface",
        Frame::System(_) => "orbit",
        Frame::Galaxy(_) => "deep space",
    }
}

/// Returns the text of the warp instrument, or nothing while the drive is off.
fn warp_text(warp: WarpState) -> Option<String> {
    match warp {
        WarpState::Off => None,
        WarpState::Spooling(done) => Some(format!("spooling {}", percent(done))),
        WarpState::Active(_) => Some("engaged".to_string()),
    }
}
