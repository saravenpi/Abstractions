//! Prints the measured numbers of the flight model. Run with `--nocapture`.

use bevy::prelude::*;

use super::{Flight, flying, gliding};
use crate::gear::{GearInput, Mode};

/// Returns the glide ratio and speed a glide settles at when looking `elevation` radians up.
pub(super) fn settled_glide(elevation: f32) -> (f32, f32) {
    let mut flight = Flight::new(Mode::Glide, Vec3::new(0.0, -12.0, -36.0));
    flight.run(&gliding(elevation), 40.0);
    let v = flight.state.velocity;
    (v.with_y(0.0).length() / -v.y, v.length())
}

/// Returns the level speed and climb rate a powered flight settles at when looking `elevation` up.
pub(super) fn settled_flight(elevation: f32) -> (f32, f32) {
    let mut flight = Flight::new(Mode::Fly, Vec3::new(0.0, 0.0, -40.0));
    flight.run(&flying(elevation), 60.0);
    (
        flight.state.velocity.with_y(0.0).length(),
        flight.state.velocity.y,
    )
}

/// Prints glide ratio, speeds, climb rate and boot descent speed.
#[test]
fn print_measured_numbers() {
    for degrees in [10, 0, -10, -15, -20, -25, -30, -45, -60, -80] {
        let (ratio, speed) = settled_glide((degrees as f32).to_radians());
        println!("glide look {degrees:4} deg: ratio {ratio:5.2} speed {speed:6.1}");
    }
    for degrees in [-20, -10, 0, 5, 10, 15, 20, 30, 45, 60] {
        let (speed, climb) = settled_flight((degrees as f32).to_radians());
        println!("fly look {degrees:4} deg: ground speed {speed:6.1} climb {climb:6.1}");
    }
    let mut fall = Flight::new(Mode::BootFall, Vec3::new(0.0, -40.0, 0.0));
    let boots = GearInput {
        boost: true,
        ..GearInput::default()
    };
    fall.run(&boots, 10.0);
    println!("boot descent {:5.2}", -fall.state.velocity.y);
    let mut drop = Flight::new(Mode::Fall, Vec3::ZERO);
    drop.run(&GearInput::default(), 40.0);
    println!("folded terminal {:5.1}", -drop.state.velocity.y);
}

/// Prints how a take-off from a jump goes: lowest height, and height and speed over time.
#[test]
fn print_take_off() {
    for start in [Vec3::new(0.0, 7.0, 0.0), Vec3::new(0.0, 7.0, -8.5)] {
        let mut flight = Flight::new(Mode::Off, start);
        let mut low = 0.0_f64;
        for frame in 0..600 {
            flight.tick(&flying(0.0), super::DT);
            low = low.min(flight.pos.y);
            if frame % 30 == 29 {
                let s = flight.state;
                println!(
                    "t {:4.1} y {:6.1} speed {:5.1} pitch {:5.2} prone {:4.2} load {:4.2} {:?}",
                    (frame + 1) as f32 * super::DT,
                    flight.pos.y,
                    s.airspeed,
                    s.pitch,
                    s.prone,
                    s.load,
                    s.mode
                );
            }
        }
        println!("lowest {low:5.2}");
    }
}

/// Prints a pull-up out of a dive.
#[test]
fn print_pull_up() {
    let mut flight = Flight::new(Mode::Glide, Vec3::new(0.0, -12.0, -36.0));
    flight.run(&gliding(-0.8), 20.0);
    for frame in 0..600 {
        flight.tick(&gliding(0.7), super::DT);
        if frame % 20 == 0 {
            let s = flight.state;
            println!(
                "t {:4.1} y {:7.1} v {:5.1} vy {:6.1} pitch {:5.2} aoa {:5.2} load {:4.2}",
                frame as f32 * super::DT,
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

/// Prints a level powered turn to the left through a quarter circle and beyond.
#[test]
fn print_turn() {
    let mut flight = Flight::new(Mode::Fly, Vec3::new(0.0, 0.0, -70.0));
    flight.run(&flying(0.0), 3.0);
    let input = GearInput {
        look: super::looking(1.5, 0.0),
        ..flying(0.0)
    };
    for frame in 0..480 {
        flight.tick(&input, super::DT);
        if frame % 30 == 29 {
            let s = flight.state;
            let heading = (-s.velocity.x).atan2(-s.velocity.z);
            println!(
                "t {:4.1} heading {:5.2} bank {:5.2} y {:6.1} v {:5.1} load {:4.2} aoa {:4.2}",
                (frame + 1) as f32 * super::DT,
                heading,
                s.bank,
                flight.pos.y,
                s.airspeed,
                s.load,
                s.aoa
            );
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
            ..flying(0.05)
        };
        flight.run(&input, 60.0);
        println!(
            "ceiling at {percent:2} percent of the atmosphere: density {:4.2} height kept {:7.1} speed {:5.1}",
            input.density, flight.pos.y, flight.state.airspeed
        );
    }
}
