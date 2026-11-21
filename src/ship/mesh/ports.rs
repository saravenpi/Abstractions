//! The jets that are not the main engines: the reaction-control ports of the attitude thrusters
//! at the nose, the tail and the wing tips, and the four lift nozzles under the belly. Each
//! one is a dark socket sunk in the skin, and the exhaust that comes out of it is drawn from
//! the same table.

use bevy::prelude::*;

use super::frame::rim;
use super::loft::column;
use super::shell::station;
use super::tail::{ENGINES, ring_radius};
use crate::ship::solid::Solid;

/// Sides of a lift nozzle.
const SIDES: u32 = 10;

/// Radius in metres of the lift nozzles: the boss, and the exit of the nozzle inside it. Four of
/// them give the 368 kN of two and a half standard gravities on 15 tonnes, 92 kN each. At a
/// chamber pressure of 7 MPa and a thrust coefficient of 1.5 the throat is 4.4 cm in radius,
/// and an exit of 12.5 cm is an expansion ratio of 8.
pub const LIFT: (f32, f32) = (0.15, 0.125);

/// Where the exits of the four lift nozzles are, under the belly plate, and the height of the
/// plane they point down from.
pub const LIFT_AT: [Vec3; 4] = [
    Vec3::new(-0.34, -0.7, -0.5),
    Vec3::new(0.34, -0.7, -0.5),
    Vec3::new(-0.34, -0.7, 0.9),
    Vec3::new(0.34, -0.7, 0.9),
];

/// Where along z the ports of the tail are, on the engine nacelles behind their bands.
const TAIL: f32 = 3.25;

/// One reaction-control port.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Port {
    /// The middle of the socket, on the skin.
    pub at: Vec3,
    /// The direction the exhaust leaves in: a body axis.
    pub out: Vec3,
    /// The axis of the stick it answers: 0 pitch up, 1 yaw left, 2 roll left.
    pub axis: usize,
    /// Which way of that axis it fires: 1 for the positive way, -1 for the other.
    pub sign: f32,
}

/// Returns the fourteen ports: four at the nose, six at the tail and four at the wing tips.
/// A jet pushes the other way from its exhaust, so the nose is pitched up by the port under it.
pub fn ports() -> [Port; 14] {
    let nose = station(-3.1);
    let [right, top] = [rim(nose, 0), rim(nose, 2)];
    let [left, bottom] = [rim(nose, 4), rim(nose, 6)];
    let port = |at: Vec3, out: Vec3, axis: usize, sign: f32| Port {
        at,
        out,
        axis,
        sign,
    };
    let (l, r) = (ENGINES[0], ENGINES[1]);
    let skin = ring_radius(TAIL) - 0.03;
    let tail = |e: Vec2, dx: f32, dy: f32| Vec3::new(e.x + dx * skin, e.y + dy * skin, TAIL);
    [
        port(bottom, Vec3::NEG_Y, 0, 1.0),
        port(top, Vec3::Y, 0, -1.0),
        port(right, Vec3::X, 1, 1.0),
        port(left, Vec3::NEG_X, 1, -1.0),
        port(tail(l, 0.0, 1.0), Vec3::Y, 0, 1.0),
        port(tail(r, 0.0, 1.0), Vec3::Y, 0, 1.0),
        port(tail(l, 0.0, -1.0), Vec3::NEG_Y, 0, -1.0),
        port(tail(r, 0.0, -1.0), Vec3::NEG_Y, 0, -1.0),
        port(tail(l, -1.0, 0.0), Vec3::NEG_X, 1, 1.0),
        port(tail(r, 1.0, 0.0), Vec3::X, 1, -1.0),
        port(Vec3::new(3.0, -0.25, 1.95), Vec3::NEG_Y, 2, 1.0),
        port(Vec3::new(-3.2, 0.47, 2.5), Vec3::Y, 2, 1.0),
        port(Vec3::new(-3.0, -0.25, 1.95), Vec3::NEG_Y, 2, -1.0),
        port(Vec3::new(3.2, 0.47, 2.5), Vec3::Y, 2, -1.0),
    ]
}

/// Returns the sockets of the fourteen ports: boxes sunk 3 cm in the skin and standing 3 cm out.
pub fn sockets() -> Solid {
    let mut sockets = Solid::default();
    for Port { at, out, .. } in ports() {
        let half = if out.x != 0.0 {
            Vec3::new(0.03, 0.04, 0.05)
        } else {
            Vec3::new(0.025, 0.03, 0.05)
        };
        sockets.slab(at - half, at + half);
    }
    sockets
}

/// Returns the four lift nozzles: a boss that stands out of the belly plate and the black of
/// the exit sunk in its end.
pub fn nozzles() -> Solid {
    let mut nozzles = Solid::default();
    let black = LIFT.1 - 0.015;
    for at in LIFT_AT {
        let x = Vec3::new(at.x, 0.0, at.z);
        column(
            &mut nozzles,
            x,
            &[(-0.58, LIFT.0), (at.y + 0.01, LIFT.0)],
            (SIDES, 0),
        );
        column(
            &mut nozzles,
            x,
            &[(-0.66, black), (at.y, black)],
            (SIDES, 4),
        );
    }
    nozzles
}
