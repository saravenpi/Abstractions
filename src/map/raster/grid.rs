//! The grid of the body pictures: 512 columns of longitude from 0 going east, 256 rows of
//! latitude from the south pole, cell index = row * 512 + column.
//!
//! Directions are unit vectors in the frame that turns with the body, +Y its spin axis and
//! longitude `atan2(-z, x)`, the inverse of the board computer's `direction`.

use std::f64::consts::{FRAC_PI_2, PI, TAU};

use bevy::math::DVec3;

/// Columns of the grid, one per 0.7 degrees of longitude.
pub const COLS: usize = 512;

/// Rows of the grid, one per 0.7 degrees of latitude.
pub const ROWS: usize = 256;

/// Cells of the grid.
pub const CELLS: usize = COLS * ROWS;

/// 64-bit words of a bitset with a bit per cell.
pub const WORDS: usize = CELLS / 64;

/// Radians of longitude a column spans.
const COLUMN: f64 = TAU / COLS as f64;

/// Radians of latitude a row spans.
const ROW: f64 = PI / ROWS as f64;

/// Returns the longitude east of +X from 0 to a full turn, and the latitude, of unit direction
/// `dir`.
fn lon_lat(dir: DVec3) -> (f64, f64) {
    (
        (-dir.z).atan2(dir.x).rem_euclid(TAU),
        dir.y.clamp(-1.0, 1.0).asin(),
    )
}

/// Returns the cell that holds unit direction `dir`.
pub fn cell_of(dir: DVec3) -> u32 {
    let (lon, lat) = lon_lat(dir);
    let column = ((lon / COLUMN) as usize).min(COLS - 1);
    let row = (((lat + FRAC_PI_2) / ROW) as usize).min(ROWS - 1);
    (row * COLS + column) as u32
}

/// Returns the unit direction of the centre of `cell`.
pub fn centre(cell: u32) -> DVec3 {
    let (row, column) = (cell as usize / COLS, cell as usize % COLS);
    let lon = (column as f64 + 0.5) * COLUMN;
    let lat = (row as f64 + 0.5) * ROW - FRAC_PI_2;
    DVec3::new(lat.cos() * lon.cos(), lat.sin(), -lat.cos() * lon.sin())
}
