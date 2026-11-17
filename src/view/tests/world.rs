//! A fake world of boxes the camera sphere is cast against.

use bevy::prelude::*;

use crate::view::Occluder;

/// Axis-aligned boxes, each from its lowest corner to its highest.
#[derive(Debug, Default, Clone)]
pub(super) struct Boxes(pub Vec<(Vec3, Vec3)>);

/// Returns how far a point travels from `origin` along `dir` before it enters the box, within `reach`.
fn enter((low, high): (Vec3, Vec3), origin: Vec3, dir: Vec3, reach: f32) -> Option<f32> {
    let (mut near, mut far) = (0.0_f32, reach);
    for axis in 0..3 {
        let (from, step) = (origin[axis], dir[axis]);
        if step.abs() < 1.0e-9 {
            if from <= low[axis] || from >= high[axis] {
                return None;
            }
            continue;
        }
        let (a, b) = ((low[axis] - from) / step, (high[axis] - from) / step);
        near = near.max(a.min(b));
        far = far.min(a.max(b));
    }
    (near <= far).then_some(near)
}

impl Occluder for Boxes {
    fn cast(&self, origin: Vec3, direction: Vec3, max: f32, radius: f32) -> Option<f32> {
        let grown =
            |(low, high): &(Vec3, Vec3)| (*low - Vec3::splat(radius), *high + Vec3::splat(radius));
        self.0
            .iter()
            .filter_map(|solid| enter(grown(solid), origin, direction, max))
            .min_by(f32::total_cmp)
    }
}

impl Boxes {
    /// Returns a world with only a floor, whose top is at height 0.
    pub(super) fn floor() -> Self {
        Self(vec![(
            Vec3::new(-90.0, -1.0, -90.0),
            Vec3::new(90.0, 0.0, 90.0),
        )])
    }

    /// Returns the world with one more box.
    pub(super) fn with(mut self, low: [f32; 3], high: [f32; 3]) -> Self {
        self.0.push((Vec3::from(low), Vec3::from(high)));
        self
    }

    /// Returns the distance from `point` to the nearest box, 0 inside one.
    pub(super) fn clearance(&self, point: Vec3) -> f32 {
        let gap = |(low, high): &(Vec3, Vec3)| (point.clamp(*low, *high) - point).length();
        self.0.iter().map(gap).fold(f32::MAX, f32::min)
    }

    /// Returns a building on the floor, entered by walking toward -Z: a corridor 1.6 m wide
    /// from z 0 to -10, a doorway 1 m wide and 2.1 m high at z -10, then a room with a
    /// ceiling at 2.2 m.
    pub(super) fn building() -> Self {
        Self::floor()
            .with([0.8, 0.0, -10.0], [1.0, 3.0, 0.0])
            .with([-1.0, 0.0, -10.0], [-0.8, 3.0, 0.0])
            .with([-5.0, 0.0, -10.2], [-0.5, 3.0, -10.0])
            .with([0.5, 0.0, -10.2], [5.0, 3.0, -10.0])
            .with([-0.5, 2.1, -10.2], [0.5, 3.0, -10.0])
            .with([-3.0, 2.2, -18.0], [3.0, 2.4, -10.2])
            .with([-3.2, 0.0, -18.0], [-3.0, 3.0, -10.2])
            .with([3.0, 0.0, -18.0], [3.2, 3.0, -10.2])
            .with([-3.2, 0.0, -18.2], [3.2, 3.0, -18.0])
    }
}

/// A world seen after a rebase: every query is turned and moved back into the world it wraps.
pub(super) struct Rebased<'a> {
    /// The world before the rebase.
    pub inner: &'a Boxes,
    /// Rotation of the rebase.
    pub turn: Quat,
    /// Translation of the rebase.
    pub shift: Vec3,
}

impl Occluder for Rebased<'_> {
    fn cast(&self, origin: Vec3, direction: Vec3, max: f32, radius: f32) -> Option<f32> {
        let back = self.turn.inverse();
        self.inner
            .cast(back * (origin - self.shift), back * direction, max, radius)
    }
}
