//! The displays of the dashboard: which picture each one shows for what the instruments read,
//! where it stands, and the component that remembers the picture it was painted with.

use bevy::prelude::*;

use super::attitude::{project, steps};
use super::faces::{FLIGHT, REACH, SIDE, drive, flight, status};
use super::paint::{BAR_STEPS, key, words};
use super::pixels::Pixels;
use super::readings::{Dial, LAMPS};
use super::state::Cockpit;
use super::tune::{ANGLE_STEP, BAYS, RAKE, RAKE_TILT};

/// A display of the dashboard, in the order of [`BAYS`].
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    /// The drive, on the left: the throttle, the warp drive and the gear.
    Drive,
    /// The flight, in the middle: the speed, the height, the attitude and the course.
    Flight,
    /// The status, on the right: the lamps.
    Status,
}

/// One picture of a display, the values it shows in steps. A display is repainted only when
/// its face changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Face {
    /// Nothing: the display has nothing true to show.
    Dark,
    /// The steps of the bars of the throttle, of the warp drive and of the gear.
    Drive([Option<u8>; 3]),
    /// The speed and the height in the steps of [`key`], the steps of their gauges, the pitch
    /// up and the roll to the right in steps, and the marks of the course and of the body in
    /// pixels from the centre of the ball.
    Flight(
        [Option<i32>; 2],
        [Option<u8>; 2],
        Option<IVec2>,
        [Option<IVec2>; 2],
    ),
    /// Which lamps are lit.
    Status([bool; LAMPS]),
}

/// The picture a display shows now and the image it is painted on.
#[derive(Component, Debug, Clone)]
pub struct Display {
    /// The image the material of the display draws.
    pub image: Handle<Image>,
    /// The face painted last.
    pub shown: Option<Face>,
}

/// Returns the step of a bar that reads `share` from 0 to 1.
fn step(share: Option<f32>) -> Option<u8> {
    share.map(|share| (share.clamp(0.0, 1.0) * f32::from(BAR_STEPS)).round() as u8)
}

/// Returns the face of the flight display for what `cockpit` reads and its bars show.
fn flown(cockpit: &Cockpit) -> Face {
    let (read, bars) = (&cockpit.read, &cockpit.gauges);
    let figures = [read.speed.map(key), read.height.map(key)];
    let gauges = [Dial::Speed, Dial::Altitude].map(|dial| step(bars[dial as usize]));
    let level = read.attitude.map(|a| {
        let step = |angle: f32| steps(angle, ANGLE_STEP);
        IVec2::new(step(a.x), step(a.y))
    });
    let marks = [read.course, read.body].map(|to| to.map(|to| project(to, REACH)));
    let nothing =
        figures == [None; 2] && gauges == [None; 2] && level.is_none() && marks == [None; 2];
    match nothing {
        true => Face::Dark,
        false => Face::Flight(figures, gauges, level, marks),
    }
}

impl Screen {
    /// Every display.
    pub const ALL: [Screen; 3] = [Screen::Drive, Screen::Flight, Screen::Status];

    /// Returns the size in pixels of the picture of the display.
    pub fn size(self) -> (usize, usize) {
        match self {
            Screen::Flight => FLIGHT,
            Screen::Drive | Screen::Status => SIDE,
        }
    }

    /// Returns the face the display shows for what the cockpit reads and its bars show. A
    /// source that is `None` gives a dark face.
    pub fn face(self, cockpit: &Cockpit) -> Face {
        let bar = |dial: Dial| step(cockpit.gauges[dial as usize]);
        match self {
            Screen::Drive => match [Dial::Throttle, Dial::Charge, Dial::Gear].map(bar) {
                [None, None, None] => Face::Dark,
                bars => Face::Drive(bars),
            },
            Screen::Flight => flown(cockpit),
            Screen::Status => Face::Status(cockpit.read.lamps),
        }
    }

    /// Returns the picture of `face` on this display.
    pub fn painted(self, face: Face) -> Pixels {
        match (self, face) {
            (Screen::Drive, Face::Drive(bars)) => drive(bars),
            (Screen::Flight, Face::Flight([speed, height], gauges, level, marks)) => {
                let speed = speed.map(|at| words(at, true));
                let height = height.map(|at| words(at, false));
                let figures = [
                    (speed.as_deref(), gauges[0]),
                    (height.as_deref(), gauges[1]),
                ];
                flight(figures, level, marks)
            }
            (Screen::Status, Face::Status(lamps)) => status(lamps),
            _ => Pixels::dark(self.size().0, self.size().1),
        }
    }

    /// Returns where the display stands on the raked panel: in its bay, a hair off the face.
    pub fn placed(self) -> Transform {
        let (across, width) = BAYS[self as usize];
        on_panel(Vec2::new(across, 0.0), 0.002).with_scale(Vec3::new(width, BAYS[1].1, 1.0))
    }
}

/// Returns where the point `at` of the raked panel is in the hull, `lift` metres off its face,
/// turned to face the pilot.
pub fn on_panel(at: Vec2, lift: f32) -> Transform {
    let tilt = Quat::from_rotation_x(-RAKE_TILT);
    Transform {
        translation: RAKE.0 + tilt * at.extend(lift),
        rotation: tilt,
        scale: Vec3::ONE,
    }
}
