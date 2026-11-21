//! Round parts: the drum that houses a joint, in tables, and the ball.

use bevy::prelude::*;

use super::Rgb;
use super::detail::Detail;
use super::loft::{Ring, Section, loft};
use super::soup::Soup;

/// One closed drum of a table: a joint housing, a disc or a short tube.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Disc {
    /// Middle of the drum.
    pub at: Vec3,
    /// The axis it turns about.
    pub axis: Vec3,
    /// Radius and half length, in meters.
    pub size: [f32; 2],
    /// Number of sides up close.
    pub sides: u32,
    /// Its colour.
    pub tone: Rgb,
    /// Whether it is too small to build from afar.
    pub fine: bool,
}

impl Disc {
    /// Returns a ten-sided drum at `at` about `axis`, of radius `size[0]` and half length `size[1]`.
    pub const fn new(at: [f32; 3], axis: Vec3, size: [f32; 2], tone: Rgb) -> Self {
        Self {
            at: Vec3::from_array(at),
            axis,
            size,
            sides: 10,
            tone,
            fine: false,
        }
    }

    /// Returns the drum with `sides` sides.
    pub const fn sides(mut self, sides: u32) -> Self {
        self.sides = sides;
        self
    }

    /// Returns the drum marked as too small to build from afar.
    pub const fn fine(mut self) -> Self {
        self.fine = true;
        self
    }
}

/// Adds the drums of `list` at the level `detail`.
pub fn discs(soup: &mut Soup, list: &[Disc], detail: Detail) {
    for disc in list.iter().filter(|disc| detail.near() || !disc.fine) {
        let [radius, half] = disc.size;
        let ends = [
            Section::new(-half, radius, radius),
            Section::new(half, radius, radius),
        ];
        let mut body = Soup::default();
        let ring = detail.ring(Ring::Round(disc.sides, 2.0));
        loft(&mut body, &ends, ring, |_, _| disc.tone);
        let turn = Quat::from_rotation_arc(Vec3::Y, disc.axis.normalize());
        soup.extend(&body, (disc.at, turn));
    }
}

/// Adds a ball of `radius` around `at` in the colour `rgb`: five rings of the outline `ring`.
pub fn orb(soup: &mut Soup, at: Vec3, radius: f32, (ring, rgb): (Ring, Rgb)) {
    let rings = [(-0.95, 0.3), (-0.6, 0.8), (0.0, 1.0), (0.6, 0.8), (0.95, 0.3)]
        .map(|(y, r): (f32, f32)| Section::new(y * radius, r * radius, r * radius));
    let mut body = Soup::default();
    loft(&mut body, &rings, ring, |_, _| rgb);
    soup.extend(&body, (at, Quat::IDENTITY));
}
