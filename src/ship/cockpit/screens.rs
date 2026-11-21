//! The displays of the dashboard: which picture each one shows for what the instruments read,
//! where it stands, and the component that remembers the picture it was painted with.

use bevy::prelude::*;

use super::attitude::{project, steps};
use super::paint::{BALL, REACH, TEXT, horizon, key, scope, text, words};
use super::pixels::Pixels;
use super::readings::Readings;
use super::tune::{ANGLE_STEP, PANEL_Z, RAKE, RAKE_TILT};

/// A display of the dashboard.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    /// The speed, on the left pod.
    Speed,
    /// The height above the ground, on the right pod. Dark away from a surface.
    Height,
    /// The attitude ball, on the raked panel. Dark where there is no horizon.
    Horizon,
    /// The scope of the course and of the body beside, on the raked panel.
    Scope,
}

/// One picture of a display, the value it shows in steps. A display is repainted only when
/// its face changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Face {
    /// Nothing: the display has nothing true to show.
    Dark,
    /// A speed or a height in the steps of [`key`].
    Text(i32),
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

    /// Returns the face the display shows for what the instruments read. A source that is
    /// `None` gives a dark face.
    pub fn face(self, read: &Readings) -> Face {
        let mark = |direction: Vec3| project(direction, REACH);
        match self {
            Screen::Speed => read.speed.map_or(Face::Dark, |v| Face::Text(key(v))),
            Screen::Height => read.height.map_or(Face::Dark, |v| Face::Text(key(v))),
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
        match (self, face) {
            (Screen::Speed, Face::Text(step)) => text(&words(step, true)),
            (Screen::Height, Face::Text(step)) => text(&words(step, false)),
            (Screen::Horizon, Face::Horizon(at)) => horizon(at.x, at.y),
            (Screen::Scope, Face::Scope(course, body)) => scope(course, body),
            _ => Pixels::dark(self.size().0, self.size().1),
        }
    }

    /// Returns where the display stands in the hull: the text displays on the faces of the
    /// pods, the ball and the scope side by side on the raked panel.
    pub fn placed(self) -> Transform {
        let z = PANEL_Z + 0.006;
        let on_pod =
            |x: f32| Transform::from_xyz(x, 0.565, z).with_scale(Vec3::new(0.155, 0.0271, 1.0));
        let tilt = Quat::from_rotation_x(-RAKE_TILT);
        let on_panel = |across: f32| Transform {
            translation: RAKE.0 + tilt * Vec3::new(across, 0.012, 0.002),
            rotation: tilt,
            scale: Vec3::new(0.088, 0.088, 1.0),
        };
        match self {
            Screen::Speed => on_pod(-0.2075),
            Screen::Height => on_pod(0.2075),
            Screen::Horizon => on_panel(-0.05),
            Screen::Scope => on_panel(0.05),
        }
    }
}
