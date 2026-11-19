//! The grid the explored ground and the body pictures share: 512 columns of longitude from 0
//! going east, 256 rows of latitude from the south pole, cell index = row * 512 + column.
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

/// 64-bit words of a whole bitset.
pub const WORDS: usize = CELLS / 64;

/// Radians of longitude a column spans.
const COLUMN: f64 = TAU / COLS as f64;

/// Radians of latitude a row spans.
const ROW: f64 = PI / ROWS as f64;

/// Returns the longitude east of +X from 0 to a full turn, and the latitude, of unit direction
/// `dir`.
pub fn lon_lat(dir: DVec3) -> (f64, f64) {
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

/// Calls `each` with the cell under unit direction `dir` and every cell whose centre lies
/// within `angle` radians of it. A cell may come twice.
pub fn disc(dir: DVec3, angle: f64, mut each: impl FnMut(u32)) {
    each(cell_of(dir));
    let (lon, lat) = lon_lat(dir);
    let reach = (angle.sin() / lat.cos().max(f64::EPSILON)).min(1.0);
    let half = match reach >= 1.0 || angle >= FRAC_PI_2 {
        true => PI,
        false => reach.asin(),
    };
    let rows = row_index(lat - angle)..=row_index(lat + angle);
    let first = ((lon - half) / COLUMN).floor() as i64 - 1;
    let count = ((2.0 * half / COLUMN).ceil() as i64 + 3).min(COLS as i64);
    let near = angle.cos();
    for row in rows {
        for step in 0..count {
            let column = (first + step).rem_euclid(COLS as i64) as usize;
            let cell = (row * COLS + column) as u32;
            if centre(cell).dot(dir) >= near {
                each(cell);
            }
        }
    }
}

/// Returns the row of latitude `lat`, clamped to the grid.
fn row_index(lat: f64) -> usize {
    (((lat + FRAC_PI_2) / ROW).max(0.0) as usize).min(ROWS - 1)
}
