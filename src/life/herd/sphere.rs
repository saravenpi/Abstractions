//! Small sphere helpers for herds: the local east, arcs, great circle paths and bearings.
//!
//! North is the spin axis, world +Y, as in `crate::climate`. East is `Y x up`; at the poles, where it
//! vanishes, `X x up` stands in.

use bevy::math::DVec3;

/// Returns the unit east tangent at unit direction `dir`.
pub(in crate::life) fn east(dir: DVec3) -> DVec3 {
    let east = DVec3::Y.cross(dir);
    if east.length_squared() > 1e-12 {
        east.normalize()
    } else {
        DVec3::X.cross(dir).normalize()
    }
}

/// Returns the angle in radians between unit directions `a` and `b`.
pub(in crate::life) fn arc(a: DVec3, b: DVec3) -> f64 {
    a.cross(b).length().atan2(a.dot(b))
}

/// Returns the point a share `s` of the way from `a` to `b` along their great circle.
pub(in crate::life) fn slerp(a: DVec3, b: DVec3, s: f64) -> DVec3 {
    let angle = arc(a, b);
    if angle < 1e-12 {
        return a;
    }
    ((a * ((1.0 - s) * angle).sin() + b * (s * angle).sin()) / angle.sin()).normalize()
}

/// Returns the unit tangent at `at` along the great circle from `from` to `to`, or east when they meet.
pub(in crate::life) fn bearing(at: DVec3, from: DVec3, to: DVec3) -> DVec3 {
    let normal = from.cross(to);
    if normal.length_squared() < 1e-24 {
        return east(at);
    }
    normal.cross(at).normalize()
}
