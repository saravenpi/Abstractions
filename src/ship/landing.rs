//! Ground contact: gear and hull points pressed against the terrain and floated by the sea.

use bevy::math::DVec3;

use super::body::Ship;
use crate::universe::PlanetSpec;

/// Feet of the landing gear in body axes: nose, left and right.
pub const GEAR: [DVec3; 3] = [
    DVec3::new(0.0, -1.25, -2.4),
    DVec3::new(-1.9, -1.25, 1.5),
    DVec3::new(1.9, -1.25, 1.5),
];

/// Hull points that can hit the ground in body axes: belly, nose, tail, wing tips and roof.
const HULL: [DVec3; 6] = [
    DVec3::new(0.0, -0.65, 0.0),
    DVec3::new(0.0, -0.1, -4.2),
    DVec3::new(0.0, 0.2, 3.4),
    DVec3::new(-3.2, -0.3, 1.6),
    DVec3::new(3.2, -0.3, 1.6),
    DVec3::new(0.0, 1.3, 0.5),
];

/// Distance in metres from the hull centre down to the feet of the gear.
pub const GEAR_DROP: f64 = 1.25;

/// Height of the gear above the ground in metres beyond which contact is not tested.
const REACH: f64 = 12.0;

/// Spring, damper and safe touchdown speed of a gear foot and of a hull point, per unit of mass.
const SPRINGS: [(f64, f64, f64); 2] = [(50.0, 8.0, 6.0), (120.0, 10.0, 2.0)];

/// Share of the damping that is left while the ship bounces after a hard landing.
const BOUNCE_DAMPING: f64 = 0.2;

/// Friction coefficient of the ground: the ship holds on slopes whose tangent is below it.
pub const FRICTION: f64 = 0.6;

/// Sideways braking per metre per second of slip, before the friction limit caps it.
const GRIP: f64 = 8.0;

/// Buoyancy per metre of depth, the depth at which it stops growing and the drag of water, per point.
const WATER: (f64, f64, f64) = (14.0, 1.5, 1.2);

/// Squared radius of gyration of the ship in square metres: how hard a force off centre turns it.
const GYRATION: f64 = 4.0;

/// Half width in metres of the stencil that measures the ground slope.
const SLOPE_STEP: f64 = 2.0;

/// What the ground and the sea do to the ship during one step.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Contact {
    /// Acceleration in frame axes, in metres per second squared.
    pub accel: DVec3,
    /// Angular acceleration around the body axes, in radians per second squared.
    pub torque: DVec3,
    /// Number of gear feet on the ground.
    pub gear: u8,
    /// Whether any point touches the ground.
    pub ground: bool,
    /// Whether any point is in the sea.
    pub wet: bool,
    /// Fastest approach speed of a touching point in metres per second.
    pub impact: f64,
    /// How far that speed is above the safe speed of its point, in metres per second.
    pub excess: f64,
    /// Height of the gear above the ground or the sea in metres.
    pub radar: f64,
    /// Tangent of the ground slope under the ship.
    pub slope: f64,
}

/// The ground under the ship, shared by every contact point.
struct Rig<'a> {
    /// Sea level radius of the planet in metres.
    radius: f64,
    /// Terrain height in metres above sea level at a position in the planet frame.
    ground: &'a dyn Fn(DVec3) -> f64,
    /// Unit direction away from the planet centre at the ship.
    up: DVec3,
    /// Unit normal of the ground under the ship.
    normal: DVec3,
    /// Share of the damping in use.
    damping: f64,
}

/// Returns the unit normal of the ground around `pos`, from heights sampled on a cross.
fn normal(ground: &dyn Fn(DVec3) -> f64, pos: DVec3, up: DVec3) -> DVec3 {
    let east = up.any_orthonormal_vector();
    let north = up.cross(east);
    let rise = |axis: DVec3| {
        (ground(pos + axis * SLOPE_STEP) - ground(pos - axis * SLOPE_STEP)) / (2.0 * SLOPE_STEP)
    };
    (up - east * rise(east) - north * rise(north)).normalize()
}

/// Adds to `out` what the ground and the sea do to the contact point `body` of kind `kind` (0 gear, 1 hull).
fn touch(ship: &Ship, rig: &Rig, body: DVec3, kind: usize, out: &mut Contact) {
    let arm = ship.rot * body;
    let at = ship.pos + arm;
    let (height, r) = ((rig.ground)(at), at.length());
    let vel = ship.vel + ship.rot * ship.spin.cross(body);
    let (spring, damper, safe) = SPRINGS[kind];
    let depth = (rig.radius + height - r) * rig.normal.dot(rig.up);
    let mut accel = DVec3::ZERO;
    if depth > 0.0 {
        let closing = -vel.dot(rig.normal);
        let press = (spring * depth + damper * rig.damping * closing).max(0.0);
        let slip = vel + rig.normal * closing;
        let grip = GRIP.min(FRICTION * press / slip.length().max(1e-9));
        accel += rig.normal * press - slip * grip;
        out.ground = true;
        out.gear += u8::from(kind == 0);
        out.impact = out.impact.max(closing);
        out.excess = out.excess.max(closing - safe);
    }
    if height < 0.0 && r < rig.radius {
        accel += rig.up * (WATER.0 * (rig.radius - r).min(WATER.1)) - vel * WATER.2;
        out.wet = true;
    }
    out.accel += accel;
    out.torque += ship.rot.inverse() * arm.cross(accel) / GYRATION;
}

/// Returns what the ground and the sea of `planet` do to the ship, which is in its surface frame.
///
/// `ground` gives the terrain height in metres above sea level at a position in the planet frame.
pub fn contact(ship: &Ship, planet: &PlanetSpec, ground: &dyn Fn(DVec3) -> f64) -> Contact {
    let r = ship.pos.length();
    let floor = ground(ship.pos).max(0.0);
    let mut out = Contact {
        radar: r - planet.radius - floor - GEAR_DROP,
        ..Contact::default()
    };
    if out.radar > REACH {
        return out;
    }
    let up = ship.pos / r;
    let normal = normal(ground, ship.pos, up);
    let cos = normal.dot(up).max(1e-6);
    out.slope = (1.0 - cos * cos).max(0.0).sqrt() / cos;
    let rig = Rig {
        radius: planet.radius,
        ground,
        up,
        normal,
        damping: if ship.bounce > 0.0 {
            BOUNCE_DAMPING
        } else {
            1.0
        },
    };
    GEAR.iter().for_each(|&p| touch(ship, &rig, p, 0, &mut out));
    HULL.iter().for_each(|&p| touch(ship, &rig, p, 1, &mut out));
    out
}
