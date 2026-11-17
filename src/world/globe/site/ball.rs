//! Sites of the sea level sphere.

use bevy::math::{DVec3, IVec3, Vec2, Vec3};

use super::Site;

/// Slot of the number that places a site along the third axis of its cell.
const SLOT_Z: u32 = 7;

impl Site for DVec3 {
    type Sample = [f64; 3];

    fn point(self) -> [f64; 3] {
        self.to_array()
    }

    fn cells(self, size: f32) -> impl Iterator<Item = IVec3> {
        let cell = (self / f64::from(size)).floor().as_ivec3();
        (-1..=1).flat_map(move |i| {
            (-1..=1).flat_map(move |j| (-1..=1).map(move |k| cell + IVec3::new(i, j, k)))
        })
    }

    fn anchor(self, cell: IVec3, size: f32, roll: impl Fn(u32) -> f32) -> Option<Self> {
        let size = f64::from(size);
        let jitter = Vec3::new(roll(0), roll(1), roll(SLOT_Z)).as_dvec3();
        let raw = (cell.as_dvec3() + jitter) * size;
        let (radius, reach) = (self.length(), raw.length());
        ((reach - radius).abs() <= size * 0.5).then(|| raw * (radius / reach))
    }

    fn offset(self, center: Self) -> Vec2 {
        let (east, south) = center.normalize().any_orthonormal_pair();
        let d = self - center;
        Vec2::new(d.dot(east) as f32, d.dot(south) as f32)
    }

    fn moved(self, by: Vec2) -> Self {
        let (east, south) = self.normalize().any_orthonormal_pair();
        let step = east * f64::from(by.x) + south * f64::from(by.y);
        (self + step).normalize() * self.length()
    }
}
