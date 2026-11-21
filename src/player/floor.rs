//! The ground a body can be put on: a ray down onto the fixed colliders of the world, the
//! terrain of the detail chunks, rocks and buildings. Where none is loaded there is no ground to
//! stand on yet, whatever height the planet gives: a body put there would fall through it.

use avian3d::prelude::*;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

/// Metres above a point the ray for the ground under it starts: more than the detail ground
/// lies over the coarse ground a ship landed on, so a point inside a slope finds its top.
pub const RISE: f32 = 12.0;

/// Metres under a point the ray for the ground under it reaches.
pub const DEPTH: f32 = 40.0;

/// Casts for the fixed ground under a point.
#[derive(SystemParam)]
pub struct Floor<'w, 's> {
    /// Casts against the collider trees.
    query: SpatialQuery<'w, 's>,
    /// The body every collider belongs to.
    colliders: Query<'w, 's, &'static ColliderOf>,
    /// The kind of every rigid body.
    bodies: Query<'w, 's, &'static RigidBody>,
}

impl Floor<'_, '_> {
    /// Returns true for a collider of a static body: the ground, a rock, a building. The ship,
    /// the bodies and anything else that moves are left out.
    fn fixed(&self, entity: Entity) -> bool {
        let body = self.colliders.get(entity).map(|of| of.body);
        body.is_ok_and(|body| self.bodies.get(body).is_ok_and(RigidBody::is_static))
    }

    /// Returns the render position of the top of the fixed ground under render position `at`,
    /// along the local up `up`, from [`RISE`] above it to [`DEPTH`] under it. Nothing when no
    /// collider is loaded there.
    pub fn under(&self, at: Vec3, up: Vec3) -> Option<Vec3> {
        let down = Dir3::new(-up).ok()?;
        let start = at + up * RISE;
        let fixed = |entity| self.fixed(entity);
        let filter = SpatialQueryFilter::default();
        let hit =
            self.query
                .cast_ray_predicate(start, down, RISE + DEPTH, true, &filter, &fixed)?;
        Some(start + down * hit.distance)
    }

    /// Returns true when a fixed collider lies anywhere `shape`, turned by `turn`, would meet if
    /// swept down the local up `up` from [`RISE`] above render position `at` to [`DEPTH`] under
    /// it: the ground beside a body as well as under its centre.
    pub fn around(&self, (shape, turn): (&Collider, Quat), at: Vec3, up: Vec3) -> bool {
        let Ok(down) = Dir3::new(-up) else {
            return false;
        };
        let fixed = |entity| self.fixed(entity);
        let reach = ShapeCastConfig::from_max_distance(RISE + DEPTH);
        let filter = SpatialQueryFilter::default();
        self.query
            .cast_shape_predicate(shape, at + up * RISE, turn, down, &reach, &filter, &fixed)
            .is_some()
    }
}
