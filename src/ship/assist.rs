//! Flight assist: turns stick input into rates, kills rotation and holds a hover near the ground.

use bevy::math::DVec3;

/// Angular acceleration of the attitude thrusters around pitch, yaw and roll, in radians per second squared.
pub const TURN_ACCEL: DVec3 = DVec3::new(2.4, 1.6, 3.6);

/// Turn rates the assist holds at full stick around pitch, yaw and roll, in radians per second.
pub const TURN_RATE: DVec3 = DVec3::new(1.1, 0.7, 1.8);

/// How much harder than the pilot the assist may fire the attitude thrusters.
const ASSIST_GAIN: f64 = 2.5;

/// Acceleration of the vertical thrusters in metres per second squared.
pub const LIFT_ACCEL: f64 = 55.0;

/// Height above the ground in metres under which the assist holds a hover.
pub const HOVER_CEILING: f64 = 250.0;

/// Climb speed in metres per second the hover gives at full upward input.
const CLIMB_SPEED: f64 = 12.0;

/// Sink speed in metres per second the hover gives at full downward input, under the safe landing speed.
const SINK_SPEED: f64 = 4.0;

/// Acceleration the hover commands per metre per second of climb speed error.
const HOVER_GAIN: f64 = 3.0;

/// Throttle under which the lever counts as neutral.
pub const NEUTRAL: f64 = 0.05;

/// Smallest cosine between the roof and the local up at which the hover still works.
const HOVER_TILT: f64 = 0.3;

/// Returns the spin after `h` seconds of attitude thrusters, for a `stick` of pitch, yaw and roll from -1 to 1.
///
/// Without assist the stick is a torque. With assist it is a turn rate, so a released stick stops the rotation.
pub fn steer(spin: DVec3, stick: DVec3, assist: bool, h: f64) -> DVec3 {
    if !assist {
        return spin + stick * TURN_ACCEL * h;
    }
    let reach = TURN_ACCEL * (ASSIST_GAIN * h);
    spin + (stick * TURN_RATE - spin).clamp(-reach, reach)
}

/// Rate per second at which the assist turns the drift of a thrusting ship onto its nose in
/// space, and the hardest it pushes for that, in metres per second squared.
const GRIP: (f64, f64) = (2.5, 160.0);

/// Deceleration of the brake in space, in metres per second squared.
const BRAKE_ACCEL: f64 = 90.0;

/// What the assist holds in space, where no air turns the flight path: the ship as it drifts
/// past the body it is nearest to.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hold {
    /// Velocity relative to the nearest body, in the axes of the frame.
    pub drift: DVec3,
    /// Unit direction of the nose, in the axes of the frame.
    pub nose: DVec3,
    /// Whether the main engine pushes.
    pub thrusting: bool,
    /// Whether the brake is held.
    pub braking: bool,
}

impl Hold {
    /// Returns the acceleration the assist adds over a step of `h` seconds.
    ///
    /// Under thrust it sheds the part of the drift that is not along the nose, so the ship goes
    /// where it points instead of sliding on the way it went before. With the brake it sheds
    /// all of the drift. With the engine idle it does nothing, so an orbit stays an orbit.
    pub fn push(&self, h: f64) -> DVec3 {
        if self.braking {
            let most = self.drift.length() / h.max(1e-9);
            return -self.drift.normalize_or_zero() * BRAKE_ACCEL.min(most);
        }
        if !self.thrusting {
            return DVec3::ZERO;
        }
        let slide = self.drift - self.nose * self.drift.dot(self.nose).max(0.0);
        (-slide * GRIP.0).clamp_length_max(GRIP.1)
    }
}

/// What the hover needs to know about the ship.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hover {
    /// Vertical thruster input from -1 to 1.
    pub stick: f64,
    /// Whether the assist is on and the throttle is neutral.
    pub armed: bool,
    /// Height of the gear above the ground in metres.
    pub radar: f64,
    /// Speed away from the planet centre in metres per second.
    pub climb: f64,
    /// Downward acceleration to cancel, in metres per second squared.
    pub weight: f64,
    /// Cosine of the angle between the roof and the local up.
    pub tilt: f64,
    /// Whether the ship touches the ground.
    pub touching: bool,
}

/// Returns the acceleration of the vertical thrusters along the roof, in metres per second squared.
///
/// Near the ground an armed assist holds the climb speed the stick asks for, zero when it is released.
/// On the ground it lets the ship settle unless the stick asks to climb. Otherwise the stick is the thrust.
pub fn lift(hover: &Hover) -> f64 {
    let held = hover.armed && hover.radar < HOVER_CEILING && hover.tilt > HOVER_TILT;
    if !held {
        return hover.stick * LIFT_ACCEL;
    }
    if hover.touching && hover.stick <= 0.0 {
        return 0.0;
    }
    let target = hover.stick
        * if hover.stick > 0.0 {
            CLIMB_SPEED
        } else {
            SINK_SPEED
        };
    let wanted = hover.weight + (target - hover.climb) * HOVER_GAIN;
    (wanted / hover.tilt).clamp(0.0, LIFT_ACCEL)
}
