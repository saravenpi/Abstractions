//! Items: a made tool, kept as its recipe beside the numbers of the materials it was made from.
//!
//! A tissue's numbers come from the life of the planet it was harvested on, which is not at hand
//! once the robot has left it, so an item keeps the materials it was made with.

use serde::{Deserialize, Serialize};

use super::words::{called, noun};
use crate::craft::build::{Build, Intent, Joint, Look, Place, Recipe, lives, mass, swing};
use crate::craft::matter::{Material, Stuff};

/// A made tool in the pack.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Item {
    /// Its id in the pack, which the hotbar points at.
    pub id: u32,
    /// What it is made of and how, and the blows its joints have taken.
    pub recipe: Recipe,
    /// The numbers of each material of the recipe, in the order of its stuff.
    pub materials: Vec<Material>,
}

impl Item {
    /// Returns the build of the item, or nothing when its recipe lost a material.
    pub fn build(&self) -> Option<Build> {
        self.recipe.design.build(&self.materials)
    }

    /// Returns what the item is for.
    pub fn intent(&self) -> Intent {
        self.recipe.design.intent
    }

    /// Returns the mass of the item in kilograms.
    pub fn mass(&self) -> f32 {
        self.build().map_or(0.0, |build| mass(&build.pieces()))
    }

    /// Returns the material at slot `at` of the design: shaft, head, binding.
    fn part(&self, at: usize) -> Option<&Stuff> {
        let index = usize::from(*self.recipe.design.of.get(at)?);
        self.recipe.stuff.get(index)
    }

    /// Returns the name of the item: its head material and the word for its shape.
    pub fn name(&self) -> String {
        let Some(build) = self.build() else {
            return "broken pieces".to_string();
        };
        let share = build.head.mass / mass(&build.pieces()).max(f32::EPSILON);
        let word = noun(self.intent(), share, build.reach());
        format!("{} {word}", self.part(1).map_or("bare", called))
    }

    /// Returns the signature of the item: the head on the shaft, and the binding.
    pub fn signature(&self) -> String {
        let name = |at: usize| self.part(at).map_or("?", called);
        let (head, shaft) = (name(1), name(0));
        match self.part(2).map(called) {
            Some(bind) => format!("{head} on {shaft}, {bind} bound"),
            None => format!("{head} on {shaft}"),
        }
    }

    /// Returns how worn the item is at `place`, 0 new to 1 broken: the joint furthest into its
    /// life, and that joint.
    pub fn worn(&self, place: &Place) -> (f32, Joint) {
        let Some(build) = self.build() else {
            return (1.0, Joint::Head);
        };
        let lives = lives(&build, &swing(&build, place));
        let used = |j: Joint| {
            f32::from(self.recipe.wear.0[j as usize]) / lives[j as usize].max(f32::EPSILON)
        };
        let joint = self.recipe.wear.weakest(&lives);
        (used(joint).clamp(0.0, 1.0), joint)
    }

    /// Returns the look the icon of the item is drawn from: its head, shaft and materials, or the
    /// bare look of its intent when its recipe lost a material. Every item has one.
    pub fn shape(&self) -> Option<Look> {
        let Some(build) = self.build() else {
            return Some(Look::bare(self.intent()));
        };
        Some(Look::of(&build, self.intent(), self.part(1), self.part(0)))
    }
}
