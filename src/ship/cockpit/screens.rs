//! The displays of the dashboard: which picture each one shows for what the instruments read,
//! where it stands, and the component that remembers the picture it was painted with.

use bevy::prelude::*;

use super::attitude::{project, steps};
use super::paint::{BALL, BAR_STEPS, REACH, TEXT, block, horizon, key, scope, words};
use super::pixels::Pixels;
use super::readings::Dial;
use super::state::Cockpit;
use super::tune::{ANGLE_STEP, RAKE, RAKE_TILT, TEXEL};

/// A display of the dashboard.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    /// The block of the speed, the throttle and the drive, on the left of the panel.
    Speed,
    /// The block of the height and of the gear, on the right. Dark away from a surface.
    Height,
    /// The attitude ball, in the middle. Dark where there is no horizon.
    Horizon,
    /// The scope of the course and of the body, under the ball.
    Scope,
}

/// One picture of a display, the values it shows in steps. A display is repainted only when
/// its face changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Face {
    /// Nothing: the display has nothing true to show.
    Dark,
    /// A block: the figure in the steps of [`key`] and the steps of its gauge and of its bars.
    Block(Option<i32>, Option<u8>, [Option<u8>; 2]),
    /// The attitude: pitch up and roll to the right, in steps.
    Horizon(IVec2),
    /// The marks of the course and of the body, in pixels from the centre.
    Scope(Option<IVec2>, Option<IVec2>),
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

impl Screen {
    /// Every display.
    pub const ALL: [Screen; 4] = [
        Screen::Speed,
        Screen::Height,
        Screen::Horizon,
        Screen::Scope,
    ];

    /// Returns the size in pixels of the picture of the display.
    pub fn size(self) -> (usize, usize) {
        match self {
            Screen::Speed | Screen::Height => TEXT,
            Screen::Horizon | Screen::Scope => (BALL, BALL),
        }
    }

    /// Returns the face the display shows for what the cockpit reads and its bars show. A
    /// source that is `None` gives a dark face.
    pub fn face(self, cockpit: &Cockpit) -> Face {
        let (read, bars) = (&cockpit.read, &cockpit.gauges);
        let mark = |direction: Vec3| project(direction, REACH);
        let bar = |dial: Dial| step(bars[dial as usize]);
        match self {
            Screen::Speed => match (read.speed, bar(Dial::Speed)) {
                (None, None) => Face::Dark,
                (speed, gauge) => Face::Block(
                    speed.map(key),
                    gauge,
                    [bar(Dial::Throttle), bar(Dial::Charge)],
                ),
            },
            Screen::Height => match (read.height, bar(Dial::Altitude)) {
                (None, None) => Face::Dark,
                (height, gauge) => Face::Block(height.map(key), gauge, [bar(Dial::Gear), None]),
            },
            Screen::Horizon => read.attitude.map_or(Face::Dark, |a| {
                let step = |angle: f32| steps(angle, ANGLE_STEP);
                Face::Horizon(IVec2::new(step(a.x), step(a.y)))
            }),
            Screen::Scope => match (read.course, read.body) {
                (None, None) => Face::Dark,
                (course, body) => Face::Scope(course.map(mark), body.map(mark)),
            },
        }
    }

    /// Returns the picture of `face` on this display.
    pub fn painted(self, face: Face) -> Pixels {
        let figure = |at: Option<i32>, per_second: bool| at.map(|at| words(at, per_second));
        match (self, face) {
            (Screen::Speed, Face::Block(speed, gauge, [throttle, charge])) => {
                let speed = figure(speed, true);
                let bars = [("THR", throttle), ("WARP", charge)];
                block(
                    "SPEED",
                    speed.as_deref(),
                    gauge,
                    (&bars, ["STB", "LND", "CLS"]),
                )
            }
            (Screen::Height, Face::Block(height, gauge, [gear, _])) => {
                let height = figure(height, false);
                block(
                    "HEIGHT",
                    height.as_deref(),
                    gauge,
                    (&[("GEAR", gear)], ["HOT", "FRM", "ERR"]),
                )
            }
            (Screen::Horizon, Face::Horizon(at)) => horizon(at.x, at.y),
            (Screen::Scope, Face::Scope(course, body)) => scope(course, body),
            _ => Pixels::dark(self.size().0, self.size().1),
        }
    }

    /// Returns where the display stands on the raked panel: the blocks left and right, the
    /// ball over the scope between them.
    pub fn placed(self) -> Transform {
        let (width, height) = self.size();
        let size = Vec3::new(width as f32 * TEXEL, height as f32 * TEXEL, 1.0);
        let (u, v) = match self {
            Screen::Speed => (-BLOCK.0 - size.x * 0.5, BLOCK.1 - size.y * 0.5),
            Screen::Height => (BLOCK.0 + size.x * 0.5, BLOCK.1 - size.y * 0.5),
            Screen::Horizon => (0.0, BLOCK.1 - size.y * 0.5),
            Screen::Scope => (0.0, BLOCK.1 - size.y * 1.5 - 0.01),
        };
        on_panel(Vec2::new(u, v), 0.002).with_scale(size)
    }
}

/// Where the blocks stand: the half gap between them across, and the height of their top
/// edge on the panel.
pub const BLOCK: (f32, f32) = (0.04, 0.115);

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
