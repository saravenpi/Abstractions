//! Recipes: an item as the few bytes that rebuild it, with stats recomputed on load.
//!
//! Like a Minecraft item stores only what differs from its defaults
//! (https://minecraft.wiki/w/Data_component_format), a recipe stores the part sizes as bytes, the
//! materials as references to where they came from, and the blows each joint has taken.

use serde::{Deserialize, Serialize};

use super::part::{Build, Head, Shaft, Weight};
use super::tune::{BORE, GRIP, SLENDER};
use super::{Hands, Intent, Wear};
use crate::craft::matter::{Material, Stuff};

/// Range of the grip length, in metres.
const GRIPS: [f32; 2] = [0.10, 0.15];

/// Range of the shaft length, in metres.
const LENGTHS: [f32; 2] = [0.2, 1.4];

/// Range of a head or counterweight mass, in kilograms.
const MASSES: [f32; 2] = [0.05, 2.0];

/// The byte that marks a missing binding or counterweight.
const NONE: u8 = u8::MAX;

/// Returns `x` within `range` as a byte, 0 at the low end and 254 at the high end.
fn pack(x: f32, range: [f32; 2]) -> u8 {
    ((x - range[0]) / (range[1] - range[0]) * 254.0)
        .round()
        .clamp(0.0, 254.0) as u8
}

/// Returns the value a byte from [`pack`] stands for within `range`.
fn unpack(q: u8, range: [f32; 2]) -> f32 {
    range[0] + f32::from(q.min(254)) / 254.0 * (range[1] - range[0])
}

/// The shape of a build, its sizes as bytes and its materials as indices into a material list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Design {
    /// What it is for, which sets the head form.
    pub intent: Intent,
    /// Grip length, packed.
    pub grip: u8,
    /// Shaft length, packed.
    pub length: u8,
    /// Shaft diameter, packed.
    pub bore: u8,
    /// Head mass, packed.
    pub head: u8,
    /// Counterweight mass, packed, or [`NONE`].
    pub counter: u8,
    /// How it is held.
    pub hands: Hands,
    /// Materials of the shaft, head, binding and counterweight, as indices; [`NONE`] when missing.
    pub of: [u8; 4],
}

impl Design {
    /// Returns a design for `intent` with a shaft of `length` metres and a head of `head` kilograms,
    /// the shaft, head and binding made of the materials at `of` (a binding of [`u8::MAX`] is none):
    /// a plain grip, a shaft as slender as the solver builds them, no counterweight, one hand.
    pub fn of(intent: Intent, length: f32, head: f32, of: [u8; 3]) -> Self {
        Self {
            intent,
            grip: pack(GRIP, GRIPS),
            length: pack(length, LENGTHS),
            bore: pack((length * SLENDER).clamp(BORE[0], BORE[1]), BORE),
            head: pack(head, MASSES),
            counter: NONE,
            hands: Hands::One,
            of: [of[0], of[1], of[2], NONE],
        }
    }

    /// Returns the build this design makes from `materials`, or nothing when a shaft or head
    /// material is missing.
    pub fn build(&self, materials: &[Material]) -> Option<Build> {
        let pick = |i: u8| materials.get(usize::from(i)).copied();
        let counterweight = (self.counter != NONE).then_some(()).and(pick(self.of[3]));
        Some(Build {
            grip: unpack(self.grip, GRIPS),
            shaft: Shaft {
                length: unpack(self.length, LENGTHS),
                diameter: unpack(self.bore, BORE),
                material: pick(self.of[0])?,
            },
            head: Head {
                form: self.intent.form(),
                mass: unpack(self.head, MASSES),
                material: pick(self.of[1])?,
            },
            counterweight: counterweight.map(|material| Weight {
                mass: unpack(self.counter, MASSES),
                material,
            }),
            binding: pick(self.of[2]),
            hands: self.hands,
        })
    }
}

/// A stored item: its design, the stuff its materials came from, and its wear.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Recipe {
    /// Sizes and material indices into `stuff`.
    pub design: Design,
    /// Where each material came from.
    pub stuff: Vec<Stuff>,
    /// Blows per joint.
    pub wear: Wear,
}

impl Recipe {
    /// Returns the build of this item, reading each material through `lookup`, or nothing when a
    /// material can no longer be read.
    pub fn build(&self, lookup: impl Fn(&Stuff) -> Option<Material>) -> Option<Build> {
        let materials: Option<Vec<Material>> = self.stuff.iter().map(lookup).collect();
        self.design.build(&materials?)
    }
}
