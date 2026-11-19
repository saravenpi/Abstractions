//! Hits: a contact point that meets the ground faster than it can take loses its approach
//! speed in one step, as an impulse on the whole rigid ship, and slides with Coulomb friction.
//! Without it the contact springs store the impact and hand it back as a bounce.

use bevy::math::DVec3;

use super::body::Ship;
use super::landing::{Contact, Point, Rig, SPRINGS};

/// Squared radius of gyration of the ship in square metres: how hard a force off centre turns it.
pub const GYRATION: f64 = 4.0;

/// Share of its spin the ship keeps through each step in which a point is hit. A hit off
/// centre spins the ship, and the next point to come down would throw the first one back off
/// the ground like a seesaw: the crumpling hull eats that spin instead.
const SPIN_KEPT: f64 = 0.5;

/// Approach speed in metres per second from which a touchdown is a crash: on the locked gear,
/// and on the hull or a folded gear, which take much less.
pub const CRASH: [f64; 2] = [25.0, 10.0];

/// Friction coefficient of a hit: the share of the stopping impulse that brakes the slide.
const SCRAPE: f64 = 0.8;

/// Depth in metres the deepest hit point is left in the ground when the ship is set back on it,
/// so that it keeps touching and the rest of the ship coming down is the same touchdown, not a
/// new one.
const SKIN: f64 = 0.02;

/// Times the hits of one step are worked over, each point in turn: stopping one point turns
/// the ship onto the others, and a few rounds settle them all.
const ROUNDS: usize = 8;

/// The velocity and the spin of the ship while the hits of a step are worked out.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Motion {
    /// Velocity in frame axes.
    pub vel: DVec3,
    /// Spin around the body axes.
    pub spin: DVec3,
}

impl Motion {
    /// Returns the motion `ship` has now.
    pub fn of(ship: &Ship) -> Self {
        Self {
            vel: ship.vel,
            spin: ship.spin,
        }
    }

    /// Returns the velocity in frame axes of the point `body` of `ship`, in body axes.
    pub fn at(&self, ship: &Ship, body: DVec3) -> DVec3 {
        self.vel + ship.rot * self.spin.cross(body)
    }

    /// Returns how much the point `body` gives to an impulse along the frame direction `dir`:
    /// 1 at the centre, more off centre, where part of the impulse turns the ship.
    pub fn give(ship: &Ship, body: DVec3, dir: DVec3) -> f64 {
        1.0 + body.cross(ship.rot.inverse() * dir).length_squared() / GYRATION
    }

    /// Applies the impulse `kick`, a change of velocity in frame axes, at the point `body`.
    pub fn push(&mut self, ship: &Ship, body: DVec3, kick: DVec3) {
        self.vel += kick;
        self.spin += body.cross(ship.rot.inverse() * kick) / GYRATION;
    }
}

/// Books on `out` the approach of the point `p` as the ship comes in: the fastest one, where
/// it is, whether it is a crash and how far it is above the safe speed of its point.
pub fn note(ship: &Ship, rig: &Rig, p: &Point, out: &mut Contact) {
    if p.depth <= 0.0 {
        return;
    }
    let closing = -Motion::of(ship).at(ship, p.body).dot(rig.normal);
    if closing > out.impact {
        (out.impact, out.at) = (closing, ship.pos + ship.rot * p.body);
        out.crash = closing >= CRASH[p.kind];
    }
    out.excess = out.excess.max(closing - SPRINGS[p.kind].2);
}

/// Works once over the hit of the point `p`, whose stopping impulse so far is `taken`: a point
/// that comes in faster than its safe speed is hit, and a hit point is stopped along the
/// ground normal. Its impulse may shrink again as the other points take their share, but it
/// never pulls the ship down.
fn strike(ship: &Ship, rig: &Rig, p: &Point, motion: &mut Motion, taken: &mut Option<f64>) {
    let closing = -motion.at(ship, p.body).dot(rig.normal);
    if p.depth <= 0.0 || (taken.is_none() && closing <= SPRINGS[p.kind].2) {
        return;
    }
    let had = taken.unwrap_or(0.0);
    let now = (had + closing / Motion::give(ship, p.body, rig.normal)).max(0.0);
    motion.push(ship, p.body, rig.normal * (now - had));
    *taken = Some(now);
}

/// Takes from `motion` the approach of every one of `points` that comes in faster than its
/// safe speed, and brakes the slide of the ship by up to [`SCRAPE`] times what stopped it. The
/// brake acts on the whole ship and does not turn it: a crash slides to a stop, it does not
/// tumble. Returns how far in metres the caller has to move the ship out along the normal to
/// set its deepest hit point back on the ground, [`SKIN`] deep: a fast ship is already well in
/// when the hit is seen, and whatever depth were left to the springs would come back as a bounce.
pub fn absorb(ship: &Ship, rig: &Rig, points: &[Point; 9], motion: &mut Motion) -> f64 {
    let mut taken = [None; 9];
    for _ in 0..ROUNDS {
        for (p, taken) in points.iter().zip(&mut taken) {
            strike(ship, rig, p, motion, taken);
        }
    }
    let stop: f64 = taken.iter().flatten().sum();
    if taken.iter().any(Option::is_some) {
        motion.spin *= SPIN_KEPT;
    }
    let slide = motion.vel - rig.normal * motion.vel.dot(rig.normal);
    motion.vel -= slide.clamp_length_max(SCRAPE * stop);
    let hit = points
        .iter()
        .zip(taken)
        .filter(|(_, taken)| taken.is_some());
    (hit.map(|(p, _)| p.depth).fold(0.0, f64::max) - SKIN).max(0.0)
}
