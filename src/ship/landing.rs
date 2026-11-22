//! Ground contact: gear and hull points pressed against the terrain and floated by the sea.
//!
//! A point that hits the ground faster than it can take loses that speed at once, in
//! [`strike`](super::strike), and the springs only ever push a point back out slowly, so a
//! hard landing or a crash stops dead instead of bouncing.

use bevy::math::DVec3;

use super::body::Ship;
use super::gear::{feet, locked};
use super::rest::{REST_SPEED, REST_SPIN};
use super::strike::{Motion, absorb, note};
use crate::universe::PlanetSpec;

/// Hull points that can hit the ground in body axes: belly, nose, tail, wing tips and roof.
pub const HULL: [DVec3; 6] = [
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
/// Three feet together damp a heave at 0.98 of the critical damping.
pub const SPRINGS: [(f64, f64, f64); 2] = [(50.0, 8.0, 6.0), (120.0, 10.0, 2.0)];

/// Fastest speed in metres per second at which the springs push a point back out of the
/// ground: the spring fades to nothing as the point leaves faster, so no landing bounces.
const REBOUND: f64 = 0.5;

/// Friction coefficient of the ground: the ship holds on slopes whose tangent is below it.
pub const FRICTION: f64 = 0.6;

/// Sideways braking per metre per second of slip, before the friction limit caps it. A point at
/// rest holds its share of the weight along the ground besides, so the braking only has to
/// stop a slip, not to carry the ship: it leaves no creep on a slope.
const GRIP: f64 = 8.0;

/// Buoyancy per metre of depth, the depth at which it stops growing and the drag of water, per point.
const WATER: (f64, f64, f64) = (14.0, 1.5, 1.2);

/// Half width in metres of the stencil that measures the ground slope.
const SLOPE_STEP: f64 = 2.0;

/// What the ground and the sea do to the ship during one step.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Contact {
    /// Acceleration in frame axes, in metres per second squared.
    pub accel: DVec3,
    /// Angular acceleration around the body axes, in radians per second squared.
    pub torque: DVec3,
    /// Change of velocity the hits of this step give at once, in frame axes.
    pub kick: DVec3,
    /// Change of spin the hits of this step give at once, around the body axes.
    pub twist: DVec3,
    /// How far in metres along `normal` the hits of this step left the ship in the ground.
    pub sunk: f64,
    /// Number of gear feet on the ground.
    pub gear: u8,
    /// Number of points of any kind on the ground.
    pub points: u8,
    /// Whether any point touches the ground.
    pub ground: bool,
    /// Whether any point is in the sea.
    pub wet: bool,
    /// Fastest approach speed of a touching point in metres per second.
    pub impact: f64,
    /// How far that speed is above the safe speed of its point, in metres per second.
    pub excess: f64,
    /// Whether that approach is a crash for the kind of point that took it.
    pub crash: bool,
    /// Where that point is, in frame axes.
    pub at: DVec3,
    /// Height of the gear above the ground or the sea in metres.
    pub radar: f64,
    /// Tangent of the ground slope under the ship.
    pub slope: f64,
    /// Unit normal of the surface `radar` is measured to, zero when it is out of reach: the
    /// ground under the ship, or straight up over a sea whose bed no point touches.
    pub normal: DVec3,
}

/// The ground under the ship, shared by every contact point.
pub struct Rig<'a> {
    /// Sea level radius of the planet in metres.
    pub radius: f64,
    /// Terrain height in metres above sea level at a position in the planet frame.
    pub ground: &'a dyn Fn(DVec3) -> f64,
    /// Unit direction away from the planet centre at the ship.
    pub up: DVec3,
    /// Unit normal of the ground under the ship.
    pub normal: DVec3,
    /// Whether the planet has a sea to float on.
    pub sea: bool,
    /// Acceleration of gravity at the ship in frame axes, in metres per second squared.
    pub pull: DVec3,
}

/// A contact point and the ground under it, sampled once per step.
pub struct Point {
    /// Position in body axes.
    pub body: DVec3,
    /// Kind: 0 for a foot of the locked gear, 1 for the hull.
    pub kind: usize,
    /// Terrain height under the point in metres above sea level.
    pub height: f64,
    /// Distance of the point from the planet centre in metres.
    pub r: f64,
    /// How deep the point is in the ground along its normal, in metres.
    pub depth: f64,
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

/// Returns the contact points of `ship` over `rig`: the gear feet where the gear is, then the hull.
fn points(ship: &Ship, rig: &Rig) -> [Point; 9] {
    let (feet, kind) = (feet(ship), usize::from(!locked(ship)));
    std::array::from_fn(|i| {
        let (body, kind) = if i < 3 {
            (feet[i], kind)
        } else {
            (HULL[i - 3], 1)
        };
        let at = ship.pos + ship.rot * body;
        let (height, r) = ((rig.ground)(at), at.length());
        let depth = (rig.radius + height - r) * rig.normal.dot(rig.up);
        Point {
            body,
            kind,
            height,
            r,
            depth,
        }
    })
}

/// Adds to `out` what the ground and sea do to point `p` (`press` load, downhill `scale`),
/// pressing by remaining depth after `out.sunk` and braking slip up to the friction limit
/// so a settled ship stands still on gentle slopes and slides only past it.
fn touch(
    ship: &Ship,
    rig: &Rig,
    (p, press, scale): (&Point, f64, f64),
    motion: &Motion,
    out: &mut Contact,
) {
    let arm = ship.rot * p.body;
    let vel = motion.at(ship, p.body);
    let (mut accel, depth) = (DVec3::ZERO, p.depth - out.sunk);
    if depth > 0.0 {
        let closing = -vel.dot(rig.normal);
        let slip = vel + rig.normal * closing;
        let downhill = rig.pull - rig.normal * rig.pull.dot(rig.normal);
        let hold = -slip * GRIP - downhill * (press * scale);
        accel += rig.normal * press + hold.clamp_length_max(FRICTION * press);
        out.ground = true;
        out.gear += u8::from(p.kind == 0);
        out.points += 1;
    }
    if rig.sea && p.height < 0.0 && p.r < rig.radius {
        accel += rig.up * (WATER.0 * (rig.radius - p.r).min(WATER.1)) - vel * WATER.2;
        out.wet = true;
    }
    out.accel += accel;
    out.torque += ship.rot.inverse() * arm.cross(accel) / super::strike::GYRATION;
}

/// Returns stillness in `[0, 1]` for `motion`: 0 past rest limits, 1 at half and under, so a
/// settling ship holds full slope weight and stops rather than creeping.
fn still(motion: &Motion) -> f64 {
    let slow = 2.0 * (1.0 - motion.vel.length() / REST_SPEED);
    let steady = 2.0 * (1.0 - motion.spin.length() / REST_SPIN);
    slow.clamp(0.0, 1.0) * steady.clamp(0.0, 1.0)
}

/// Books on `out` what the ground and the sea of `rig` do to `ship` during one step: the hits
/// first, then the springs, the grip and the water on what the hits left.
fn press(ship: &Ship, rig: &Rig, out: &mut Contact) {
    let cos = rig.normal.dot(rig.up).max(1e-6);
    out.slope = (1.0 - cos * cos).max(0.0).sqrt() / cos;
    let points = points(ship, rig);
    let mut motion = Motion::of(ship);
    points.iter().for_each(|p| note(ship, rig, p, out));
    out.sunk = absorb(ship, rig, &points, &mut motion);
    let loads = points.each_ref().map(|p| {
        let depth = p.depth - out.sunk;
        if depth <= 0.0 {
            return 0.0;
        }
        let (spring, damper, _) = SPRINGS[p.kind];
        let closing = -motion.at(ship, p.body).dot(rig.normal);
        let fade = (1.0 + closing / REBOUND).clamp(0.0, 1.0);
        (spring * depth + damper * closing).max(0.0) * fade
    });
    let scale = still(&motion) / loads.iter().sum::<f64>().max(1e-9);
    for (p, press) in points.iter().zip(loads) {
        touch(ship, rig, (p, press, scale), &motion, out);
    }
    (out.kick, out.twist) = (motion.vel - ship.vel, motion.spin - ship.spin);
}

/// Returns what the ground and sea of `planet` do to the ship from terrain `ground` heights.
/// Over untouched water, `normal` falls back to vertical because radar reads the sea surface
/// and the bed slope beneath it does not constrain the ship.
pub fn contact(ship: &Ship, planet: &PlanetSpec, ground: &dyn Fn(DVec3) -> f64) -> Contact {
    let (r, sea) = (ship.pos.length(), planet.record.ocean > 0.0);
    let bed = ground(ship.pos);
    let floor = bed.max(if sea { 0.0 } else { f64::NEG_INFINITY });
    let mut out = Contact {
        radar: r - planet.radius - floor - GEAR_DROP,
        ..Contact::default()
    };
    if out.radar > REACH {
        return out;
    }
    let up = ship.pos / r;
    let rig = Rig {
        radius: planet.radius,
        ground,
        up,
        normal: normal(ground, ship.pos, up),
        sea,
        pull: -up * (planet.mu() / (r * r)),
    };
    press(ship, &rig, &mut out);
    let afloat = floor > bed && !out.ground;
    out.normal = if afloat { up } else { rig.normal };
    out
}
