//! The airframe as a few convex parts, each given by its corner points in ship-root space: the
//! fuselage, the canopy, the fin, the heat shield, the three pods of the gear, the two engine
//! pods, and the wing and the winglet of each side. They come from the same stations the drawn
//! parts are made of, so what the ship collides with is what is seen. The fittings smaller than
//! 15 cm, the antenna, the ports, the lamps and the hatch, are left out.

use bevy::prelude::*;

use super::blade::{self, FIN, WINGLET};
use super::dome;
use super::frame::{FUSELAGE, SIDES, SPINE, rim};
use super::outside::SHIELD;
use super::struts::{self, MAIN, NOSE};
use super::tail::{self, ENGINES};
use super::wing;

/// Returns the corner points of each convex part of the airframe in ship-root space.
pub fn hulls() -> Vec<Vec<Vec3>> {
    let mut parts = vec![
        fuselage(),
        dome::corners(),
        blade::corners(0.0, &FIN),
        shield(),
        struts::corners(0.0, &NOSE),
    ];
    for side in [-1.0, 1.0] {
        parts.push(wing::corners(side));
        parts.push(blade::corners(3.2 * side, &WINGLET));
        parts.push(struts::corners(1.9 * side, &MAIN));
    }
    parts.extend(ENGINES.map(engine));
    parts
}

/// Returns the corner points of the heat shield.
fn shield() -> Vec<Vec3> {
    let corner = |[z, half, top, bottom]: [f32; 4]| {
        [(-half, top), (half, top), (half, bottom), (-half, bottom)]
            .map(|(x, y)| Vec3::new(x, y, z))
    };
    SHIELD.iter().flat_map(|&station| corner(station)).collect()
}

/// Returns the corner points of the engine pod on the axis `at`, with the exit of its nozzle.
fn engine(at: Vec2) -> Vec<Vec3> {
    let mut points = tail::corners(at);
    points.extend(tail::exit(at));
    points
}

/// Returns the corners of every station of the fuselage, and its two ends.
fn fuselage() -> Vec<Vec3> {
    let rings = FUSELAGE
        .iter()
        .flat_map(|&station| (0..SIDES).map(move |i| rim(station, i)));
    rings.chain([SPINE[0], SPINE[2]]).collect()
}
