//! The held tool: a craft recipe in the hand, with the numbers its build has on this world.

use bevy::prelude::*;

use super::act::Strike;
use super::layer::Layer;
use super::tune::{HEAVY_HEAD, LIGHT, PARRY, PARRY_LIGHT};
use crate::craft::build::{Build, Form, Intent, Place, Recipe, Swing, Throw, mass, swing, throw};

/// How a build is swung, from its shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Style {
    /// A point: stabbed straight ahead.
    Thrust,
    /// A heavy head: chopped or smashed, overhead unless swung sideways.
    Chop,
    /// An edge or a light head: slashed in the direction the mouse asks.
    Slash,
}

impl Style {
    /// Returns how `build` is swung: a point thrusts, a head with more than [`HEAVY_HEAD`] of the mass
    /// chops, anything else slashes.
    pub fn of(build: &Build) -> Self {
        let share = build.head.mass / mass(&build.pieces()).max(f32::EPSILON);
        match build.head.form {
            Form::Point => Style::Thrust,
            _ if share > HEAVY_HEAD => Style::Chop,
            _ => Style::Slash,
        }
    }

    /// Returns the intent a blow of this style carries.
    pub fn intent(self) -> Intent {
        match self {
            Style::Thrust => Intent::Pierce,
            Style::Chop => Intent::Crush,
            Style::Slash => Intent::Cut,
        }
    }
}

/// The tool in the robot's hands, on the root of its body.
#[derive(Component, Debug, Clone, PartialEq)]
#[require(Strike, Layer)]
pub struct Held {
    /// The item as it is stored.
    pub recipe: Recipe,
    /// The build the recipe makes.
    pub build: Build,
    /// The world it is used on.
    pub place: Place,
    /// How it swings there.
    pub swing: Swing,
    /// How it flies there.
    pub throw: Throw,
    /// How it is swung.
    pub style: Style,
    /// True while it is in the hands, false while it is stowed.
    pub out: bool,
}

impl Held {
    /// Returns the item of `recipe`, built as `build`, in the hands at `place`.
    pub fn new(recipe: Recipe, build: Build, place: Place) -> Self {
        Self {
            swing: swing(&build, &place),
            throw: throw(&build, &place),
            style: Style::of(&build),
            recipe,
            build,
            place,
            out: true,
        }
    }

    /// Returns true when the item is made to be thrown: the primary button draws and throws it.
    pub fn thrown(&self) -> bool {
        self.recipe.design.intent == Intent::Throw
    }

    /// Returns how long a raised guard parries a blow, in seconds: shorter for a very light tool.
    pub fn parry(&self) -> f32 {
        match self.throw.mass < LIGHT {
            true => PARRY_LIGHT,
            false => PARRY,
        }
    }

    /// Returns the share of the mass of the counterweight among head and counterweight, 0 to 1: how
    /// much it calms the kickback of a blow.
    pub fn counter(&self) -> f32 {
        let weight = self.build.counterweight.map_or(0.0, |w| w.mass);
        weight / (weight + self.build.head.mass).max(f32::EPSILON)
    }
}
