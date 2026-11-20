//! Positions on a body as a whole cell plus whole millimetres inside it, so a position stays
//! exact to half a millimetre however far from the origin it lies.

use glam::DVec3;

/// Metres on a side of one cell. A position is `cell * CELL + mm / 1000` on each axis, and the
/// millimetres stay within half a cell, 1e9, well inside an `i32`.
pub const CELL: f64 = 2.0e6;

/// A position cut into a cell per axis and millimetres from that cell's centre.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Packed {
    /// The cell of each axis, counted from the origin.
    pub cell: [i64; 3],
    /// Whole millimetres from the centre of the cell on each axis.
    pub mm: [i32; 3],
}

/// Returns `position`, in metres, as a cell and whole millimetres per axis. The error is at
/// most 0.5 mm anywhere; a coordinate that is not a number packs as 0.
pub fn pack_cells(position: DVec3) -> Packed {
    let cell = position.to_array().map(|metres| (metres / CELL).round() as i64);
    let mm = [0, 1, 2].map(|axis| {
        let rest = position[axis] - cell[axis] as f64 * CELL;
        (rest * 1000.0).round() as i32
    });
    Packed { cell, mm }
}

/// Returns the position in metres that [`pack_cells`] stored.
pub fn unpack_cells(packed: Packed) -> DVec3 {
    DVec3::from_array([0, 1, 2].map(|axis| {
        packed.cell[axis] as f64 * CELL + f64::from(packed.mm[axis]) / 1000.0
    }))
}
