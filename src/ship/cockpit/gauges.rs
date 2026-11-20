//! Where the bars and the lamps stand on the dashboard, and how a reading stretches a bar or
//! lights a lamp. A source that is `None` leaves nothing lit: the part shrinks to nothing.

use bevy::prelude::*;

use super::readings::{Dial, Lamp};
use super::tune::{PANEL_Z, RAKE, RAKE_TILT};

/// Length of a bar at full reading and its height, in metres.
const BAR: (f32, f32) = (0.155, 0.012);

/// Side of a lamp, and the distance between lamps, in metres.
const LAMP: (f32, f32) = (0.016, 0.03);

/// Scale of a part that shows nothing: too small to see, and still a valid transform.
const NOTHING: f32 = 1.0e-4;

impl Dial {
    /// Returns where the left end of the bar stands on the face of its pod: x and height.
    fn origin(self) -> Vec2 {
        match self {
            Dial::Speed => Vec2::new(-0.285, 0.515),
            Dial::Throttle => Vec2::new(-0.285, 0.497),
            Dial::Altitude => Vec2::new(0.13, 0.515),
            Dial::Charge => Vec2::new(0.13, 0.497),
            Dial::Gear => Vec2::new(0.13, 0.479),
        }
    }

    /// Returns the lit strip of the bar when it reads `value`: a unit quad that starts at its
    /// left end, stretched to the reading. A bar with no reading is dark.
    pub fn place(self, value: Option<f32>) -> Transform {
        let at = self.origin().extend(PANEL_Z + 0.006);
        let Some(value) = value else {
            return Transform::from_translation(at).with_scale(Vec3::new(NOTHING, NOTHING, 1.0));
        };
        let lit = value.clamp(0.0, 1.0).max(NOTHING);
        Transform::from_translation(at).with_scale(Vec3::new(BAR.0 * lit, BAR.1, 1.0))
    }
}

impl Lamp {
    /// Returns the colour of the lamp as a linear share of white: the lamps of the state of
    /// the ship are white, a warning is amber, a danger is red.
    pub fn tone(self) -> Vec3 {
        match self {
            Lamp::Assist | Lamp::Landed => Vec3::ONE,
            Lamp::Close | Lamp::Hot | Lamp::Frame => Vec3::new(1.0, 0.45, 0.04),
            Lamp::Failing => Vec3::new(1.0, 0.06, 0.04),
        }
    }

    /// Returns where the lamp stands on the raked panel, lit or dark.
    pub fn place(self, lit: bool) -> Transform {
        let tilt = Quat::from_rotation_x(-RAKE_TILT);
        let across = LAMP.1 * (self as usize as f32 - 2.5);
        let size = if lit { LAMP.0 } else { NOTHING };
        Transform {
            translation: RAKE.0 + tilt * Vec3::new(across, -0.055, 0.002),
            rotation: tilt,
            scale: Vec3::new(size, size, 1.0),
        }
    }
}
