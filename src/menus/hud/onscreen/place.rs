//! Where a marker shows on the screen: pure geometry from the direction to the target in the
//! camera frame to a pixel, and an arrow when the target is out of view.

use bevy::prelude::*;

/// Most markers shown over the world at once.
pub const MAX: usize = 6;

/// Where a marker is drawn.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placed {
    /// The centre of the icon in pixels from the top left of the screen.
    pub pos: Vec2,
    /// Clockwise angle in radians from straight up of the arrow that points toward the target,
    /// or nothing while the icon sits on the real place of the target.
    pub arrow: Option<f32>,
}

/// Returns where a target in `dir` is drawn on a screen of `screen` pixels seen through a
/// vertical field of view `fov` radians wide, with its icon kept inside `safe`.
///
/// `dir` is in the camera frame: +X right, +Y up, forward is -Z. A target in front whose
/// projected place is inside `safe` sits there. Any other target slides to the border of
/// `safe` along the ray from the middle of the screen toward it, with an arrow that points the
/// way to turn. A target to the side keeps its real height along the border. A target behind
/// is never mirrored: it keeps the side it is on and sinks toward the bottom border the farther
/// behind it is, so behind always reads as the sides and the bottom, never the top.
pub fn place(dir: Vec3, fov: f32, screen: Vec2, safe: Rect) -> Placed {
    let half = screen * 0.5;
    let tan_y = (fov * 0.5).tan();
    let tan_x = tan_y * screen.x / screen.y;
    let toward = Vec2::new(dir.x / tan_x * half.x, -dir.y / tan_y * half.y);
    if dir.z < -1e-4 {
        let pos = half + toward / -dir.z;
        if safe.contains(pos) {
            return Placed { pos, arrow: None };
        }
    }
    let toward = toward + Vec2::Y * dir.z.max(0.0) * screen.y;
    let toward = if toward.length_squared() < 1e-8 {
        Vec2::X
    } else {
        toward
    };
    let room = Vec2::select(toward.cmpgt(Vec2::ZERO), safe.max - half, half - safe.min);
    let reach = (room / toward.abs().max(Vec2::splat(1e-6))).min_element();
    Placed {
        pos: half + toward * reach.max(0.0),
        arrow: Some(toward.x.atan2(-toward.y)),
    }
}

/// Returns the indices of the markers to show, at most [`MAX`]: by rank, then the nearest. Each
/// item is the rank of a marker, lowest first, and its distance. The order is also the order in
/// which markers take their room on the screen.
pub fn choose(items: &[(u8, f64)], out: &mut Vec<usize>) {
    out.clear();
    out.extend(0..items.len());
    out.sort_by(|a, b| {
        let (first, second) = (items[*a], items[*b]);
        first.0.cmp(&second.0).then(first.1.total_cmp(&second.1))
    });
    out.truncate(MAX);
}
