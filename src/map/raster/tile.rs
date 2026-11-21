//! Tiles of a body picture at any zoom: the same flat map as the picture, cut into squares of
//! [`SIDE`] pixels on a pyramid whose level `k` has `2^(k+1)` columns and `2^k` rows, painted
//! from the ground itself.

use std::f64::consts::{PI, TAU};

use bevy::math::{DVec2, DVec3, Vec2};

use super::tiles_view::corner;
use crate::computer::direction;
use crate::map::geo::copy_near;
use crate::map::lat_lon;
use crate::map::level::REFERENCE_MAP;
use crate::map::sight::Sight;

/// Pixels along one side of a tile.
pub const SIDE: usize = 256;

/// Deepest level of the pyramid: a pixel then spans about 1.2 m on an Earth, which the closest
/// zoom of the flat map draws on under two pixels.
pub const LEVEL_MAX: u8 = 16;

/// Zoom of the flat map from which tiles are drawn over the picture.
pub const ZOOM_FROM: f32 = 3.0;

/// Most tiles shown at once.
pub const SHOWN_MAX: usize = 36;

/// One square of the pyramid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TileId {
    /// The level, 0 for a body in two tiles.
    pub level: u8,
    /// The column from longitude 0, going east.
    pub col: u32,
    /// The row from the north pole.
    pub row: u32,
}

impl TileId {
    /// Returns the columns of a level.
    pub fn cols(level: u8) -> u32 {
        2 << level
    }

    /// Returns the rows of a level.
    pub fn rows(level: u8) -> u32 {
        1 << level
    }

    /// Returns the unit direction at the pixel position `(px, py)` of the tile, counted from its
    /// north-west corner: a whole number is a corner of a pixel, a half the middle of one.
    pub fn direction(self, (px, py): (f64, f64)) -> DVec3 {
        let across = f64::from(Self::cols(self.level)) * SIDE as f64;
        let down = f64::from(Self::rows(self.level)) * SIDE as f64;
        let lon = (f64::from(self.col) * SIDE as f64 + px) / across * TAU;
        let lat = (0.5 - (f64::from(self.row) * SIDE as f64 + py) / down) * PI;
        direction(lon, lat)
    }

    /// Returns the pixel of the tile that holds unit direction `dir`, or nothing outside it.
    pub fn pixel(self, dir: DVec3) -> Option<(usize, usize)> {
        let (lat, lon) = lat_lon(dir);
        let across = f64::from(Self::cols(self.level)) * SIDE as f64;
        let down = f64::from(Self::rows(self.level)) * SIDE as f64;
        let x = lon / TAU * across - f64::from(self.col) * SIDE as f64;
        let y = (0.5 - lat / PI) * down - f64::from(self.row) * SIDE as f64;
        let inside = |v: f64| (0.0..SIDE as f64).contains(&v);
        (inside(x) && inside(y)).then_some((x as usize, y as usize))
    }
}

/// Returns the level whose pixels are about two screen pixels at `zoom`.
pub fn level_for(zoom: f32) -> u8 {
    let texels = REFERENCE_MAP * zoom / (4.0 * SIDE as f32);
    let level = texels.max(1.0).log2().ceil();
    (level as u8).clamp(1, LEVEL_MAX)
}

/// Returns the tiles `lens` looks at on `level`, nearest the middle first and at most
/// [`SHOWN_MAX`], each with its column counted without wrapping, so that it is placed where the
/// lens sees it. The identity of a tile east of the seam wraps round.
pub fn visible(lens: Sight, level: u8) -> Vec<(i64, TileId)> {
    let (cols, rows) = (
        f64::from(TileId::cols(level)),
        f64::from(TileId::rows(level)),
    );
    let half = 1.0 / f64::from(lens.zoom);
    let (x, y) = (lens.centre.x, lens.centre.y);
    let east = |x: f64| ((x + 1.0) * 0.5 * cols).floor() as i64;
    let south = |y: f64| (((0.5 - y) * rows).floor().max(0.0) as i64).min(rows as i64 - 1);
    let mut tiles = Vec::new();
    for row in south(y + half)..=south(y - half) {
        for col in east(x - half)..=east(x + half) {
            let wrapped = col.rem_euclid(cols as i64) as u32;
            tiles.push((
                col,
                TileId {
                    level,
                    col: wrapped,
                    row: row as u32,
                },
            ));
        }
    }
    nearest(
        tiles,
        Vec2::new(((x + 1.0) * 0.5 * cols) as f32, ((0.5 - y) * rows) as f32),
    )
}

/// Returns at most [`SHOWN_MAX`] of `tiles`, those nearest `middle` first, where `middle` is in
/// tiles from the north-west corner of the picture.
fn nearest(mut tiles: Vec<(i64, TileId)>, middle: Vec2) -> Vec<(i64, TileId)> {
    let away = |(col, id): &(i64, TileId)| {
        Vec2::new(*col as f32 + 0.5, id.row as f32 + 0.5).distance_squared(middle)
    };
    tiles.sort_by(|a, b| away(a).total_cmp(&away(b)));
    tiles.truncate(SHOWN_MAX);
    tiles
}

/// Returns how far tile `id` is from what `lens` shows: the levels between it and the level
/// shown, then the chart distance from its middle to the centre of the view, round the turn of
/// longitude the shorter way. The cache drops the farthest first.
pub fn away(id: TileId, lens: Sight) -> (u8, f64) {
    let (cols, rows) = (
        f64::from(TileId::cols(id.level)),
        f64::from(TileId::rows(id.level)),
    );
    let middle = corner(id.level, i64::from(id.col), id.row) + DVec2::new(1.0 / cols, -0.5 / rows);
    let gap = copy_near(middle, lens.centre.x) - lens.centre;
    (id.level.abs_diff(level_for(lens.zoom)), gap.length())
}
