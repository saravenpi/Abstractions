//! The grid the broad cover is kept on: the texels of a cube around the planet, six faces of
//! [`FACE`] by [`FACE`], and snapshots [`SNAPSHOT`] seconds apart. The cover of a place is read
//! between the four texels around it and between the two snapshots around the second, which is
//! what the graphics card does with the cover map, so the rain and the picture agree.
//!
//! The faces follow the cube map order of the graphics card: +X, -X, +Y, -Y, +Z, -Z.

use bevy::math::DVec3;

/// Texels along one side of a face of the grid.
pub const FACE: i32 = 64;

/// Texels of the whole grid: six faces, each row after row.
pub const TEXELS: usize = 6 * (FACE * FACE) as usize;

/// Seconds of universe time between two snapshots of the cover.
pub const SNAPSHOT: f64 = 16.0;

/// For each face, the axis it looks along, the axis its columns run along and the axis its
/// rows run along.
const AXES: [[DVec3; 3]; 6] = [
    [DVec3::X, DVec3::NEG_Z, DVec3::NEG_Y],
    [DVec3::NEG_X, DVec3::Z, DVec3::NEG_Y],
    [DVec3::Y, DVec3::X, DVec3::Z],
    [DVec3::NEG_Y, DVec3::X, DVec3::NEG_Z],
    [DVec3::Z, DVec3::X, DVec3::NEG_Y],
    [DVec3::NEG_Z, DVec3::NEG_X, DVec3::NEG_Y],
];

/// Returns the unit direction of the centre of texel `column`, `row` of `face`. A texel one
/// past the edge of a face is folded round that edge onto the next face, where it is the
/// texel that touches the same stretch of the edge: the neighbour the graphics card reads.
pub fn texel_dir(face: usize, column: i32, row: i32) -> DVec3 {
    let [major, along, down] = AXES[face];
    let side = |texel: i32| 2.0 * (f64::from(texel) + 0.5) / f64::from(FACE) - 1.0;
    let fold = |side: f64| (side.clamp(-1.0, 1.0), (side.abs() - 1.0).max(0.0));
    let ((x, past_x), (y, past_y)) = (fold(side(column)), fold(side(row)));
    (major * (1.0 - past_x - past_y) + along * x + down * y).normalize()
}

/// Returns the unit direction of the centre of texel number `index` of the whole grid, which
/// counts the faces one after the other and each face row after row, as the texture does.
pub fn texel(index: usize) -> DVec3 {
    let side = FACE as usize;
    let within = index % (side * side);
    texel_dir(
        index / (side * side) % 6,
        (within % side) as i32,
        (within / side) as i32,
    )
}

/// The four texels around a place and how far the place is from the first towards the others.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Patch {
    /// The face that holds the place.
    pub face: usize,
    /// Column and row of the first of the four texels.
    pub corner: [i32; 2],
    /// Share of the way from that texel to the next column and to the next row.
    pub part: [f32; 2],
}

impl Patch {
    /// Returns the patch around the unit direction `dir`.
    pub fn around(dir: DVec3) -> Self {
        let towards = |face: &usize| dir.dot(AXES[*face][0]);
        let face = (0..6)
            .max_by(|a, b| towards(a).total_cmp(&towards(b)))
            .unwrap_or(0);
        let [major, along, down] = AXES[face];
        let texel =
            |axis: DVec3| 0.5 * (dir.dot(axis) / dir.dot(major) + 1.0) * f64::from(FACE) - 0.5;
        let (column, row) = (texel(along), texel(down));
        Self {
            face,
            corner: [column.floor() as i32, row.floor() as i32],
            part: [(column - column.floor()) as f32, (row - row.floor()) as f32],
        }
    }

    /// Returns the unit directions of the four texels: the first, the next column, the next
    /// row, and the next of both.
    pub fn dirs(&self) -> [DVec3; 4] {
        let [column, row] = self.corner;
        [(0, 0), (1, 0), (0, 1), (1, 1)].map(|(dx, dy)| texel_dir(self.face, column + dx, row + dy))
    }

    /// Returns the value at the place between `values` at the four texels of [`Patch::dirs`].
    pub fn blend(&self, values: [f32; 4]) -> f32 {
        let [x, y] = self.part;
        let top = values[0] + (values[1] - values[0]) * x;
        let bottom = values[2] + (values[3] - values[2]) * x;
        top + (bottom - top) * y
    }
}

/// Returns the number of the snapshot before `seconds` of universe time and the share of the
/// way from it to the next one.
pub fn moment(seconds: f64) -> (i64, f32) {
    let along = seconds / SNAPSHOT;
    (along.floor() as i64, (along - along.floor()) as f32)
}
