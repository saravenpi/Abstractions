//! What the rig reads and what it gives back: plain data, so the rig runs without a game around it.

use bevy::prelude::*;

use super::mode::ViewPrefs;
use super::tune::RADIANS_PER_PIXEL;

/// What the camera follows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Kind {
    /// The body walking, standing or swimming.
    #[default]
    OnFoot,
    /// The body gliding in the wingsuit or flying on its boosters.
    Flying,
    /// The ship, with the player seated in it.
    Ship,
}

/// The thing the camera follows, in render space. The game writes it every frame before the rig runs.
///
/// The rig keeps its state relative to this, so `up` and `forward` must turn with the world:
/// a rebase that moves and turns everything leaves the view as it was.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Target {
    /// Centre of the body capsule, or of the hull of the ship.
    pub position: Vec3,
    /// On foot and flying: the local up. For the ship: the direction of its roof.
    pub up: Vec3,
    /// The direction yaw is measured from. On foot and flying it must not turn with the yaw of
    /// the rig: use the north of the upright frame. For the ship: the direction of its nose.
    pub forward: Vec3,
    /// Velocity relative to the surroundings, in render axes. During a warp, the warp speed.
    pub velocity: Vec3,
    /// Angular velocity in render axes, radians per second. Only the ship needs it.
    pub spin: Vec3,
    /// The local up scaled by how near a surface is: zero in open space, unit length low in an
    /// atmosphere. Only the ship needs it.
    pub ground: Vec3,
    /// Radius of the sphere that holds the whole target.
    pub radius: f32,
    /// What the target is.
    pub kind: Kind,
    /// Roll of a flying body around its direction of flight, positive to the right.
    pub bank: f32,
    /// Direction the back of a flying body faces, in the upright axes of the target, or zero.
    /// When given, the camera rolls its roof toward it as seen along where it looks, a full turn
    /// round, in place of [`Self::bank`].
    pub back: Vec3,
}

impl Default for Target {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            up: Vec3::Y,
            forward: Vec3::NEG_Z,
            velocity: Vec3::ZERO,
            spin: Vec3::ZERO,
            ground: Vec3::ZERO,
            radius: 0.9,
            kind: Kind::OnFoot,
            bank: 0.0,
            back: Vec3::ZERO,
        }
    }
}

/// What the player asks of the camera this frame.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct Look {
    /// Turn to the left, in radians.
    pub yaw_delta: f32,
    /// Turn upward, in radians.
    pub pitch_delta: f32,
    /// Wheel notches, positive toward the body.
    pub zoom_delta: f32,
    /// Whether the player pressed the recentre key: in the ship the view goes back to the nose.
    pub recentre: bool,
}

impl Look {
    /// Returns the look asked by a mouse movement of `motion` pixels and `wheel` notches, with the
    /// mouse settings applied.
    pub fn from_mouse(motion: Vec2, wheel: f32, sensitivity: f32, invert_y: bool) -> Self {
        let scale = RADIANS_PER_PIXEL * sensitivity;
        let y_sign = if invert_y { -1.0 } else { 1.0 };
        Self {
            yaw_delta: -motion.x * scale,
            pitch_delta: -motion.y * scale * y_sign,
            zoom_delta: wheel,
            recentre: false,
        }
    }
}

/// Where the camera is, where it looks and how wide it sees.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct CameraPose {
    /// Position in render space.
    pub position: Vec3,
    /// Orientation in render space. The camera looks along its -Z with +Y up.
    pub rotation: Quat,
    /// Vertical field of view in radians.
    pub fov: f32,
}

/// How much of the body of the player to draw: 0 hides it, 1 draws it fully.
///
/// It is 0 in first person, on foot and in the pilot seat of the ship, and falls to 0 when the
/// third person camera comes close to the head. At 0 the head is hidden and the rest of the
/// body stays: the pilot sees their own arms and legs.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct BodyFade(pub f32);

impl Default for BodyFade {
    fn default() -> Self {
        Self(1.0)
    }
}

/// The look angles the rig holds: on foot they are the heading of the body and the pitch of its eyes.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct Heading {
    /// Yaw around the up of the target, from its forward, positive to the left.
    pub yaw: f32,
    /// Pitch, positive upward.
    pub pitch: f32,
}

impl Shown {
    /// Returns true when the camera and the look angles are all finite numbers.
    pub fn sound(&self) -> bool {
        let pose = &self.pose;
        let angles = Vec3::new(pose.fov, self.heading.yaw, self.heading.pitch);
        pose.position.is_finite() && pose.rotation.is_finite() && angles.is_finite()
    }
}

/// Something that can tell how far a sphere travels before it touches the world.
pub trait Occluder {
    /// Returns how far the centre of a sphere of `radius` travels from `origin` along the unit
    /// `direction` before it touches something, or `None` when the first `max` metres are free.
    /// A sphere that starts inside something travels 0.
    fn cast(&self, origin: Vec3, direction: Vec3, max: f32, radius: f32) -> Option<f32>;
}

impl<F: Fn(Vec3, Vec3, f32, f32) -> Option<f32>> Occluder for F {
    fn cast(&self, origin: Vec3, direction: Vec3, max: f32, radius: f32) -> Option<f32> {
        self(origin, direction, max, radius)
    }
}

/// Everything one step of the rig reads.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Moment {
    /// What the camera follows.
    pub target: Target,
    /// What the player asks.
    pub look: Look,
    /// What the player chose.
    pub prefs: ViewPrefs,
    /// The field of view setting, vertical, in radians.
    pub fov: f32,
    /// Frame time in seconds.
    pub dt: f32,
}

/// Everything one step of the rig gives back.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shown {
    /// The camera.
    pub pose: CameraPose,
    /// How much of the body to draw.
    pub fade: BodyFade,
    /// The look angles after this step.
    pub heading: Heading,
}
