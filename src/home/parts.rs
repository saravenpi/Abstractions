//! Parts every piece of the base is made of: legs that reach the ground, level decks and lamps.

use bevy::prelude::*;

use super::draft::Draft;
use super::glow::Glow;
use super::soup::Rgb;
use super::tune::SINK;

/// Colour of a glowing part before its material tints it.
pub const WHITE: Rgb = [1.0, 1.0, 1.0];

impl Draft<'_> {
    /// Adds a leg of half width `half` under the point `at` of the piece, from the height `top`
    /// down to the ground and a little below it, and records where it meets the ground.
    pub fn leg(&mut self, at: Vec2, top: f32, half: f32, rgb: Rgb) {
        let ground = self.ground(at);
        let (foot, head) = (Vec3::new(at.x, ground, at.y), Vec3::new(at.x, top, at.y));
        self.feet.push(self.at(foot));
        self.draw(|soup| soup.beam(foot - Vec3::Y * SINK, head, half, rgb));
    }

    /// Adds a level platform between two corners and records its top.
    pub fn deck(&mut self, lo: Vec3, hi: Vec3, rgb: Rgb) {
        let top = [Vec3::new(lo.x, hi.y, lo.z), Vec3::new(hi.x, hi.y, lo.z), hi];
        self.decks.push(top.map(|p| self.at(p)));
        self.draw(|soup| soup.slab(lo, hi, rgb));
    }

    /// Adds a glowing box between two corners of the piece.
    pub fn light(&mut self, glow: Glow, lo: Vec3, hi: Vec3) {
        self.shine(glow, |soup| soup.slab(lo, hi, WHITE));
    }

    /// Records that `glow` casts its light from the point `at` of the piece.
    pub fn lamp(&mut self, glow: Glow, at: Vec3) {
        self.lamps.push((glow, self.at(at)));
    }

    /// Returns the lowest ground under the points `feet` of the piece, in its frame.
    pub fn lowest(&self, feet: &[Vec2]) -> f32 {
        feet.iter()
            .map(|foot| self.ground(*foot))
            .fold(f32::MAX, f32::min)
    }
}
