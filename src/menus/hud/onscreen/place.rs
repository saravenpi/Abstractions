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
/// vertical field of view `fov` radians wide.
///
/// `dir` is in the camera frame: +X right, +Y up, forward is -Z. A target in front and inside
/// the screen inset by `inset` pixels sits at its projected place. Any other target slides to
/// the border of that safe rectangle along the ray from the centre toward it, its height along
/// the border following its real height, with an arrow that points the way to turn. A target
/// behind is never mirrored: it keeps the side it is on.
pub fn place(dir: Vec3, fov: f32, screen: Vec2, inset: f32) -> Placed {
    let half = screen * 0.5;
    let safe = (half - Vec2::splat(inset)).max(Vec2::ONE);
    let tan_y = (fov * 0.5).tan();
    let tan_x = tan_y * screen.x / screen.y;
    let toward = Vec2::new(dir.x / tan_x * half.x, -dir.y / tan_y * half.y);
    if dir.z < -1e-4 {
        let ahead = toward / -dir.z;
        if ahead.x.abs() <= safe.x && ahead.y.abs() <= safe.y {
            return Placed {
                pos: half + ahead,
                arrow: None,
            };
        }
    }
    let toward = if toward.length_squared() < 1e-8 {
        Vec2::X
    } else {
        toward
    };
    let reach = (safe.x / toward.x.abs().max(1e-6)).min(safe.y / toward.y.abs().max(1e-6));
    Placed {
        pos: half + toward * reach,
        arrow: Some(toward.x.atan2(-toward.y)),
    }
}

/// Returns the indices of the markers to show, at most [`MAX`]: those that must always show
/// first, then the nearest. Each item is whether it must show and its distance.
pub fn choose(items: &[(bool, f64)], out: &mut Vec<usize>) {
    out.clear();
    out.extend(0..items.len());
    out.sort_by(|a, b| {
        let (first, second) = (items[*a], items[*b]);
        second.0.cmp(&first.0).then(first.1.total_cmp(&second.1))
    });
    out.truncate(MAX);
}

/// Returns the opacity of a marker drawn on the screen whose target lies `angle` radians from
/// the middle of the view: faint near the middle so it never hides what the player looks at.
pub fn dimmed(angle: f32) -> f32 {
    if angle < 15.0_f32.to_radians() {
        0.4
    } else {
        1.0
    }
}
