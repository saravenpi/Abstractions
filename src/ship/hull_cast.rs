//! The built solids the ship meets: houses, roofs, stalls and the base, as the static colliders
//! the world streams near the player. The hull shape is swept against them and rays find the
//! decks under the contact points, in the render space the collider trees were last built in.
//! The terrain keeps its own height: its colliders are left out.

use avian3d::prelude::*;
use bevy::ecs::system::SystemParam;
use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::gear::GEAR;
use super::hull::{Hit, SKIN};
use super::mesh::hulls;
use crate::space::{Anchor, Frame};

/// Metres above a contact point a deck ray starts: more than a fast ship sinks in one step.
const LIFT: f64 = 1.0;

/// Metres under a contact point a deck ray looks, so that the radar reads a roof under the ship.
const DEPTH: f64 = 30.0;

/// Height in metres of the belly, the bottom of the heat shield, which the struts of the locked
/// gear stand under.
const BELLY: f32 = -0.65;

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
    /// Returns the hull shapes made of the convex parts of the drawn airframe, see
    /// [`hulls`], and of the legs of [`GEAR`].
    fn default() -> Self {
        let part = |points: Vec<Vec3>| {
            let hull = Collider::convex_hull(&points).expect("the airframe parts are solid");
            (Vec3::ZERO, Quat::IDENTITY, hull)
        };
        let low = GEAR[0].y as f32 + SPRING;
        let leg = |foot: DVec3| {
            let at = Vec3::new(foot.x as f32, (low + BELLY) * 0.5, foot.z as f32);
            let strut = Collider::cuboid(STRUT_WIDTH, BELLY - low, STRUT_WIDTH);
            (at, Quat::IDENTITY, strut)
        };
        let folded: Vec<_> = hulls().into_iter().map(part).collect();
        let down = folded.iter().cloned().chain(GEAR.map(leg)).collect();
        Self([Collider::compound(folded), Collider::compound(down)])
    }
}

/// Marks remote player ship hulls so local hull casts collide with them.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct OtherHull;

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
    /// Remote ship entities included alongside static solids.
    others: Query<'w, 's, (), With<OtherHull>>,
    /// The hull shape.
    shape: Local<'s, Shape>,
}

impl Solids<'_, '_> {
    /// Returns true for a collider the hull meets: a static solid or another player's hull that
    /// is not the terrain trimesh.
    fn keeps(&self, entity: Entity) -> bool {
        let Ok((collider, of)) = self.colliders.get(entity) else {
            return false;
        };
        let fixed = self.bodies.get(of.body).is_ok_and(RigidBody::is_static);
        (fixed || self.others.contains(of.body)) && collider.shape().as_trimesh().is_none()
    }

    /// Returns the anchor to cast with for positions in `frame`, when the trees are in it.
    fn view(&self, frame: Frame) -> Option<Anchor> {
        self.built
            .0
            .filter(|anchor| anchor.frame == frame && !anchor.flat)
    }

    /// Returns the height in metres above a sea level `radius` metres from the planet centre of
    /// the top of a built solid under `pos` in the surface frame `frame`, from [`LIFT`] above it
    /// to [`DEPTH`] under it. Without one it is minus infinity.
    pub fn top(&self, frame: Frame, pos: DVec3, radius: f64) -> f64 {
        let (Some(anchor), Some(up)) = (self.view(frame), pos.try_normalize()) else {
            return f64::NEG_INFINITY;
        };
        let Ok(down) = Dir3::new((anchor.rotation * -up).as_vec3()) else {
            return f64::NEG_INFINITY;
        };
        let (start, reach) = (anchor.to_render(pos + up * LIFT), (LIFT + DEPTH) as f32);
        let filter = SpatialQueryFilter::default();
        let keeps = |entity| self.keeps(entity);
        let hit = self
            .query
            .cast_ray_predicate(start, down, reach, true, &filter, &keeps);
        hit.map_or(f64::NEG_INFINITY, |hit| {
            (pos + up * (LIFT - f64::from(hit.distance))).length() - radius
        })
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
