//! Where a marker shows on the screen: pure geometry from the direction to the target in the
//! camera frame to a pixel, and an arrow when the target is out of view.
//!
//! The pixel comes through the camera as it really is this frame: its field of view, which the
//! view widens with speed and in third person, and the picture it renders, which the game view
//! shows at a whole scale and crops evenly where the window is not a multiple of it.

use bevy::prelude::*;

use crate::render::Fit;

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

/// What the game camera sees, laid on the interface.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frustum {
    /// The vertical field of view in radians.
    pub fov: f32,
    /// The rectangle in interface pixels the whole picture of the camera covers. It may reach a
    /// little past the window, which crops the part of a game pixel that does not fit.
    pub image: Rect,
}

impl Frustum {
    /// Returns the frustum of a camera seeing `fov` radians up and down whose picture fills a
    /// screen of exactly `screen` interface pixels.
    #[cfg(test)]
    pub fn whole(fov: f32, screen: Vec2) -> Self {
        let image = Rect::from_corners(Vec2::ZERO, screen);
        Self { fov, image }
    }

    /// Returns the frustum of a camera seeing `fov` radians up and down whose picture `fit` lays
    /// on the window, `per` physical pixels to an interface pixel: `fit.scale` window pixels to a
    /// game pixel, cut by `fit.crop` at the left and the top, as the game view shows it.
    pub fn fitted(fov: f32, fit: Fit, per: f32) -> Self {
        let per = per.max(1e-3);
        let min = -fit.crop.as_vec2() / per;
        let size = (fit.size * fit.scale).as_vec2() / per;
        let image = Rect::from_corners(min, min + size);
        Self { fov, image }
    }

    /// Returns what a game camera with `projection` sees on `window` under an interface scale of
    /// `ui`: the field of view the camera has this frame, or `fov` without a perspective one.
    pub fn of(projection: Option<&Projection>, fov: f32, window: &Window, ui: f32) -> Self {
        let fov = match projection {
            Some(Projection::Perspective(perspective)) => perspective.fov,
            _ => fov,
        };
        let per = window.scale_factor() * ui;
        Self::fitted(fov, Fit::of(window.physical_size()), per)
    }
}

/// Returns where a target in `dir` is drawn through `lens`, with its icon kept inside `safe`,
/// in interface pixels from the top left of the window.
///
/// `dir` is in the camera frame: +X right, +Y up, forward is -Z. A target in front whose
/// projected place is inside `safe` sits there. Any other target slides to the border of
/// `safe` along the ray from the middle of the screen toward it, with an arrow that points the
/// way to turn. A target to the side keeps its real height along the border. A target behind
/// is never mirrored: it keeps the side it is on and sinks toward the bottom border the farther
/// behind it is, so behind always reads as the sides and the bottom, never the top.
pub fn place(dir: Vec3, lens: Frustum, safe: Rect) -> Placed {
    let (half, middle) = (lens.image.half_size(), lens.image.center());
    let tan_y = (lens.fov * 0.5).tan();
    let tan_x = tan_y * half.x / half.y.max(1e-6);
    let toward = Vec2::new(dir.x / tan_x * half.x, -dir.y / tan_y * half.y);
    if dir.z < -1e-4 {
        let pos = middle + toward / -dir.z;
        if safe.contains(pos) {
            return Placed { pos, arrow: None };
        }
    }
    let toward = toward + Vec2::Y * dir.z.max(0.0) * lens.image.height();
    let toward = if toward.length_squared() < 1e-8 {
        Vec2::X
    } else {
        toward
    };
    let room = Vec2::select(
        toward.cmpgt(Vec2::ZERO),
        safe.max - middle,
        middle - safe.min,
    );
    let reach = (room / toward.abs().max(Vec2::splat(1e-6))).min_element();
    Placed {
        pos: middle + toward * reach.max(0.0),
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
