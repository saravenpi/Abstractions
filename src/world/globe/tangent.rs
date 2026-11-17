//! Tangent charts: a metric map of the ground around one point of the globe.

use bevy::math::{DVec2, DVec3};

/// An azimuthal equidistant chart: distances and bearings from the origin are true.
///
/// Chart x grows east and chart z grows south, so `(x, up, z)` is right-handed like flat `(X, Y, Z)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tangent {
    /// Unit direction of the point the chart is centred on.
    pub origin: DVec3,
    /// Unit direction of chart +x at the origin.
    pub east: DVec3,
    /// Unit direction of chart -z at the origin.
    pub north: DVec3,
    /// Radius of the globe in metres.
    pub radius: f64,
    /// Chart coordinates of the origin.
    pub offset: DVec2,
}

impl Tangent {
    /// Builds the chart centred on direction `origin` of a globe of `radius` metres.
    ///
    /// Chart +x points along `east` made perpendicular to `origin`, and `offset` maps to the origin.
    pub fn new(origin: DVec3, east: DVec3, radius: f64, offset: DVec2) -> Self {
        let origin = origin.normalize();
        let east = (east - origin * east.dot(origin)).normalize();
        Self {
            origin,
            east,
            north: origin.cross(east),
            radius,
            offset,
        }
    }

    /// Returns the unit direction at chart position `p`, in metres.
    pub fn dir(&self, p: DVec2) -> DVec3 {
        let d = p - self.offset;
        let len = d.length();
        if len < 1e-12 {
            return self.origin;
        }
        let along = (self.east * d.x - self.north * d.y) / len;
        let angle = len / self.radius;
        self.origin * angle.cos() + along * angle.sin()
    }

    /// Returns the chart position in metres of unit direction `dir`.
    pub fn locate(&self, dir: DVec3) -> DVec2 {
        let cos = dir.dot(self.origin);
        let side = dir - self.origin * cos;
        let sin = side.length();
        if sin < 1e-15 {
            return self.offset;
        }
        let reach = sin.atan2(cos) * self.radius / sin;
        self.offset + DVec2::new(side.dot(self.east), -side.dot(self.north)) * reach
    }
}
