//! Explanations: why a build is good or bad, in plain words.

use std::fmt;

use super::Build;
use super::hit::{Layer, Strike};
use super::swing::Swing;
use super::wear::{Joint, Wear};
use crate::craft::matter::Class;

/// One reason a build works or fails.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Why {
    /// The edge or point is no harder than this layer of the target, so it only bruises it.
    EdgeTooSoft(Class),
    /// This joint gives first.
    WeakestJoint(Joint),
    /// Holding it level takes more torque than the grip has on this world.
    TooHeavy,
    /// The head or shaft has a fracture toughness under 1 MPa·m½ and shatters.
    TooBrittle,
    /// Nothing carried is hard enough for an edge or a point.
    NoEdgeMaterial,
}

impl fmt::Display for Why {
    /// Writes the reason as the player reads it.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Why::EdgeTooSoft(class) => write!(f, "edge too soft for {}", class.name()),
            Why::WeakestJoint(joint) => write!(
                f,
                "weakest joint: {}",
                match joint {
                    Joint::Head => "head binding",
                    Joint::Counterweight => "counterweight binding",
                    Joint::Edge => "the edge",
                    Joint::Shaft => "the shaft",
                }
            ),
            Why::TooHeavy => f.write_str("too heavy to swing on this world"),
            Why::TooBrittle => f.write_str("too brittle, it will shatter"),
            Why::NoEdgeMaterial => f.write_str("nothing carried holds an edge"),
        }
    }
}

/// Returns the reasons that explain `build`: the layer its edge was too soft for, whether it is too
/// heavy or too brittle, and always the joint that gives first, given its `swing`, its `strike` on
/// `layers` and the `lives` of its joints.
pub fn explain(
    build: &Build,
    swing: &Swing,
    strike: &Strike,
    layers: &[Layer],
    lives: &[f32; 4],
) -> Vec<Why> {
    let mut out = Vec::with_capacity(4);
    if let Some(layer) = strike.soft.and_then(|i| layers.get(i as usize)) {
        out.push(Why::EdgeTooSoft(layer.class));
    }
    if swing.torque > build.hands.budget() {
        out.push(Why::TooHeavy);
    }
    if build.head.material.brittle() || build.shaft.material.brittle() {
        out.push(Why::TooBrittle);
    }
    out.push(Why::WeakestJoint(Wear::default().weakest(lives)));
    out
}
