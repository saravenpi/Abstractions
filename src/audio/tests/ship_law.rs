//! The physics of the voice of a ship, and its parts taken alone: the lobe of a jet, the
//! inertia of the pumps, and the delay and the Doppler shift of the path through the air.

use super::RATE;
use crate::audio::mix::BLOCK;
use crate::audio::ship::feed::{Ch, HullSound};
use crate::audio::ship::law::{lobe, peak_hz, spread};
use crate::audio::ship::path::Path;
use crate::audio::ship::pump::Pump;
use crate::audio::ship::tune::{EARSHOT, HELMHOLTZ, LOBE_FLOOR, NEAR, PUMP_IDLE};
use crate::audio::spatial::Air;
use crate::universe::PlanetRecord;

/// The roar peaks where the Helmholtz number says, lower for a wider nozzle and in slower air,
/// and a jet is loudest at an angle behind its nozzle, weakest ahead of the nose.
#[test]
fn a_jet_follows_its_size_and_its_angle() {
    assert!((peak_hz(340.0, 0.6, 1.0) - HELMHOLTZ * 340.0 / 0.6).abs() < 0.01);
    assert!(peak_hz(340.0, 0.6, 1.0) < peak_hz(340.0, 0.25, 1.0));
    assert!(peak_hz(240.0, 0.6, 1.0) < peak_hz(340.0, 0.6, 1.0));
    assert!(peak_hz(340.0, 0.6, 0.2) > peak_hz(340.0, 0.6, 1.0));
    assert!(lobe(0.64) > lobe(1.0) && lobe(1.0) > lobe(0.0) && lobe(0.0) > lobe(-1.0));
    assert!((lobe(0.64) - 1.0).abs() < 1.0e-5 && lobe(-1.0) >= LOBE_FLOOR);
}

/// Sound spreads as the inverse of the distance and is gone at the edge of earshot, and the air
/// of the planet takes more of the high end the farther and the thinner it is: about 700 Hz is
/// left of a kilometre of the air of home.
#[test]
fn distance_and_air_follow_their_laws() {
    assert!((spread(NEAR.1) - 1.0).abs() < 1.0e-3);
    assert!((spread(200.0) / spread(400.0) - 2.0).abs() < 0.02);
    assert_eq!(spread(EARSHOT), 0.0);
    assert_eq!(spread(0.0), spread(NEAR.0));
    let home = Air::home().absorb;
    let thin = Air::sea(&PlanetRecord {
        pressure: 0.1,
        ..PlanetRecord::TEMPERATE
    })
    .absorb;
    assert!(home.corner(100.0) > home.corner(1000.0));
    assert!(home.corner(1000.0) > thin.corner(1000.0));
    assert!(
        (600.0..800.0).contains(&home.corner(1000.0)),
        "{}",
        home.corner(1000.0)
    );
}

/// Runs `pump` toward `asked` for `seconds` and returns its speed.
fn spun(pump: &mut Pump, asked: f32, seconds: f32) -> f32 {
    for _ in 0..(seconds * RATE) as usize {
        let (whine, shaft) = pump.frame(asked);
        assert!(whine.is_finite() && shaft.is_finite());
    }
    pump.speed()
}

/// The pumps have inertia: they lag the lever, come up in about a second, wind down more
/// slowly, and stop for good.
#[test]
fn the_pumps_lag_the_lever() {
    let mut pump = Pump::new(RATE, 7);
    assert!((spun(&mut pump, PUMP_IDLE, 8.0) - PUMP_IDLE).abs() < 0.01);
    let soon = spun(&mut pump, 1.0, 0.1);
    assert!(soon > PUMP_IDLE && soon < 0.55, "{soon}");
    assert!(spun(&mut pump, 1.0, 5.0) > 0.98);
    let falling = spun(&mut pump, 0.0, 0.9);
    assert!(falling > 0.55 && falling < 0.8, "{falling}");
    assert_eq!(spun(&mut pump, 0.0, 30.0), 0.0);
    assert_eq!(pump.frame(0.0), (0.0, 0.0));
}

/// Returns a ship on the ground in the air of the home planet, `distance` metres from the listener.
fn at(distance: f32) -> HullSound {
    HullSound::default()
        .with(Ch::Present, 1.0)
        .with(Ch::Air, 1.0)
        .with(Ch::Height, 0.0)
        .with(Ch::Distance, distance)
}

/// Sound takes its time: a ship 340 metres away in air where sound makes 340 metres per second
/// is heard one second late.
#[test]
fn sound_arrives_late() {
    let mut path = Path::new(RATE);
    path.clear(&at(340.0), &Air::home().absorb);
    let mut first = None;
    for i in 0..(2.0 * RATE) as usize {
        path.write(if i < 480 { 0.5 } else { 0.0 });
        if path.read()[0].abs() > 1.0e-4 && first.is_none() {
            first = Some(i as f32 / RATE);
        }
    }
    let late = first.expect("the sound arrives");
    assert!((late - 1.0).abs() < 0.02, "{late}");
}

/// A ship that closes in at 100 metres per second raises a 1000 hertz tone by `1 / (1 - v / c)`,
/// to 1417 hertz, even when its distance only moves once per frame as it does in the game.
#[test]
fn a_closing_ship_is_heard_higher() {
    let mut path = Path::new(RATE);
    let home = Air::home().absorb;
    path.clear(&at(900.0), &home);
    let (mut rises, mut before) = (0, 0.0_f32);
    for i in 0..(6.0 * RATE) as usize {
        let seconds = i as f32 / RATE;
        if i % BLOCK as usize == 0 {
            let frame = (seconds * 60.0).floor() / 60.0;
            path.tune(&at(900.0 - 100.0 * frame), &home);
        }
        path.write((std::f32::consts::TAU * 1000.0 * seconds).sin());
        let now = path.read()[0];
        if seconds >= 5.0 && before <= 0.0 && now > 0.0 {
            rises += 1;
        }
        before = now;
    }
    assert!((rises - 1417_i32).abs() < 25, "{rises} hertz");
}
