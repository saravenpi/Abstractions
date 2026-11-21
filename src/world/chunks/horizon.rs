//! What of the coarse terrain the eye cannot see: nodes beneath the curve of the planet, and
//! nodes too far for any shadow.
//!
//! The view reaches as far as the air lets it, about 30 km on a clear day, but from near the
//! ground most of that disc lies behind the curve of the planet. A node is hidden when even its
//! highest point is below the horizon of the eye over a sphere that every ground and every sea
//! stands on or above. A node farther than any shadow reaches casts and receives none, which its
//! fades respect through [`Shadowless`].

use bevy::ecs::system::SystemParam;
use bevy::light::{NotShadowCaster, NotShadowReceiver};
use bevy::math::DVec3;
use bevy::prelude::*;

use super::Chunks;
use super::dither::{Shadowless, Way};
use super::nodes::Node;
use super::scene::Scene;
use crate::render::GameCamera;
use crate::world::Terrain;
use crate::world::focus::Frames;
use crate::world::globe::ChunkId;
use crate::world::mesh::Frame;

/// Metres the eye is taken higher than the game camera: its pose is the one of the frame before,
/// and a higher eye sees farther, so the horizon is never drawn too near.
const LIFT: f64 = 5.0;

/// Metres from the focus beyond which a node casts and receives no shadow: past the 220 m of the
/// farthest shadow setting, with room for the camera.
pub const CAST_REACH: f64 = 300.0;

/// Where a coarse node lies on its planet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds {
    /// Unit direction of its middle from the planet centre.
    pub dir: DVec3,
    /// Angle in radians from its middle to its farthest corner.
    pub angle: f64,
    /// Distance in metres from the planet centre of its lowest and its highest vertex.
    pub radii: (f64, f64),
}

impl Bounds {
    /// Bounds that hold the whole planet: never below the horizon, never far.
    pub const WHOLE: Bounds = Bounds {
        dir: DVec3::Y,
        angle: std::f64::consts::PI,
        radii: (0.0, f64::INFINITY),
    };

    /// Returns the bounds of node `id` cut into `cells` per side, whose `mesh` is stored in
    /// `frame`, from its vertices, skirts included, and from its four corners. Its lowest point
    /// is taken lower by how far a cell of the node sags under the sphere of its corners.
    pub fn of((id, cells): (ChunkId, usize), mesh: &Mesh, frame: &Frame) -> Self {
        let dir = id.dir(0.5, 0.5).normalize();
        let corners = [(0.0, 0.0), (1.0, 0.0), (0.0, 1.0), (1.0, 1.0)];
        let angle = corners
            .map(|(s, t)| dir.angle_between(id.dir(s, t).normalize()))
            .into_iter()
            .fold(0.0, f64::max);
        let positions = mesh.attribute(Mesh::ATTRIBUTE_POSITION);
        let points = positions
            .and_then(|values| values.as_float3())
            .unwrap_or(&[]);
        let radius = |p: &[f32; 3]| (frame.origin + frame.rot * Vec3::from(*p).as_dvec3()).length();
        let (low, high) = points
            .iter()
            .map(radius)
            .fold((f64::INFINITY, 0.0), |(low, high), r| {
                (low.min(r), high.max(r))
            });
        let cell = 2.0 * angle / cells.max(1) as f64;
        match points.is_empty() {
            true => Self::WHOLE,
            false => Self {
                dir,
                angle,
                radii: (low * cell.cos(), high),
            },
        }
    }

    /// Returns true when every point of the node lies below the horizon of `eye`.
    pub fn hidden(&self, eye: &Eye) -> bool {
        if eye.height <= eye.floor || !self.radii.1.is_finite() {
            return false;
        }
        let over = |radius: f64| (eye.floor / radius.max(eye.floor)).acos();
        let apart = eye.dir.angle_between(self.dir) - self.angle;
        apart > over(eye.height) + over(self.radii.1)
    }

    /// Returns the least distance in metres from the planet position `at` to any point of the node.
    pub fn nearest(&self, at: DVec3) -> f64 {
        let (low, high) = self.radii;
        let gap = (at.normalize_or(DVec3::Y).angle_between(self.dir) - self.angle).max(0.0);
        let along = (at.length() * gap.cos()).clamp(low.min(high), high);
        (at.length().powi(2) + along * along - 2.0 * at.length() * along * gap.cos())
            .max(0.0)
            .sqrt()
    }
}

/// The eye that the horizon is drawn for.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Eye {
    /// Unit direction of the eye from the planet centre.
    pub dir: DVec3,
    /// Distance in metres of the eye from the planet centre.
    pub height: f64,
    /// Radius in metres of the sphere that hides what lies beyond its horizon: no ground and no
    /// sea is lower.
    pub floor: f64,
}

/// Returns the radius under every ground and every sea that `chunks` draws: the lowest point of
/// every node on the draw cut, and the sea level on a planet with a sea, whose flat sea every
/// node draws. The hidden parents do not count: they draw nothing.
fn floor(chunks: &Chunks, terrain: &Terrain) -> f64 {
    let globe = terrain.globe();
    let drawn = chunks.nodes.live.values().filter(|node| node.visible);
    let lowest = drawn.map(|node| node.bounds.radii.0);
    let low = lowest.fold(f64::INFINITY, f64::min);
    match globe.has_sea() {
        true => low.max(globe.radius() + f64::from(globe.sea_level())),
        false => low,
    }
}

/// Hides `node` when it is `beyond` the horizon and shows it again when it comes back over it.
fn hide(commands: &mut Commands, node: &mut Node, beyond: bool) {
    if node.beyond != beyond {
        node.beyond = beyond;
        node.show(commands);
    }
}

/// Makes `node` cast and receive no shadow when it is `far`, and gives both back when it comes
/// near, leaving a node that fades out without shadow until its fade ends.
fn shade(commands: &mut Commands, node: &mut Node, far: bool, fading: Option<Way>) {
    if node.shadowless == far {
        return;
    }
    node.shadowless = far;
    let mut entity = commands.entity(node.entity);
    match far {
        true => entity.try_insert((Shadowless, NotShadowCaster, NotShadowReceiver)),
        false => entity.try_remove::<(Shadowless, NotShadowReceiver)>(),
    };
    if !far && !matches!(fading, Some(Way::Hide | Way::Leave)) {
        entity.try_remove::<NotShadowCaster>();
    }
}

/// What the horizon is drawn from: the focus, the game camera and the planet under them.
#[derive(SystemParam)]
pub struct Seer<'w, 's> {
    /// The focus of the streamer.
    scene: Scene<'w, 's>,
    /// The streamed planet.
    terrain: Option<Res<'w, Terrain>>,
    /// The anchor and the planet frame.
    frames: Frames<'w>,
    /// The game camera.
    camera: Query<'w, 's, &'static GlobalTransform, With<GameCamera>>,
}

impl Seer<'_, '_> {
    /// Returns the focus and the eye, the game camera or the focus without one, as planet
    /// positions.
    fn places(&self) -> Option<(DVec3, DVec3)> {
        let at = self.scene.at()?;
        let (anchor, link) = self.frames.link()?;
        let camera = self.camera.iter().next().map(|g| g.translation());
        let eye = camera.map_or(at, |c| link.to_planet(anchor.to_frame(c)));
        Some((at, eye))
    }
}

/// Hides the coarse nodes below the horizon of the eye and takes the shadows off the ones far
/// from the focus, whenever the eye moved a metre or the nodes changed.
pub fn horizon(
    mut commands: Commands,
    mut chunks: ResMut<Chunks>,
    seer: Seer,
    mut last: Local<Option<(IVec3, u64, usize)>>,
) {
    let (Some((at, seen)), Some(terrain)) = (seer.places(), seer.terrain.as_deref()) else {
        return;
    };
    let key = (
        seen.round().as_ivec3(),
        chunks.stamp,
        chunks.nodes.live.len(),
    );
    if *last == Some(key) {
        return;
    }
    *last = Some(key);
    let eye = Eye {
        dir: seen.normalize_or(DVec3::Y),
        height: seen.length() + LIFT,
        floor: floor(&chunks, terrain),
    };
    let chunks = &mut *chunks;
    for node in chunks.nodes.live.values_mut() {
        let beyond = node.bounds.hidden(&eye);
        let far = node.bounds.nearest(at) > CAST_REACH;
        let fading = chunks.fades.way(node.entity);
        hide(&mut commands, node, beyond);
        shade(&mut commands, node, far, fading);
    }
}
