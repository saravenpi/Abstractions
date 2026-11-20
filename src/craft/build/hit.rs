//! Hits: a blow against a target's layers, tested layer by layer as Dwarf Fortress does.
//!
//! An edge or a point cuts a layer only when it is harder than the layer and carries the work the cut
//! needs; when it fails, the blow turns blunt for that layer and every one after. A blunt blow breaks
//! a stiff layer when it carries the strain energy the layer stores at breaking; a soft layer only
//! bruises, and the blow stops there. Each layer passed costs its work and about 5 % of the momentum
//! (https://dwarffortresswiki.org/index.php/Material_science).

use super::tune::{EDGE_ANGLE, KEPT, POINT_ANGLE, SOFT};
use crate::craft::matter::{Class, Material};

/// One layer of a target, outermost first.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Layer {
    /// What it is.
    pub class: Class,
    /// Its numbers.
    pub material: Material,
    /// Thickness in metres.
    pub thickness: f32,
}

/// How a blow meets a layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mode {
    /// A long edge drawn through: a wide wedge.
    Slash,
    /// A small point pushed in: a narrow wedge.
    Stab,
    /// A face that crushes.
    Blunt,
}

/// A blow as it arrives.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Blow {
    /// Kinetic energy in joules.
    pub energy: f32,
    /// How it meets the target.
    pub mode: Mode,
    /// Width of edge or point that meets the target, in metres.
    pub width: f32,
    /// Area of the face that meets the target if the blow turns blunt, in square metres.
    pub area: f32,
    /// Mohs hardness of what meets the target.
    pub hardness: f32,
}

/// What a blow did to a target.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Strike {
    /// Layers cut or broken through.
    pub passed: u8,
    /// Share of the work of the layer that stopped the blow it did carry, 0 to 1.
    pub part: f32,
    /// The first layer the edge was too soft for, if any.
    pub soft: Option<u8>,
    /// Energy left after the last layer, in joules.
    pub left: f32,
}

impl Strike {
    /// Returns how deep the blow went: whole layers passed plus the share of the next.
    pub fn score(&self) -> f32 {
        self.passed as f32 + self.part
    }
}

/// Returns the work in joules a wedge of `mode` needs to cut through `layer` over `width` metres:
/// width·t·(G + σ·t·tan(θ/2)), new crack surface at the work of fracture plus pushing the wedge in.
fn cut_work(mode: Mode, width: f32, layer: &Layer) -> f32 {
    let angle = if mode == Mode::Stab {
        POINT_ANGLE
    } else {
        EDGE_ANGLE
    };
    let t = layer.thickness;
    let wedge = layer.material.strength * 1.0e6 * t * (angle / 2.0).tan();
    width * t * (layer.material.work() + wedge)
}

/// Returns the work in joules a blunt face of `area` needs to break through `layer`: the strain
/// energy σ²/2E of the layer under the face. A soft layer, under [`SOFT`] GPa, gives and bruises
/// instead of breaking, so no blunt blow gets through it.
fn break_work(area: f32, layer: &Layer) -> f32 {
    if layer.material.stiffness < SOFT {
        return f32::INFINITY;
    }
    layer.material.resilience() * area * layer.thickness
}

/// Returns what `blow` does to `layers`, outermost first.
pub fn strike(blow: &Blow, layers: &[Layer]) -> Strike {
    let mut out = Strike::default();
    let (mut energy, mut sharp) = (blow.energy, blow.mode != Mode::Blunt);
    for (i, layer) in layers.iter().enumerate() {
        if sharp && blow.hardness <= layer.material.hardness {
            out.soft = out.soft.or(Some(i as u8));
            sharp = false;
        }
        let cut = sharp.then(|| cut_work(blow.mode, blow.width, layer));
        sharp &= cut.is_some_and(|w| w <= energy);
        let work = cut
            .filter(|_| sharp)
            .unwrap_or_else(|| break_work(blow.area, layer));
        if work > energy {
            out.part = energy / work.max(f32::EPSILON);
            return Strike { left: 0.0, ..out };
        }
        energy = (energy - work) * KEPT * KEPT;
        out.passed += 1;
    }
    Strike {
        left: energy,
        ..out
    }
}

/// Returns the reference target the solver scores against: a shell-like cuticle 3 mm thick over
/// 10 mm of hide, both at their anchor numbers.
pub fn reference() -> [Layer; 2] {
    [
        Layer {
            class: Class::Cuticle,
            material: Class::Cuticle.anchor(),
            thickness: 0.003,
        },
        Layer {
            class: Class::Hide,
            material: Class::Hide.anchor(),
            thickness: 0.01,
        },
    ]
}
