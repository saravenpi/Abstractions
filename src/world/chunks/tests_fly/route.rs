//! The path a flyover follows: a great circle or a steady turn, at a height over the ground or
//! over the sea.

use bevy::math::{DQuat, DVec3};

use crate::world::Globe;

/// Sideways acceleration of a turning flyover in metres per second squared.
const BANK: f64 = 20.0;

/// Fastest turn of a flyover in radians per second.
const TIGHTEST: f64 = 0.5;

/// Heights under which a flyover follows the ground instead of the sea level sphere.
const HUGS: f64 = 100.0;

/// Where a flyover is and where it goes.
#[derive(Debug, Clone, Copy)]
pub struct Route {
    /// Unit direction from the planet centre to the focus.
    pub up: DVec3,
    /// Unit direction of travel, tangent to the sphere.
    pub heading: DVec3,
    /// Speed in metres per second.
    pub speed: f64,
    /// Height in metres: over the ground when low, over the sea when high.
    pub height: f64,
    /// Whether the path turns steadily instead of going straight.
    pub turns: bool,
}

impl Route {
    /// Returns the planet position of the focus on `globe`.
    pub fn at(&self, globe: &Globe) -> DVec3 {
        let ground = match self.height < HUGS {
            true => f64::from(globe.height(self.up).max(0.0)),
            false => 0.0,
        };
        self.up * (globe.radius() + ground + self.height)
    }

    /// Returns the height of the focus over the ground or the sea under it on `globe`.
    pub fn clearance(&self, globe: &Globe) -> f64 {
        let ground = f64::from(globe.height(self.up).max(0.0));
        self.at(globe).length() - globe.radius() - ground
    }

    /// Moves the focus `dt` seconds along the path over `globe`.
    pub fn step(&mut self, globe: &Globe, dt: f64) {
        if self.turns {
            let rate = (BANK / self.speed).min(TIGHTEST);
            self.heading = DQuat::from_axis_angle(self.up, rate * dt) * self.heading;
        }
        let angle = self.speed * dt / self.at(globe).length();
        let roll = DQuat::from_axis_angle(self.up.cross(self.heading).normalize(), angle);
        self.up = (roll * self.up).normalize();
        self.heading = (roll * self.heading).normalize();
    }
}

/// A flyover keeps its speed over the ground and a turning one changes its heading.
#[test]
fn route_keeps_speed_and_turns() {
    let globe = Globe::new(7, 6000.0, 9.81);
    let mut route = Route {
        up: DVec3::Y,
        heading: DVec3::X,
        speed: 300.0,
        height: 1000.0,
        turns: false,
    };
    let start = route.at(&globe);
    (0..60).for_each(|_| route.step(&globe, 1.0 / 60.0));
    let arc = start.angle_between(route.at(&globe)) * 7000.0;
    assert!((arc - 300.0).abs() < 0.01, "{arc}");
    assert!(route.heading.dot(route.up).abs() < 1.0e-9);
    route.turns = true;
    let before = route.heading;
    (0..60).for_each(|_| route.step(&globe, 1.0 / 60.0));
    assert!(before.angle_between(route.heading) > 0.05);
}
