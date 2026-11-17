//! Chunk identifiers: the nodes of the quadtree laid over each cube face.

use bevy::math::DVec3;

use super::cube::Face;

/// How far past a face edge a point is pushed to land on the neighbouring face, in face units.
const NUDGE: f64 = 1e-6;

/// A square node of the quadtree of one face.
///
/// Level 0 is the whole face. Level `n` cuts it into `2^n` by `2^n` nodes, `x` along u and `y` along v.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkId {
    /// Face the node lies on.
    pub face: Face,
    /// Depth in the quadtree.
    pub level: u8,
    /// Column, from 0 at `u = -1`.
    pub x: u32,
    /// Row, from 0 at `v = -1`.
    pub y: u32,
}

impl ChunkId {
    /// Returns the face coordinates `(u, v)` of the point `(s, t)` of the node, each from 0 to 1.
    pub fn uv(self, s: f64, t: f64) -> (f64, f64) {
        let n = (1u64 << self.level) as f64;
        (
            (f64::from(self.x) + s) / n * 2.0 - 1.0,
            (f64::from(self.y) + t) / n * 2.0 - 1.0,
        )
    }

    /// Returns the unit direction of the point `(s, t)` of the node, each from 0 to 1.
    pub fn dir(self, s: f64, t: f64) -> DVec3 {
        let (u, v) = self.uv(s, t);
        self.face.dir(u, v)
    }

    /// Returns the node of the same level one step away along x and y, each step being -1, 0 or 1.
    ///
    /// A step off the face lands on the adjacent face, where the shared edge may run the other
    /// way and along the other axis. A diagonal step off a cube corner has no node.
    pub fn neighbor(self, dx: i32, dy: i32) -> Option<ChunkId> {
        let n = 1i64 << self.level;
        let x = i64::from(self.x) + i64::from(dx.signum());
        let y = i64::from(self.y) + i64::from(dy.signum());
        match ((0..n).contains(&x), (0..n).contains(&y)) {
            (true, true) => Some(Self {
                x: x as u32,
                y: y as u32,
                ..self
            }),
            (false, false) => None,
            _ => Some(self.across(x, y)),
        }
    }

    /// Returns the node of the adjacent face that touches this face at column `x` and row `y`,
    /// one of which lies one step off the face.
    fn across(self, x: i64, y: i64) -> ChunkId {
        let n = (1i64 << self.level) as f64;
        let edge = |c: i64| ((c as f64 + 0.5) / n * 2.0 - 1.0).clamp(-1.0 - NUDGE, 1.0 + NUDGE);
        let (face, u, v) = Face::locate(self.face.dir(edge(x), edge(y)));
        let cell = |c: f64| (((c + 1.0) * 0.5 * n).floor()).clamp(0.0, n - 1.0) as u32;
        ChunkId {
            face,
            level: self.level,
            x: cell(u),
            y: cell(v),
        }
    }
}
