//! Tests of a ship in orbital cruise as the other players draw it: at 226 km/s around an
//! Earth-size planet that itself runs round its star at 30 km/s, the jitter buffer draws it on
//! its path, not on the chords between its records.

use glam::{DVec3, Quat};

use super::buffer_tests::{INTERVAL, arrivals, play};
use super::{Buffer, Snap};

/// Metres from the centre of the planet to the ship: an Earth radius and 111 km.
const RADIUS: f64 = 6.482e6;

/// Speed of the cruise along its track in metres per second.
const SPEED: f64 = 226_000.0;

/// Where the planet is at time 0 in the frame of its star, in metres.
const START: DVec3 = DVec3::new(1.5e11, 0.0, 0.0);

/// Velocity of the planet round its star in metres per second.
const ORBIT: DVec3 = DVec3::new(0.0, 0.0, -29_800.0);

/// Returns the true state of the cruising ship at `time`, in the frame of the star.
fn skim(time: f64) -> Snap<()> {
    let angle = time * SPEED / RADIUS;
    let (sin, cos) = angle.sin_cos();
    let round = DVec3::new(cos, 0.0, -sin) * RADIUS;
    let along = DVec3::new(-sin, 0.0, -cos) * SPEED;
    Snap {
        time,
        position: START + ORBIT * time + round,
        velocity: Some((ORBIT + along).as_vec3()),
        rotation: Quat::from_rotation_y(angle as f32),
        extra: (),
    }
}

/// At 30 records a second with 5 percent loss and up to 20 ms of jitter, every state drawn is
/// within 10 m of the true path of the cruise, a hull length while the ship covers 3.8 km a
/// frame, and the ship moves on every frame.
#[test]
fn a_cruising_ship_is_drawn_on_its_path() {
    let mut buffer = Buffer::new(INTERVAL);
    let list = arrivals(skim, INTERVAL, 0.02, 0.05, 30.0);
    let (mut worst, mut frozen, mut last) = (0.0_f64, 0, None);
    play(&mut buffer, &list, 30.0, |view, now, _| {
        if now > 1.0 {
            worst = worst.max(view.position.distance(skim(view.time).position));
            let moved = last.map_or(1.0, |at: DVec3| at.distance(view.position));
            frozen += usize::from(moved < 1.0);
        }
        last = Some(view.position);
    });
    assert!(worst < 10.0, "worst error {worst} m");
    assert_eq!(frozen, 0);
}
