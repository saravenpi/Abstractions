//! What the game tells the audio about a blow: where, how hard, what it struck and with what.

use bevy::prelude::*;

/// The outer layer of what was struck.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Outer {
    /// Bare flesh, a wound or a soft body.
    Flesh,
    /// Hide.
    Hide,
    /// The shell of a segmented body.
    Shell,
    /// A plate or scute.
    Plate,
    /// Bone.
    Bone,
    /// Horn, antler or a tusk.
    Horn,
    /// The plate of the robot.
    RobotPlate,
    /// Soil or rock.
    Ground,
}

/// A body in a blow, as the sound needs it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Body {
    /// Young's modulus of its material, in gigapascals.
    pub stiffness: f32,
    /// Mass that rings, in kilograms: the part struck, or the head of the tool.
    pub mass: f32,
}

/// A blow that landed. Send it where the blow is resolved and the audio plays it from the place.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct Impact {
    /// The contact point, in render space.
    pub at: Vec3,
    /// The energy, in joules.
    pub joules: f32,
    /// True when the blow opened what it struck: the sound then has a short wet noise.
    pub cut: bool,
    /// The outer layer that was struck.
    pub outer: Outer,
    /// What was struck.
    pub struck: Body,
    /// The head of the tool, or the part of the animal, that struck.
    pub tool: Body,
    /// True when the blow glanced off along the surface: a short high tick instead of a thud.
    pub deflected: bool,
}

/// A tool swung through the air.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct Whoosh {
    /// Where the tip is, in render space.
    pub at: Vec3,
    /// Speed of the tip in metres per second.
    pub speed: f32,
    /// Moment of inertia of the tool about the hand, in kg·m².
    pub inertia: f32,
}
