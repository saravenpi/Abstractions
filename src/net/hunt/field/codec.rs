//! How the sound of a blow travels: the layer that rang and the head that struck are a byte each, and
//! the point is metres in the turning frame of the body.

use abstractions_net::wire::{self, CLEAN, DEFLECTED};
use bevy::math::DVec3;

use super::tune::{MASS_BIAS, STIFFNESS_BIAS};
use crate::audio::{Body, Impact, Outer};
use crate::space::{Anchor, Frame};
use crate::universe::BodyId;

/// The outer layers, in the order of their numbers.
const OUTERS: [Outer; 8] = [
    Outer::Flesh,
    Outer::Hide,
    Outer::Shell,
    Outer::Plate,
    Outer::Bone,
    Outer::Horn,
    Outer::RobotPlate,
    Outer::Ground,
];

/// Returns the number of an outer layer on the network.
pub fn outer_code(outer: Outer) -> u8 {
    OUTERS.iter().position(|o| *o == outer).unwrap_or(0) as u8
}

/// Returns the layer numbered `code`, flesh for a number it has none for.
pub fn outer_of(code: u8) -> Outer {
    OUTERS
        .get(usize::from(code))
        .copied()
        .unwrap_or(Outer::Flesh)
}

/// Returns the stiffness in gigapascals and the mass in kilograms that ring in what `outer` shows,
/// where the network does not carry them.
pub fn struck_of(outer: Outer) -> Body {
    let (stiffness, mass) = match outer {
        Outer::Flesh => (0.001, 20.0),
        Outer::Hide => (0.1, 20.0),
        Outer::Shell => (5.0, 5.0),
        Outer::Plate => (5.0, 20.0),
        Outer::Bone => (18.0, 10.0),
        Outer::Horn => (4.0, 3.0),
        Outer::RobotPlate => (70.0, 80.0),
        Outer::Ground => (30.0, 500.0),
    };
    Body { stiffness, mass }
}

/// Returns the byte of a tool head: its stiffness in the high four bits and its mass in the low
/// four, each as a power of two.
pub fn tool_code(tool: &Body) -> u8 {
    let code =
        |value: f32, bias: f32| (value.max(1e-3).log2().round() + bias).clamp(0.0, 15.0) as u8;
    code(tool.stiffness, STIFFNESS_BIAS) << 4 | code(tool.mass, MASS_BIAS)
}

/// Returns the tool head a byte stands for.
pub fn tool_of(code: u8) -> Body {
    let value = |code: u8, bias: f32| 2f32.powf(f32::from(code) - bias);
    Body {
        stiffness: value(code >> 4, STIFFNESS_BIAS),
        mass: value(code & 15, MASS_BIAS),
    }
}

/// Returns `impact` as the network carries it, or nothing when the anchor is not on `body`.
pub fn to_wire(impact: &Impact, body: BodyId, anchor: &Anchor) -> Option<wire::Impact> {
    (anchor.frame == Frame::Surface(body)).then(|| wire::Impact {
        body,
        at: anchor.to_frame(impact.at).to_array(),
        joules: impact.joules,
        cut: impact.cut,
        outer: outer_code(impact.outer),
        tool: tool_code(&impact.tool),
        result: if impact.deflected { DEFLECTED } else { CLEAN },
    })
}

/// Returns the impact `told` as the audio plays it, or nothing when it is not on `body` or the
/// anchor is not.
pub fn from_wire(told: &wire::Impact, body: BodyId, anchor: &Anchor) -> Option<Impact> {
    let here = told.body == body && anchor.frame == Frame::Surface(body);
    let outer = outer_of(told.outer);
    here.then(|| Impact {
        at: anchor.to_render(DVec3::from_array(told.at)),
        joules: told.joules,
        cut: told.cut,
        outer,
        struck: struck_of(outer),
        tool: tool_of(told.tool),
        deflected: told.result == DEFLECTED,
    })
}
