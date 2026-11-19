//! Bodies: parametric rigid parts with sockets, limbs and chains, built from the plan and the planet.
//!
//! Body frame: +Y up, -Z forward (Bevy's forward), +X right, origin on the ground under the
//! centre of the torso. Limbs hang along -Y of their own frame, like the robot's rig. Lengths
//! are metres.

mod build;

pub use build::build;

use bevy::math::{Quat, Vec3};

/// What a part is, for the animation and for later crafting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PartKind {
    /// Main body.
    Torso,
    /// Hip block.
    Pelvis,
    /// A neck segment.
    Neck,
    /// Head.
    Head,
    /// Lower jaw.
    Jaw,
    /// A tail segment.
    Tail,
    /// Upper leg.
    Thigh,
    /// Lower leg.
    Shin,
    /// Foot.
    Foot,
    /// Wing.
    Wing,
    /// Fin.
    Fin,
    /// Bell of a jelly.
    Bell,
    /// A tentacle segment.
    Tentacle,
    /// A body segment of a segmented plan.
    Segment,
    /// Horn or tusk.
    Horn,
    /// Armour plate.
    Plate,
    /// Eye.
    Eye,
    /// Antenna or feeler.
    Antenna,
    /// The body of a slime.
    Blob,
    /// A crystal facet block.
    Crystal,
}

/// The silhouette of a part.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShapeKind {
    /// A box.
    Block,
    /// A prism tapering from its socket to its tip.
    Taper,
    /// A low-poly ellipsoid.
    Ball,
    /// A flat fan, for wings and fins.
    Fan,
    /// A cone.
    Spike,
}

/// One rigid part. Parts are ordered so that a parent always comes before its children.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Part {
    /// What it is.
    pub kind: PartKind,
    /// Index of the parent part, none for the root torso.
    pub parent: Option<u8>,
    /// Joint point in the frame of the parent part.
    pub socket: Vec3,
    /// Rest rotation in the frame of the parent part.
    pub rest: Quat,
    /// Silhouette.
    pub shape: ShapeKind,
    /// Extent: width, length along its axis, depth.
    pub size: Vec3,
    /// Tip size over base size, for tapers.
    pub taper: f32,
    /// Index into the species' tissues.
    pub tissue: u8,
    /// -1 left, 0 centre, 1 right.
    pub side: i8,
}

/// Which way a knee bends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Knee {
    /// The joint points forward, like a mammal knee.
    Forward,
    /// Backward, like a bird's ankle.
    Back,
    /// Up and out, like an insect.
    Out,
}

/// One leg.
#[derive(Debug, Clone, PartialEq)]
pub struct Leg {
    /// Part indices from hip to foot: two or three bones.
    pub bones: Vec<u8>,
    /// Bone lengths, in the order of `bones`.
    pub lengths: Vec<f32>,
    /// Where the foot rests on flat ground, in the body frame.
    pub rest: Vec3,
    /// Which way the knee bends.
    pub knee: Knee,
    /// -1 left, 1 right.
    pub side: i8,
    /// Order from front to back, 0 for the front pair.
    pub rank: u8,
}

/// A chain of parts that follows its root: neck, spine, tail or tentacle.
#[derive(Debug, Clone, PartialEq)]
pub struct Chain {
    /// Part indices from root to tip.
    pub parts: Vec<u8>,
    /// Distance between successive joints.
    pub lengths: Vec<f32>,
    /// Largest bend per joint in radians.
    pub limit: f32,
}

/// A pair of wings.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Wings {
    /// Left and right wing parts.
    pub parts: [u8; 2],
    /// Span tip to tip in metres.
    pub span: f32,
    /// Flaps per second.
    pub rate: f32,
}

/// How a body moves, which picks the motion model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Locomotion {
    /// Legs on a gait.
    Walk,
    /// Lateral undulation of a legless chain.
    Slither,
    /// A bell that pulses.
    Pulse,
    /// A slime that flows.
    Ooze,
    /// A flock of members.
    Swarm,
    /// Rigid hops and rolls.
    Tumble,
    /// Wings.
    Fly,
    /// A body wave through water.
    Swim,
}

/// A feature behaviour reads now and attacks will read later.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FeatureKind {
    /// Horns or tusks.
    Horn,
    /// A tail that can sweep.
    Tail,
    /// A long neck that can lunge.
    Neck,
    /// Jaws.
    Jaw,
    /// Armour plates.
    Plate,
    /// A gland.
    Gland,
    /// Spines.
    Spike,
}

/// A listed feature with its size and tissue.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Feature {
    /// Which feature.
    pub kind: FeatureKind,
    /// Size in metres.
    pub size: f32,
    /// Index into the species' tissues.
    pub tissue: u8,
}

/// A whole body.
#[derive(Debug, Clone, PartialEq)]
pub struct Body {
    /// Every part, parents first.
    pub parts: Vec<Part>,
    /// Legs, front pair first, left before right.
    pub legs: Vec<Leg>,
    /// Spine chain for segmented and slithering bodies.
    pub spine: Option<Chain>,
    /// Neck chain.
    pub neck: Option<Chain>,
    /// Tail chain.
    pub tail: Option<Chain>,
    /// Tentacle chains.
    pub tentacles: Vec<Chain>,
    /// Wings.
    pub wings: Option<Wings>,
    /// Motion model.
    pub locomotion: Locomotion,
    /// Nose to tail length in metres.
    pub length: f32,
    /// Hip height over the ground at rest, in metres.
    pub height: f32,
    /// Listed features.
    pub features: Vec<Feature>,
}
