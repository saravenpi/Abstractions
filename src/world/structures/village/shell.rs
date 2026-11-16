//! The walled room every enterable house starts from: plinth, walls with a doorway, stairs and door frame.

use bevy::prelude::*;

use super::kinds::House;
use super::style::{Flavor, Style};
use crate::world::structures::brush::{Build, Mat};
use crate::world::structures::palette::PLANK;

/// Wall thickness.
pub const T: f32 = 0.3;

/// Doorway height, above the 2.1 m the player needs.
pub const DOOR_H: f32 = 2.3;

/// Shape of a walled room with its doorway in the +z wall.
#[derive(Debug, Clone, Copy)]
pub struct Shell {
    /// Half length along x.
    pub hx: f32,
    /// Half depth along z.
    pub hz: f32,
    /// Wall height.
    pub h: f32,
    /// Doorway center along x.
    pub door: f32,
    /// Doorway width, at least 1.1 m.
    pub gap: f32,
    /// Doorway height.
    pub door_h: f32,
    /// Extra platform in front of the door, for porches.
    pub front: f32,
}

impl Shell {
    /// Returns the half sizes of the floor inside the walls, along x and z.
    pub fn inner(&self) -> Vec2 {
        Vec2::new(self.hx - T, self.hz - T)
    }
}

impl House {
    /// Returns the walled room of this kind. The two-storey room is 0.2 m shallower than its lot.
    pub fn shell(self) -> Shell {
        let d = self.dims();
        let (h, door, gap, door_h) = match self {
            House::Shed => (2.5, 0.0, 1.2, 2.2),
            House::Stall => (2.7, 1.5, 1.2, DOOR_H),
            House::TwoStory => (2.5, 0.0, 1.4, 2.2),
            House::Smithy => (2.7, 0.4, 2.6, 2.4),
            House::Barn => (3.4, 0.0, 2.6, 3.0),
            House::Tower => (2.6, 0.0, 1.4, DOOR_H),
            House::Hall => (3.4, 0.0, 2.0, 2.8),
            House::Cottage | House::Long | House::Porch => (2.7, 0.0, 1.4, DOOR_H),
        };
        let hz = if self == House::TwoStory { 2.4 } else { d.y };
        Shell {
            hx: d.x,
            hz,
            h,
            door,
            gap,
            door_h,
            front: d.z,
        }
    }

    /// Draws this kind on the current site, turned by 0.3 rad, in the style of `biome` and lot `variant`.
    #[cfg(test)]
    pub fn draw_variant(self, b: &mut Build, biome: crate::world::Biome, variant: u32) {
        let lot = super::lots::Lot {
            pos: b.site.origin,
            yaw: 0.3,
            floor: b.site.floor,
            depth: 2.0,
            house: self,
            variant,
        };
        super::houses::draw(b, &super::style::style_for(biome, variant), &lot);
    }
}

/// Draws the four walls with the doorway.
pub fn walls(b: &mut Build, s: &Shell, mat: Mat) {
    let (hx, hz, h) = (s.hx, s.hz, s.h);
    let (d0, d1) = (s.door - s.gap * 0.5, s.door + s.gap * 0.5);
    b.boxes(
        &[
            ([-hx, 0.0, -hz], [hx, h, -hz + T]),
            ([-hx, 0.0, -hz + T], [-hx + T, h, hz]),
            ([hx - T, 0.0, -hz + T], [hx, h, hz]),
            ([-hx + T, 0.0, hz - T], [d0, h, hz]),
            ([d1, 0.0, hz - T], [hx - T, h, hz]),
            ([d0, s.door_h, hz - T], [d1, h, hz]),
        ],
        mat,
    );
}

/// Draws the foundation platform (or a stilt deck on posts) and the plank floor inside.
pub fn plinth(b: &mut Build, s: &Shell, st: &Style) {
    let (hx, hz) = (s.hx + 0.5, s.hz + 0.5);
    let lo = Vec3::new(-hx, -b.site.depth, -hz);
    let hi = Vec3::new(hx, 0.0, s.hz + s.front + 0.5);
    if st.flavor == Flavor::Stilt {
        b.cuboid(Vec3::new(lo.x, -0.35, lo.z), hi, Mat::solid(PLANK));
        for (x, z) in [
            (-hx + 0.3, lo.z + 0.3),
            (hx - 0.3, lo.z + 0.3),
            (-hx + 0.3, hi.z - 0.3),
            (hx - 0.3, hi.z - 0.3),
        ] {
            let low = b.ground_at(x, z) - b.base_y - 0.6;
            b.pillar(
                Vec3::new(x, low, z),
                Vec2::new(0.2, -0.35 - low),
                5,
                Mat::deco(st.trim),
            );
        }
    } else {
        b.cuboid(lo, hi, Mat::solid(st.base));
    }
    b.floor_pane(
        Vec2::new(-s.hx + T, -s.hz + T),
        Vec2::new(s.hx - T, s.hz - T),
        0.02,
        PLANK,
    );
}

/// Draws the stairs down from the doorway and the door frame.
pub fn entrance(b: &mut Build, s: &Shell, st: &Style) {
    let z = s.hz + s.front + 0.5;
    let low = b.ground_at(s.door, z + 1.6) - b.base_y;
    if low < -0.1 {
        b.stair(
            (Vec2::new(s.door, z), Vec2::Y),
            1.8,
            (low, 0.0),
            Mat::solid(st.base),
        );
    }
    let (d0, d1) = (s.door - s.gap * 0.5, s.door + s.gap * 0.5);
    let t = Mat::deco(st.trim);
    b.boxes(
        &[
            (
                [d0 - 0.12, 0.0, s.hz - T - 0.02],
                [d0, s.door_h, s.hz + 0.06],
            ),
            (
                [d1, 0.0, s.hz - T - 0.02],
                [d1 + 0.12, s.door_h, s.hz + 0.06],
            ),
            (
                [d0 - 0.12, s.door_h, s.hz - T - 0.02],
                [d1 + 0.12, s.door_h + 0.12, s.hz + 0.06],
            ),
        ],
        t,
    );
}
