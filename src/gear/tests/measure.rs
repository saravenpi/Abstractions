//! Prints the measured numbers of the flight model. Run with `--nocapture`.

use bevy::prelude::*;

use super::glide::sticks;
use super::{DT, Flight, flying, gliding};
use crate::gear::{GearInput, Mode};

/// Returns the glide ratio, the airspeed and the sink rate a hands-off glide settles at in
/// the air `input` describes.
pub(super) fn settled_glide(input: &GearInput) -> (f32, f32, f32) {
    let mut flight = Flight::new(Mode::Glide, Vec3::new(0.0, -12.0, -36.0));
    flight.run(input, 40.0);
    let v = flight.state.velocity;
    (v.with_y(0.0).length() / -v.y, v.length(), -v.y)
}

/// Returns the level speed and climb rate a hands-off powered flight settles at on a course
/// set `course` radians above the horizon.
pub(super) fn settled_flight(course: f32) -> (f32, f32) {
    let mut flight = Flight::new(Mode::Fly, Vec3::new(0.0, 0.0, -40.0));
    flight.state.servo.course = course;
    flight.run(&flying(0.0), 60.0);
    (
        flight.state.velocity.with_y(0.0).length(),
        flight.state.velocity.y,
    )
}

/// Returns a settled hands-off glide pushed into a dive: the forward key held for `push`
/// seconds and let go, then flown until the speed tops out. Gives the flight at the top speed.
pub(super) fn dive(push: f32) -> Flight {
    let mut flight = Flight::new(Mode::Glide, Vec3::new(0.0, -12.0, -36.0));
    flight.run(&gliding(0.0), 30.0);
    flight.run(&sticks(-1.0, 0.0), push);
    let mut fastest = flight;
    for _ in 0..600 {
        flight.tick(&gliding(0.0), DT);
        if flight.state.airspeed > fastest.state.airspeed {
            fastest = flight;
        }
    }
    fastest
}

/// Prints the hands-off glide, the dives, the powered courses and the boots.
#[test]
fn print_measured_numbers() {
    let (ratio, speed, sink) = settled_glide(&gliding(0.0));
    println!("hands-off glide: ratio {ratio:5.2} speed {speed:5.1} sink {sink:5.2}");
    for push in [0.3, 0.6, 0.9, 1.2] {
        let top = dive(push).state;
        println!(
            "dive pushed {push:3.1} s: top {:5.1} at pitch {:5.2}",
            top.airspeed, top.pitch
        );
    }
    for course in [-0.2, 0.0, 0.2, 0.4, 0.5] {
        let (speed, climb) = settled_flight(course);
        println!("fly course {course:4.1}: ground speed {speed:6.1} climb {climb:6.1}");
    }
    let mut fall = Flight::new(Mode::BootFall, Vec3::new(0.0, -40.0, 0.0));
    fall.run(&super::braking(), 10.0);
    println!("boot descent {:5.2}", -fall.state.velocity.y);
    let mut drop = Flight::new(Mode::Fall, Vec3::ZERO);
    drop.run(&GearInput::default(), 40.0);
    println!("folded terminal {:5.1}", -drop.state.velocity.y);
}

/// Prints a pull-up out of a dive: the back key held for 2.2 s and let go.
#[test]
fn print_pull_up() {
    let mut flight = dive(1.2);
    for frame in 0..600 {
        let stick = f32::from(u8::from(frame < 132));
        flight.tick(&sticks(stick, 0.0), DT);
        if frame % 20 == 0 {
            let s = flight.state;
            println!(
                "t {:4.1} y {:7.1} v {:5.1} vy {:6.1} pitch {:5.2} aoa {:5.2} load {:4.2}",
                frame as f32 * DT,
                flight.pos.y,
                s.airspeed,
                s.velocity.y,
                s.pitch,
                s.aoa,
                s.load
            );
        }
    }
}

/// Prints a gliding turn and a powered turn to the left with the roll key held.
#[test]
fn print_turn() {
    for powered in [false, true] {
        let mut flight = Flight::new(Mode::Glide, Vec3::new(0.0, -12.0, -36.0));
        let input = GearInput {
            boost: powered,
            ..sticks(0.0, -1.0)
        };
        flight.run(&GearInput { roll: 0.0, ..input }, 30.0);
        for frame in 0..240 {
            flight.tick(&input, DT);
            if frame % 30 == 29 {
                let s = flight.state;
                let heading = (-s.velocity.x).atan2(-s.velocity.z);
                println!(
                    "t {:4.1} heading {:5.2} bank {:5.2} vy {:6.1} v {:5.1} load {:4.2}",
                    (frame + 1) as f32 * DT,
                    heading,
                    s.bank,
                    s.velocity.y,
                    s.airspeed,
                    s.load
                );
            }
        }
    }
}

/// Prints the highest density ratio step at which level flight still holds, and what that is
/// as a share of the atmosphere height.
#[test]
fn print_ceiling() {
    for percent in (0..=60).step_by(2) {
        let share = percent as f32 / 100.0;
        let mut flight = Flight::new(Mode::Fly, Vec3::new(0.0, 0.0, -70.0));
        let input = GearInput {
            density: crate::gear::air_density(1.0, share),
            ..flying(0.0)
        };
        flight.run(&input, 60.0);
        println!(
            "ceiling at {percent:2} percent of the atmosphere: density {:4.2} height kept {:7.1} speed {:5.1}",
            input.density, flight.pos.y, flight.state.airspeed
        );
    }
}

/// Prints how a take-off from a jump goes: lowest height, and height and speed over time.
#[test]
fn print_take_off() {
    for start in [Vec3::new(0.0, 7.0, 0.0), Vec3::new(3.0, 7.0, -8.5)] {
        let mut flight = Flight::new(Mode::Off, start);
        let mut low = 0.0_f64;
        for frame in 0..600 {
            flight.tick(&flying(0.0), DT);
            low = low.min(flight.pos.y);
            if frame % 15 == 14 && frame < 200 {
                let s = flight.state;
                println!(
                    "t {:4.2} y {:6.2} v {:5.1} vy {:5.1} pitch {:5.2} course {:5.2} {:?}",
                    (frame + 1) as f32 * DT,
                    flight.pos.y,
                    s.airspeed,
                    s.velocity.y,
                    s.pitch,
                    s.servo.course,
                    s.mode
                );
            }
        }
        println!("lowest {low:5.2}");
    }
}
