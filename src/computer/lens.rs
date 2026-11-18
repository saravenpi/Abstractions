//! The lens of the map: how far it is zoomed in and where it looks, the pixel of a chart
//! point, and the way from one lens to another.
//!
//! Chart points run from -1 to 1 on both axes with north up. Pixels are counted from the top
//! left corner of the map, which is `SIZE` pixels wide and high.

use bevy::math::Vec2;

use super::map::SIZE;

/// Closest zoom: the whole chart seen 32 times larger.
pub const MAX_ZOOM: f32 = 32.0;

/// How close to its goal an easing lens snaps onto it: a quarter of a pixel.
const SNAP: f32 = 0.002;

/// Share of the visible map kept between a revealed mark and the edge.
const MARGIN: f32 = 0.15;

/// How the map is looked at.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lens {
    /// How many times larger than the whole chart the map is drawn, from 1 to [`MAX_ZOOM`].
    pub zoom: f32,
    /// The chart point drawn in the middle of the map.
    pub centre: Vec2,
}

impl Default for Lens {
    /// The whole chart, centred.
    fn default() -> Self {
        Lens {
            zoom: 1.0,
            centre: Vec2::ZERO,
        }
    }
}

impl Lens {
    /// Returns the pixel the chart point `at` is drawn at.
    pub fn to_map(self, at: Vec2) -> Vec2 {
        let seen = (at - self.centre) * self.zoom;
        (Vec2::new(seen.x, -seen.y) * 0.5 + 0.5) * SIZE
    }

    /// Returns the chart point drawn at `pixel`.
    pub fn to_chart(self, pixel: Vec2) -> Vec2 {
        let seen = (pixel / SIZE - 0.5) * 2.0;
        Vec2::new(seen.x, -seen.y) / self.zoom + self.centre
    }

    /// Returns the lens with its zoom inside its range and its centre close enough to the
    /// middle of the chart that the map never shows past the edge of the chart.
    pub fn clamped(self) -> Lens {
        let zoom = self.zoom.clamp(1.0, MAX_ZOOM);
        let reach = Vec2::splat(1.0 - 1.0 / zoom);
        Lens {
            zoom,
            centre: self.centre.clamp(-reach, reach),
        }
    }

    /// Returns the lens zoomed `factor` times closer around `pixel`: the chart point under
    /// that pixel stays under it, unless the edge of the chart stops the map.
    pub fn zoomed(self, pixel: Vec2, factor: f32) -> Lens {
        let held = self.to_chart(pixel);
        let zoom = (self.zoom * factor).clamp(1.0, MAX_ZOOM);
        let centre = held - (held - self.centre) * self.zoom / zoom;
        Lens { zoom, centre }.clamped()
    }

    /// Returns the lens after the chart was dragged by `pixels`.
    pub fn panned(self, pixels: Vec2) -> Lens {
        let moved = Vec2::new(-pixels.x, pixels.y) * 2.0 / (SIZE * self.zoom);
        Lens {
            centre: self.centre + moved,
            ..self
        }
        .clamped()
    }

    /// Returns the lens `share` of the way to `goal`: the zoom by ratio, so each step looks the
    /// same size, and the centre in a straight line. Close enough, it is the goal itself.
    pub fn toward(self, goal: Lens, share: f32) -> Lens {
        let zoom = self.zoom * (goal.zoom / self.zoom).powf(share);
        let centre = self.centre.lerp(goal.centre, share);
        let off = centre.distance(goal.centre) * zoom + (zoom / goal.zoom).ln().abs();
        match off < SNAP || share >= 1.0 {
            true => goal,
            false => Lens { zoom, centre },
        }
    }

    /// Returns the lens moved by the least that shows the chart point `at` with a margin
    /// around it. A point already in view leaves the lens as it is.
    pub fn revealing(self, at: Vec2) -> Lens {
        let room = Vec2::splat((1.0 - MARGIN) / self.zoom);
        let gap = at - self.centre;
        Lens {
            centre: self.centre + gap - gap.clamp(-room, room),
            ..self
        }
        .clamped()
    }
}
