//! The lens of the flat map of a planet: how far it is zoomed in and the chart point it looks
//! at, in f64. At the closest zoom one pixel of the map is a few parts in a hundred million of
//! the chart, finer than an f32 chart point holds far from the middle of the chart, so the place
//! a pixel shows and the pixel a place is drawn at are worked out in f64 from the centre of the
//! view, and only the pixel, a small number, is cast.
//!
//! Chart points run as on the charts of the board computer: -1 to 1 across with north up, the
//! flat map filling the middle half. Pixels are counted from the top left corner of the map,
//! which is `SIZE` pixels wide and high.

use bevy::math::{DVec2, Vec2};

use crate::computer::SIZE;

/// How close to its goal an easing sight snaps onto it: a quarter of a pixel.
const SNAP: f64 = 0.002;

/// How the flat map of a planet is looked at.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sight {
    /// How many times larger than the whole chart the map is drawn.
    pub zoom: f32,
    /// The chart point drawn in the middle of the map.
    pub centre: DVec2,
}

impl Default for Sight {
    /// The whole chart, centred.
    fn default() -> Self {
        Sight {
            zoom: 1.0,
            centre: DVec2::ZERO,
        }
    }
}

impl Sight {
    /// Returns the pixel the chart point `at` is drawn at, from its gap to the centre in f64.
    pub fn to_map(self, at: DVec2) -> Vec2 {
        let seen = (at - self.centre) * f64::from(self.zoom);
        ((DVec2::new(seen.x, -seen.y) * 0.5 + 0.5) * f64::from(SIZE)).as_vec2()
    }

    /// Returns the chart point drawn at `pixel`.
    pub fn to_chart(self, pixel: Vec2) -> DVec2 {
        let seen = (pixel.as_dvec2() / f64::from(SIZE) - 0.5) * 2.0;
        DVec2::new(seen.x, -seen.y) / f64::from(self.zoom) + self.centre
    }

    /// Returns the sight `share` of the way to `goal`: the zoom by ratio, so each step looks the
    /// same size, and the centre in a straight line. Close enough, it is the goal itself.
    pub fn toward(self, goal: Sight, share: f32) -> Sight {
        let zoom = self.zoom * (goal.zoom / self.zoom).powf(share);
        let centre = self.centre.lerp(goal.centre, f64::from(share));
        let gap = centre.distance(goal.centre) * f64::from(zoom);
        let off = gap + f64::from((zoom / goal.zoom).ln().abs());
        match off < SNAP || share >= 1.0 {
            true => goal,
            false => Sight { zoom, centre },
        }
    }
}

#[cfg(test)]
#[path = "sight_tests.rs"]
mod tests;
