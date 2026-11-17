//! The cube sphere: six faces, each an equal-angle chart of a sixth of the globe.

use std::f64::consts::FRAC_PI_4;

use bevy::math::DVec3;

/// One face of the cube that wraps the globe, named by the axis it looks along.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Face {
    /// Face around +X.
    PosX,
    /// Face around -X.
    NegX,
    /// Face around +Y, the north pole.
    PosY,
    /// Face around -Y, the south pole.
    NegY,
    /// Face around +Z.
    PosZ,
    /// Face around -Z.
    NegZ,
}

impl Face {
    /// Every face, in a stable order.
    pub const ALL: [Face; 6] = [
        Face::PosX,
        Face::NegX,
        Face::PosY,
        Face::NegY,
        Face::PosZ,
        Face::NegZ,
    ];

    /// Returns the outward normal, the u axis and the v axis of the face.
    ///
    /// On every face `v x u` is the normal, so chart `(x, up, z)` is right-handed like flat `(X, Y, Z)`.
    pub fn axes(self) -> [DVec3; 3] {
        match self {
            Face::PosX => [DVec3::X, DVec3::Z, DVec3::Y],
            Face::NegX => [DVec3::NEG_X, DVec3::Y, DVec3::Z],
            Face::PosY => [DVec3::Y, DVec3::X, DVec3::Z],
            Face::NegY => [DVec3::NEG_Y, DVec3::X, DVec3::NEG_Z],
            Face::PosZ => [DVec3::Z, DVec3::Y, DVec3::X],
            Face::NegZ => [DVec3::NEG_Z, DVec3::X, DVec3::Y],
        }
    }

    /// Returns the unit direction at face coordinates `(u, v)`, each from -1 to 1 across the face.
    ///
    /// Equal steps of `u` are equal angles. Values up to 2 in size reach past the face edge.
    pub fn dir(self, u: f64, v: f64) -> DVec3 {
        let [n, a, b] = self.axes();
        (n + a * (u * FRAC_PI_4).tan() + b * (v * FRAC_PI_4).tan()).normalize()
    }

    /// Returns the coordinates `(u, v)` of `dir` in the chart of this face, also past its edges.
    ///
    /// `dir` must lie on the same side of the globe as the face.
    pub fn project(self, dir: DVec3) -> (f64, f64) {
        let [n, a, b] = self.axes();
        let d = dir.dot(n);
        (
            (dir.dot(a) / d).atan() / FRAC_PI_4,
            (dir.dot(b) / d).atan() / FRAC_PI_4,
        )
    }

    /// Returns the face that holds `dir` and its coordinates `(u, v)` there.
    pub fn locate(dir: DVec3) -> (Face, f64, f64) {
        let a = dir.abs();
        let face = if a.x >= a.y && a.x >= a.z {
            [Face::NegX, Face::PosX][usize::from(dir.x > 0.0)]
        } else if a.y >= a.z {
            [Face::NegY, Face::PosY][usize::from(dir.y > 0.0)]
        } else {
            [Face::NegZ, Face::PosZ][usize::from(dir.z > 0.0)]
        };
        let (u, v) = face.project(dir);
        (face, u, v)
    }
}
