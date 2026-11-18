//! Sites of the legacy plane.

use bevy::math::{DVec3, IVec2, IVec3, Vec2};

use super::Site;
use crate::world::islands::plus;

impl Site for Vec2 {
    type Sample = [f64; 2];

    fn point(self) -> [f64; 2] {
        [self.x as f64, self.y as f64]
    }

    fn cells(self, size: f32) -> impl Iterator<Item = IVec3> {
        let cell = (self / size).floor().as_ivec2();
        (-1..=1).flat_map(move |i| (-1..=1).map(move |j| (cell + IVec2::new(i, j)).extend(0)))
    }

    fn anchor(self, cell: IVec3, size: f32, roll: impl Fn(u32) -> f32) -> Option<Self> {
        Some((cell.truncate().as_vec2() + Vec2::new(roll(0), roll(1))) * size)
    }

    fn offset(self, center: Self) -> Vec2 {
        self - center
    }

    fn moved(self, by: Vec2) -> Self {
        Vec2::new(plus(self.x, by.x), plus(self.y, by.y))
    }

    fn ball(self) -> Option<DVec3> {
        None
    }
}
