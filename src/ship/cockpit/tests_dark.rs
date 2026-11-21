//! Instrument tests: each one reads the telemetry value when its source is there and goes dark
//! when its source is `None`, and the horizon and the scope are drawn from real directions.

use bevy::prelude::*;

use super::Cockpit;
use super::paint::{key, value, words};
use super::readings::{Dial, Lamp, Readings};
use super::screens::{Face, Screen};
use crate::ship::telemetry::{ShipTelemetry, WarpState};
use crate::ship::warp::GALAXY_CAP;
use crate::space::Frame;
use crate::universe::BodyId;

/// Returns a cockpit that reads `read` and whose bars have not moved.
fn seen(read: Readings) -> Cockpit {
    Cockpit { read, ..default() }
}

/// Returns telemetry on a planet: `speed` over the ground, `height` above it and a drive.
fn aloft(speed: f64, height: f64, warp: WarpState) -> ShipTelemetry {
    ShipTelemetry {
        surface_speed: speed,
        altitude_ground: height,
        warp,
        landed: false,
        ..default()
    }
}

/// Returns telemetry in space, between the bodies.
fn in_space() -> ShipTelemetry {
    ShipTelemetry {
        frame: Frame::System(BodyId::default().star),
        altitude_ground: 5.0e6,
        surface_speed: 800.0,
        landed: false,
        ..default()
    }
}

/// On a planet every bar reads its number; in space the altitude and the gear, which mean
/// nothing there, are dark, and so is the height display.
#[test]
fn an_instrument_without_a_source_is_dark() {
    let near = Readings::of(&aloft(100.0, 800.0, WarpState::Off), 1.0);
    assert!(near.bars.iter().all(Option::is_some));
    assert_eq!(near.height, Some(800.0));
    let far = Readings::of(&in_space(), 1.0);
    assert_eq!(far.bars[Dial::Altitude as usize], None);
    assert_eq!(far.bars[Dial::Gear as usize], None);
    assert_eq!(far.height, None);
    assert!(far.bars[Dial::Speed as usize].is_some() && far.speed == Some(800.0));
    let Face::Flight([speed, height], _, level, marks) = Screen::Flight.face(&seen(far)) else {
        panic!("the flight display is dark");
    };
    assert!(speed.is_some() && height.is_none() && level.is_none() && marks == [None; 2]);
    assert_eq!(Screen::Flight.face(&seen(Readings::default())), Face::Dark);
    assert_eq!(Screen::Drive.face(&seen(Readings::default())), Face::Dark);
    let nothing = Readings::of(&aloft(f64::NAN, f64::NAN, WarpState::Off), f64::NAN);
    assert_eq!(nothing.height, None);
    assert_eq!(nothing.bars[Dial::Altitude as usize], None);
    assert_eq!(nothing.bars[Dial::Speed as usize], None);
    assert_eq!(nothing.bars[Dial::Gear as usize], None);
}

/// A bar reads the number the HUD reads: the speed against the top speed of the place, the
/// spool of the drive, the throttle, the gear. In warp the throttle is dark and the speed is
/// the warp speed.
#[test]
fn a_bar_reads_the_telemetry() {
    let top = crate::ship::top_speed(800.0);
    let read = |t: &ShipTelemetry| Readings::of(t, 0.6);
    let flying = read(&ShipTelemetry {
        throttle: 0.4,
        ..aloft(0.5 * top, 800.0, WarpState::Spooling(0.25))
    });
    let bar = |read: &Readings, dial: Dial| read.bars[dial as usize].expect("a reading");
    assert!((bar(&flying, Dial::Speed) - 0.5).abs() < 1.0e-6);
    assert!((bar(&flying, Dial::Throttle) - 0.4).abs() < 1.0e-6);
    assert_eq!(bar(&flying, Dial::Charge), 0.25);
    assert!((bar(&flying, Dial::Gear) - 0.6).abs() < 1.0e-6);
    assert_eq!(flying.speed, Some(0.5 * top));
    let warp = read(&aloft(0.0, 800.0, WarpState::Active(GALAXY_CAP)));
    assert_eq!(bar(&warp, Dial::Speed), 1.0);
    assert_eq!(bar(&warp, Dial::Charge), 1.0);
    assert_eq!(warp.bars[Dial::Throttle as usize], None);
    assert_eq!(warp.speed, Some(GALAXY_CAP));
    let low = bar(&read(&aloft(0.0, 10.0, WarpState::Off)), Dial::Altitude);
    let high = bar(&read(&aloft(0.0, 10_000.0, WarpState::Off)), Dial::Altitude);
    assert!(low > 0.0 && high > low && high < 1.0);
}

/// The lamps of the telemetry light for the assist and for the ground, and nothing else.
#[test]
fn the_lamps_follow_the_state() {
    let landed = Readings::of(&ShipTelemetry::default(), 1.0);
    assert!(landed.lamps[Lamp::Landed as usize] && landed.lamps[Lamp::Assist as usize]);
    assert!(!landed.lamps[Lamp::Close as usize] && !landed.lamps[Lamp::Hot as usize]);
    let flying = Readings::of(
        &ShipTelemetry {
            assist: false,
            ..aloft(10.0, 100.0, WarpState::Off)
        },
        1.0,
    );
    assert!(flying.lamps.iter().all(|lit| !lit));
}

/// The flight display reads the value of its source in steps of half a percent, a figure
/// without a source is left out, and the display is dark with none.
#[test]
fn a_text_display_shows_its_value_in_steps() {
    let read = Readings {
        speed: Some(1500.0),
        height: None,
        ..default()
    };
    let Face::Flight([Some(step), None], ..) = Screen::Flight.face(&seen(read)) else {
        panic!("the flight display does not show the speed alone");
    };
    assert!((value(step) / 1500.0 - 1.0).abs() < 0.003);
    assert_eq!(key(1000.0), key(1000.4));
    assert_ne!(key(1000.0), key(1010.0));
    assert_eq!(key(-5.0), 0);
    assert_eq!(words(0, true), "0 m/s");
    assert_eq!(Screen::Flight.painted(Face::Dark).count(), 0);
    let shown = Face::Flight([Some(step), None], [None; 2], None, [None; 2]);
    assert!(Screen::Flight.painted(shown).count() > 10);
}

/// The status display shows the lamps that are read, each in its place, and the drive display
/// the bars of the throttle, of the warp drive and of the gear.
#[test]
fn the_side_displays_show_the_lamps_and_the_bars() {
    let mut cockpit = seen(Readings::of(
        &aloft(100.0, 800.0, WarpState::Spooling(0.5)),
        1.0,
    ));
    cockpit.read.lamps[Lamp::Hot as usize] = true;
    cockpit.gauges = cockpit.read.bars;
    let Face::Status(lamps) = Screen::Status.face(&cockpit) else {
        panic!("the status display is dark");
    };
    assert!(lamps[Lamp::Hot as usize] && !lamps[Lamp::Failing as usize]);
    let Face::Drive([throttle, charge, gear]) = Screen::Drive.face(&cockpit) else {
        panic!("the drive display is dark");
    };
    assert_eq!((throttle, charge, gear), (Some(0), Some(16), Some(32)));
}
