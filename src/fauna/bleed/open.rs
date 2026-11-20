//! Opening: the layers of a part of an animal, and whether a blow cut through them into the flesh
//! where the vessels are.

use super::tune::{FLESH, OUTER, THIN};
use crate::craft::build::{Blow, Layer, strike};
use crate::craft::matter::{Class, Grade, material};
use crate::life::Species;
use crate::life::body::PartKind;

/// What a blow opened.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Opening {
    /// How deep it cut, in metres.
    pub depth: f32,
    /// How wide the edge or point that cut was, in metres.
    pub width: f32,
    /// True when it went right through to a major vessel of a neck, a limb root or a torso.
    pub major: bool,
}

/// Returns true for the parts a major vessel runs in.
fn trunk(kind: PartKind) -> bool {
    matches!(kind, PartKind::Neck | PartKind::Torso | PartKind::Thigh)
}

/// Returns the layers of part `index` of `species`, outermost first, and how many of them are outer
/// layers over the flesh: the tissue of the part as hide, shell, plate or bone when it is hard
/// enough to build with, then flesh, which has the numbers of hide.
pub fn stack(species: &Species, index: usize) -> (Vec<Layer>, usize) {
    let part = &species.body.parts[index];
    let thick = part.size.x.min(part.size.z);
    let tissue = species.tissues.get(usize::from(part.tissue));
    let outer = tissue.and_then(|t| {
        let class = Class::of(t.kind, part.kind)?;
        let material = material(class, t, Grade::Good)?;
        let thickness = (OUTER * thick).max(THIN);
        Some(Layer {
            class,
            material,
            thickness,
        })
    });
    let flesh = Layer {
        class: Class::Hide,
        material: Class::Hide.anchor(),
        thickness: (FLESH * thick).max(THIN),
    };
    let skin = usize::from(outer.is_some());
    (outer.into_iter().chain([flesh]).collect(), skin)
}

/// Returns what `blow` opened on part `index` of `species`, or nothing when it did not reach the
/// flesh: a blunt blow bruises and breaks nothing open, an edge too soft for the hide turns blunt.
/// The depth is the layers passed plus the share of the next one the blow had the energy for.
pub fn opened(species: &Species, index: usize, blow: &Blow) -> Option<Opening> {
    let (layers, skin) = stack(species, index);
    let hit = strike(blow, &layers);
    let passed = usize::from(hit.passed);
    if passed < skin || (passed == skin && hit.part <= 0.0) {
        return None;
    }
    let whole: f32 = layers.iter().take(passed).map(|l| l.thickness).sum();
    let next = layers.get(passed).map_or(0.0, |l| l.thickness * hit.part);
    Some(Opening {
        depth: whole + next,
        width: blow.width,
        major: trunk(species.body.parts[index].kind) && passed == layers.len(),
    })
}
