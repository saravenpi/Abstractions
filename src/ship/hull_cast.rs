//! The built solids the ship meets: houses, roofs, stalls and the base, as the static colliders
//! the world streams near the player. The hull shape is swept against them, in the render
//! space the collider trees were last built in.
//! The terrain keeps its own height: its colliders are left out.

use avian3d::prelude::*;
use bevy::ecs::system::SystemParam;
use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::gear::GEAR;
use super::hull::{Hit, SKIN};
use crate::space::{Anchor, Frame};

/// The parts of the hull shape in body axes: centre, then full width, height and length. The
/// fuselage from the nose tip to the tail and from the belly to the roof, then the wings with
/// their winglets, as the contact points of the hull and the drawn airframe span them.
const PARTS: [(Vec3, Vec3); 2] = [
    (Vec3::new(0.0, 0.3, -0.45), Vec3::new(2.0, 1.9, 7.5)),
    (Vec3::new(0.0, -0.1, 1.7), Vec3::new(6.4, 0.5, 1.8)),
];

/// Metres of a locked leg above its foot that the hull shape leaves out: the foot itself and
/// the travel of its spring stay with the deck rays and the landing rules.
const SPRING: f32 = 0.35;

/// Width in metres of the struts of the locked gear in the hull shape.
const STRUT_WIDTH: f32 = 0.25;

/// The anchor in force when the physics last built its collider trees, which place every
/// collider: casts made against them must use it, not an anchor that moved since.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct Built(pub Option<Anchor>);

/// Keeps [`Built`] on the anchor the physics step just placed the colliders with.
fn record(anchor: Option<Res<Anchor>>, mut built: ResMut<Built>) {
    built.0 = anchor.map(|anchor| *anchor);
}

/// The hull shapes the sweeps cast, made once: with the gear folded, and with the struts of the
/// locked gear down to [`SPRING`] above its feet.
pub struct Shape([Collider; 2]);

impl Default for Shape {
    /// Returns the hull shapes made of [`PARTS`] and of the legs of [`GEAR`].
    fn default() -> Self {
        let part = |(at, size): (Vec3, Vec3)| {
            (at, Quat::IDENTITY, Collider::cuboid(size.x, size.y, size.z))
        };
        let low = GEAR[0].y as f32 + SPRING;
        let leg = |foot: DVec3| {
            let top = PARTS[0].0.y - PARTS[0].1.y * 0.5;
            let at = Vec3::new(foot.x as f32, (low + top) * 0.5, foot.z as f32);
            (at, Vec3::new(STRUT_WIDTH, top - low, STRUT_WIDTH))
        };
        let folded = PARTS.map(part).to_vec();
        let down = PARTS.into_iter().chain(GEAR.map(leg)).map(part).collect();
        Self([Collider::compound(folded), Collider::compound(down)])
    }
}

/// The spatial queries of the hull against the built solids.
#[derive(SystemParam)]
pub struct Solids<'w, 's> {
    /// Casts against the collider trees.
    query: SpatialQuery<'w, 's>,
    /// The anchor the trees were built with.
    built: Res<'w, Built>,
    /// Every collider and the body it belongs to.
    colliders: Query<'w, 's, (&'static Collider, &'static ColliderOf)>,
    /// The kind of every rigid body.
    bodies: Query<'w, 's, &'static RigidBody>,
    /// The hull shape.
    shape: Local<'s, Shape>,
}

impl Solids<'_, '_> {
    /// Returns true for a collider the hull meets: a static one that is not the terrain mesh.
    /// The ship, the player and anything else that moves are left out.
    fn keeps(&self, entity: Entity) -> bool {
        let Ok((collider, of)) = self.colliders.get(entity) else {
            return false;
        };
        let fixed = self.bodies.get(of.body).is_ok_and(RigidBody::is_static);
        fixed && collider.shape().as_trimesh().is_none()
    }

    /// Returns the anchor to cast with for positions in `frame`, when the trees are in it.
    fn view(&self, frame: Frame) -> Option<Anchor> {
        self.built
            .0
            .filter(|anchor| anchor.frame == frame && !anchor.flat)
    }

    /// Returns where the solid `entity` bears on a hull centred at `centre`, in render space:
    /// the point of the solid nearest that centre, so that a face met flat pushes through the
    /// centre and an edge under one end of the hull pushes at that end. The witness `touch` of
    /// the sweep stands in when the centre is inside the solid.
    fn bearing(&self, entity: Entity, centre: Vec3, touch: Vec3) -> Vec3 {
        let filter = SpatialQueryFilter::default();
        let only = |other| other == entity;
        let near = self
            .query
            .project_point_predicate(centre, true, &filter, &only);
        near.filter(|near| !near.is_inside)
            .map_or(touch, |near| near.point)
    }

    /// Sweeps the hull of a ship in `frame`, at `at` turned to `rot`, along `motion`, with its
    /// struts when the gear is `down`, and returns what it meets within that move and a
    /// [`SKIN`] beyond, in frame axes.
    pub fn cast(
        &self,
        frame: Frame,
        (at, rot): (DVec3, DQuat),
        motion: DVec3,
        down: bool,
    ) -> Option<Hit> {
        let anchor = self.view(frame)?;
        let (way, length) = Dir3::new_and_length((anchor.rotation * motion).as_vec3()).ok()?;
        let config = ShapeCastConfig {
            ignore_origin_penetration: true,
            ..ShapeCastConfig::from_max_distance(length + SKIN as f32)
        };
        let (start, turn) = (anchor.to_render(at), anchor.rotation_to_render(rot));
        let filter = SpatialQueryFilter::default();
        let keeps = |entity| self.keeps(entity);
        let hit = self.query.cast_shape_predicate(
            &self.shape.0[usize::from(down)],
            start,
            turn,
            way,
            &config,
            &filter,
            &keeps,
        )?;
        let depth = (hit.point1 - hit.point2).dot(hit.normal1).max(0.0);
        let centre = start + way * hit.distance;
        Some(Hit {
            distance: f64::from(hit.distance),
            normal: (anchor.rotation.inverse() * hit.normal1.as_dvec3()).normalize(),
            point: anchor.to_frame(self.bearing(hit.entity, centre, hit.point1)),
            depth: f64::from(depth),
        })
    }
}

/// Registers [`Built`] and keeps it on the anchor of every physics step.
pub fn plugin(app: &mut App) {
    app.init_resource::<Built>()
        .add_systems(FixedPostUpdate, record.after(PhysicsSystems::Last));
}
