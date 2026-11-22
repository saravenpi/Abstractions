//! The speed hold of the flight assist: the lever asks for a speed along the nose, and the
//! assist flies the ship to it. Engines idle means stop.

use bevy::math::DVec3;

use super::assist::NEUTRAL;
use super::drag::{heat_cap, terminal};
use super::flight::BOOST;
use super::tune::{ORBIT_CAP, SPACE_CAP};

/// Speed in metres per second the full lever asks for at a surface, and the seconds the
/// clearance lasts at the speed it adds: the top speed is `SLOW.0 + clearance / SLOW.1`, so the
/// ship is slow where it has to be exact. Since 0.11.0 the second number is 1.6 s, down from
/// 4 s: the full lever asks for 219 m/s at 286 m, where it asked for 112, as a fast small craft
/// flies low. In air the thrust against the drag caps it, see [`terminal`].
const SLOW: (f64, f64) = (40.0, 1.6);

/// Seconds of flight the ship keeps between itself and the ground under it when it sinks,
/// and over which the climb of a hover grows with the height: the descent and the hover are
/// as slow as before the top speed was raised.
const REACH: f64 = 4.0;

/// How many times faster the full lever is with the boost.
pub(in crate::ship) const BOOST_SPEED: f64 = 2.0;

/// Share of the top speed the vertical thrusters give at full stick.
const CLIMB_SHARE: f64 = 0.3;

/// Fastest descent in metres per second the assist flies near a surface, under the speed that
/// harms the hull, and the clearance in metres under which it holds it: the nose of a diving
/// ship reaches that far under its gear. Above that it grows like the top speed.
const SINK: (f64, f64) = (1.8, 3.0);

/// Height in metres of the gear above the ground under which a hovering ship that is not sunk
/// on purpose climbs back, and the climb in metres per second for each metre it is under:
/// ground that rises under a hovering ship lifts it instead of running into it.
const FLOOR: (f64, f64) = (1.5, 4.0);

/// Steady climb and sink in metres per second of the vertical stick over a ground with the
/// lever idle, slowed only by [`descent`] near the ground.
pub const HOVER_RATES: (f64, f64) = (10.0, 6.0);

/// Height in metres of the gear above the ground up to which those rates hold: above it they
/// grow with the height, as the top speed does, so that a climb to space does not take all day.
pub const HOVER_CEILING: f64 = 150.0;

/// Seconds in which the assist closes most of a speed error: after that long, 63 percent is gone.
const SETTLE: f64 = 0.25;

/// Acceleration in metres per second squared the assist adds to that, so that the ship gets
/// to the speed asked for in a finite time instead of creeping toward it.
const FIRM: f64 = 4.0;

/// Hardest push along the nose: metres per second squared at rest, and the seconds in which
/// the engine can make or shed the top speed. The second number is under [`REACH`], so a ship
/// that sinks toward a surface always sheds its speed in time.
const THRUST: (f64, f64) = (40.0, 3.0);

/// Hardest push across the nose: metres per second squared at rest, and what each metre per
/// second of speed adds, so that the path follows the nose at any speed.
const SIDE: (f64, f64) = (60.0, 2.0);

/// How many times harder the engine pushes with the boost or the brake.
const HARD: f64 = 2.5;

/// Share of the push across the nose the assist keeps in home air or thicker: in air the ship
/// carries a turn a little wider than in vacuum, as a winged craft would.
pub const AIR_GRIP: f64 = 0.6;

/// Returns the speed in metres per second the full lever asks for `clearance` metres from the
/// nearest surface: slow near the ground or a body, fast in open space.
pub fn top_speed(clearance: f64) -> f64 {
    top_speed_at(clearance, 0.0)
}

/// Returns the speed in metres per second the full lever asks for `clearance` metres from the
/// nearest surface of a body whose circular orbit at the ship runs at `orbital` metres per
/// second: the one speed rule of the ship, for the pilot and the autopilot alike. It grows
/// smoothly with the clearance up to [`SPACE_CAP`], reached 48 km out, or up to [`ORBIT_CAP`]
/// times the orbit speed where that is faster, near a giant, so that a ship can leave it. The
/// boost doubles it.
pub fn top_speed_at(clearance: f64, orbital: f64) -> f64 {
    (SLOW.0 + clearance.max(0.0) / SLOW.1).min(SPACE_CAP.max(ORBIT_CAP * orbital))
}

/// Returns the fastest descent in metres per second the assist flies `clearance` metres above
/// the ground: under the speed that harms the hull near it, faster higher up.
pub fn descent(clearance: f64) -> f64 {
    SINK.0 + (clearance - SINK.1).max(0.0) / REACH
}

/// Returns the acceleration that closes the speed error `err` over a step of `h` seconds
/// without pushing harder than `limit`: in proportion to the error, a little more so that it
/// ends, and never past the speed asked for.
fn close(err: DVec3, limit: f64, h: f64) -> DVec3 {
    let size = err.length();
    let pull = (size / SETTLE + FIRM).min(limit).min(size / h.max(1e-9));
    err.normalize_or_zero() * pull
}

/// What the speed hold knows about the ship.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cruise {
    /// Velocity relative to the ground or to the nearest body, in the axes of the frame.
    pub vel: DVec3,
    /// Unit direction of the nose.
    pub nose: DVec3,
    /// Unit direction the vertical thrusters push along: the local up near a ground, the roof
    /// in space.
    pub rise: DVec3,
    /// Unit direction away from the ground under the ship, zero in space.
    pub up: DVec3,
    /// Main engine lever from 0 to 1.
    pub throttle: f64,
    /// Vertical stick from -1 to 1.
    pub lift: f64,
    /// Whether the boost is held: twice the speed, and a harder push.
    pub boost: bool,
    /// Whether the brake is held: the ship stops, and harder than at idle.
    pub brake: bool,
    /// Metres to the nearest surface.
    pub clearance: f64,
    /// Whether that surface is ground or sea within reach of the gear, which a hovering ship
    /// keeps off.
    pub ground: bool,
    /// Share of the push across the nose that is used, from [`AIR_GRIP`] to 1.
    pub grip: f64,
    /// Speed in metres per second of a circular orbit around the nearest body at the ship's
    /// distance from its centre, zero without a body.
    pub orbital: f64,
    /// Density of the air around the ship in kilograms per cubic metre, zero in space: the
    /// assist flies no faster than the engine can hold against its drag.
    pub air: f64,
}

impl Cruise {
    /// Returns true when nothing asks for a speed along the nose: the lever is neutral or the
    /// brake is held.
    fn idle(&self) -> bool {
        self.throttle < NEUTRAL || self.brake
    }

    /// Returns the climb in metres per second the vertical stick asks for with a top speed of
    /// `top`: over a ground with the lever idle the steady rates of [`HOVER_RATES`] up to
    /// their ceiling and more above it, else a share of the top speed.
    fn climb(&self, top: f64) -> f64 {
        if !self.idle() || self.up == DVec3::ZERO {
            return self.lift * top * CLIMB_SHARE;
        }
        let rate = if self.lift > 0.0 {
            HOVER_RATES.0
        } else {
            HOVER_RATES.1
        };
        let high = (self.clearance - HOVER_CEILING).max(0.0) / REACH;
        self.lift * (rate + high * CLIMB_SHARE)
    }

    /// Returns the velocity the pilot asks for: the lever along the nose and the vertical
    /// stick along `rise`, never faster than the engine holds against the drag of the air, and
    /// never pushed past [`heat_cap`], a mild glow of the nose in that air: a ship already
    /// faster, falling in from orbit, keeps its speed and is left to the air to slow.
    /// Toward the ground it is never faster than a descent the gear can take, and a hovering
    /// ship that is not sunk keeps its gear [`FLOOR`] above the ground.
    pub fn target(&self) -> DVec3 {
        let (boost, push) = if self.boost {
            (BOOST_SPEED, BOOST)
        } else {
            (1.0, 1.0)
        };
        let top = (top_speed_at(self.clearance, self.orbital) * boost)
            .min(terminal(self.air, push))
            .min(heat_cap(self.air));
        let ahead = if self.idle() { 0.0 } else { self.throttle };
        let asked = self.nose * (ahead * top) + self.rise * self.climb(top);
        let sink = -asked.dot(self.up) - descent(self.clearance);
        let held = asked + self.up * sink.max(0.0);
        if !(self.ground && self.idle() && self.lift >= 0.0) {
            return held;
        }
        let floor = (FLOOR.0 - self.clearance) * FLOOR.1;
        held + self.up * (floor - held.dot(self.up)).max(0.0)
    }

    /// Returns the acceleration the assist adds over a step of `h` seconds, gravity and drag
    /// left aside: the caller cancels both in full.
    ///
    /// Along the nose the engine makes the speed asked for. Across it the thrusters bring the
    /// path onto the nose. At idle or under the brake there is no nose to follow: the ship
    /// sheds all of its speed, whatever the air.
    pub fn push(&self, h: f64) -> DVec3 {
        let err = self.target() - self.vel;
        let pace = top_speed_at(self.clearance, self.orbital).max(self.vel.length());
        let hard = if self.boost || self.brake { HARD } else { 1.0 };
        let thrust = (THRUST.0 + pace / THRUST.1) * hard;
        if self.idle() {
            return close(err, thrust, h);
        }
        let along = self.nose * err.dot(self.nose);
        let side = SIDE.0 + pace * SIDE.1;
        close(along, thrust, h) + close(err - along, side, h) * self.grip
    }
}
