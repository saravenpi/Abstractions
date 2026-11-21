//! The flight instruments: what each one reads from the ship, as text.

use bevy::prelude::*;

use super::context::Element;
use super::units::{height, percent, speed};
use crate::ship::{ShipTelemetry, WarpState};
use crate::space::Frame;
use crate::ui::Icon;

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
    /// Highest point of the orbit.
    Apoapsis,
    /// Lowest point of the orbit.
    Periapsis,
    /// Landing gear.
    Gear,
    /// The headlight in the nose.
    Headlight,
}

impl Gauge {
    /// Every instrument, in display order, with its label.
    pub(super) const ALL: [(Gauge, &'static str); 8] = [
        (Gauge::Altitude, "Altitude"),
        (Gauge::Speed, "Speed"),
        (Gauge::Throttle, "Throttle"),
        (Gauge::Assist, "Stabiliser"),
        (Gauge::Apoapsis, "Highest point"),
        (Gauge::Periapsis, "Lowest point"),
        (Gauge::Gear, "Gear"),
        (Gauge::Headlight, "Headlight"),
    ];

    /// Returns the HUD element this instrument belongs to.
    pub(super) const fn element(self) -> Element {
        match self {
            Gauge::Altitude => Element::Altitude,
            Gauge::Speed => Element::Speed,
            Gauge::Throttle => Element::Throttle,
            Gauge::Assist => Element::Assist,
            Gauge::Apoapsis | Gauge::Periapsis => Element::Apsides,
            Gauge::Gear => Element::Gear,
            Gauge::Headlight => Element::Headlight,
        }
    }
}

/// What the instruments are read from.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Flight {
    /// Instruments of the ship.
    pub telemetry: ShipTelemetry,
    /// Whether the ship flies over a gas giant, which has a cloud deck and no ground.
    pub giant: bool,
    /// Height above the sea and speed through the air in metres, while the player glides on
    /// foot. The ship's instruments are not read then.
    pub glide: Option<(f64, f64)>,
    /// Extension of the landing gear, from 0 (folded) to 1 (down and locked), with a ship.
    pub gear: Option<f64>,
    /// Whether the headlight is on and the name of the key that switches it, with a ship.
    pub headlight: Option<(bool, String)>,
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

/// Returns the text of the gear instrument near a planet with ground: up while folded, down
/// once locked, moving on its way. Nothing in space or over a gas giant, where no gear helps.
fn gear_text(flight: &Flight) -> Option<&'static str> {
    let near = matches!(flight.telemetry.frame, Frame::Surface(_)) && !flight.giant;
    let gear = flight.gear.filter(|_| near)?;
    Some(match gear {
        folded if folded <= 0.0 => "up",
        locked if locked >= 1.0 => "down",
        _ => "moving",
    })
}

/// Returns the text of the headlight instrument near a planet: on or off. Its row shows the key
/// that switches it after it. Nothing in space, where the lamp lights nothing.
fn headlight_text(flight: &Flight) -> Option<String> {
    let near = matches!(flight.telemetry.frame, Frame::Surface(_));
    let (on, _) = flight.headlight.as_ref().filter(|_| near)?;
    Some(["off", "on"][usize::from(*on)].to_string())
}

/// Returns the icon of an instrument. The gear shows its state by its icon, down, up or on its
/// way, and the headlight a lit lamp or a lamp struck out; the words beside the icon say the
/// state too.
pub(super) fn symbol(gauge: Gauge, flight: &Flight) -> Icon {
    match gauge {
        Gauge::Altitude => Icon::Altitude,
        Gauge::Speed => Icon::Speed,
        Gauge::Throttle => Icon::Throttle,
        Gauge::Assist => Icon::Assist,
        Gauge::Apoapsis => Icon::Apoapsis,
        Gauge::Periapsis => Icon::Periapsis,
        Gauge::Gear => match gear_text(flight) {
            Some("down") => Icon::GearDown,
            Some("moving") => Icon::Reload,
            _ => Icon::GearUp,
        },
        Gauge::Headlight => match flight.headlight {
            Some((true, _)) => Icon::HeadlightOn,
            _ => Icon::HeadlightOff,
        },
    }
}

/// Returns what an instrument shows, or nothing when it has nothing to show: no orbit outside
/// space, no gear away from a planet. Which instruments show in a place is `context::shown`.
pub fn reading(gauge: Gauge, flight: &Flight) -> Option<String> {
    let t = &flight.telemetry;
    let in_orbit = matches!(t.frame, Frame::System(_)) && !t.landed;
    match gauge {
        Gauge::Altitude => Some(match flight.glide {
            Some((altitude, _)) => height(altitude),
            None => altitude_text(flight),
        }),
        Gauge::Speed => Some(match flight.glide {
            Some((_, airspeed)) => format!("{} air", speed(airspeed)),
            None => speed_text(flight),
        }),
        Gauge::Throttle => Some(percent(t.throttle)),
        Gauge::Assist => Some(assist_text(t).to_string()),
        Gauge::Apoapsis => t.apoapsis.filter(|_| in_orbit).map(height),
        Gauge::Periapsis => t.periapsis.filter(|_| in_orbit).map(height),
        Gauge::Gear => gear_text(flight).map(str::to_string),
        Gauge::Headlight => headlight_text(flight),
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

/// Returns the text of the altitude instrument: the height above the ground, or above the
/// cloud deck of a gas giant, which is its sea level.
fn altitude_text(flight: &Flight) -> String {
    let t = &flight.telemetry;
    match flight.giant && matches!(t.frame, Frame::Surface(_)) {
        true => format!("{} above deck", height(t.altitude_sea)),
        false => height(t.altitude_ground),
    }
}
