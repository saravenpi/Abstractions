//! The rule table: how the planet and the role of a species shape its body.

use crate::life::{Activity, Habitat, History, Role, World};
use crate::universe::{EARTH_RADIUS, NO_AIR, STANDARD_GRAVITY};

/// Lift ratio, pressure over gravity both against home, under which nothing flies.
const LIFT_FLOOR: f32 = 0.25;

/// Heaviest flyer in kilograms when the lift ratio is 1. It grows with the square of the ratio.
const FLYER_MASS: f32 = 20.0;

/// Body length in metres of a species of 1 kg.
const LENGTH_PER_KG: f32 = 0.35;

/// Longest land body in metres on a world of home gravity. It shrinks as gravity grows.
const LAND_CAP: f32 = 48.0;

/// Mean temperature in kelvin at which the cold rules do nothing.
const MILD: f32 = 288.0;

/// Kelvin of cold, or heat, that moves the cold rules all the way.
const COLD_SPAN: f32 = 60.0;

/// Surface temperature in kelvin of a star whose light the eyes are tuned to.
const SUNLIKE: f32 = 5800.0;

/// What the rules decide for one body. Factors are around 1 on a home-like world.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Traits {
    /// Body length scale in metres.
    pub size: f32,
    /// Limb thickness over the plain one: square-cube law against gravity and mass.
    pub thick: f32,
    /// Limb length over the plain one: long on light worlds, short in the cold.
    pub reach: f32,
    /// Tail, ear and neck length over the plain one: short in the cold (Allen).
    pub append: f32,
    /// Torso width and depth over the plain one: round in the cold (Bergmann).
    pub round: f32,
    /// Hide thickness, which swells the torso and the neck in the cold.
    pub hide: f32,
    /// Eye radius over head size, large where light is dim.
    pub eye: f32,
    /// Wing span over body length, when the air can carry this body.
    pub wing: Option<f32>,
}

/// Returns the traits the rule table gives a species of `role` and `history` on `world`.
pub fn traits(role: &Role, history: &History, world: &World) -> Traits {
    let g = gravity(world);
    let cold = cold(world);
    let mass = history.mass.max(1e-4);
    let land = !matches!(role.habitat, Habitat::Water | Habitat::Air);
    let plain = LENGTH_PER_KG * mass.cbrt() * insular(world, mass);
    let size = if land { plain.min(LAND_CAP / g) } else { plain };
    Traits {
        size,
        thick: (g.sqrt() * (mass / 50.0).powf(0.08)).clamp(0.35, 3.8),
        reach: (g.powf(-0.25) * (1.0 - 0.15 * cold)).clamp(0.55, 1.6),
        append: (1.0 - 0.3 * cold).clamp(0.6, 1.4),
        round: (1.0 + 0.2 * cold).clamp(0.8, 1.25),
        hide: 1.0 + 0.15 * cold.max(0.0),
        eye: eye(world, role.activity),
        wing: wing(world, mass),
    }
}

/// Returns the surface gravity in standard gravities, the pull of the Earth being 1.
pub fn gravity(world: &World) -> f32 {
    (world.gravity / STANDARD_GRAVITY as f32).max(0.05)
}

/// Returns the lift ratio: air pressure over gravity, both against the Earth.
pub fn lift(world: &World) -> f32 {
    world.pressure / gravity(world)
}

/// Returns how cold the world is, from -1 (hot) through 0 (mild) to 1 (cold).
fn cold(world: &World) -> f32 {
    ((MILD - world.temperature) / COLD_SPAN).clamp(-1.0, 1.0)
}

/// Returns the size factor of island rules: on small or watery worlds the large shrink and the small grow.
fn insular(world: &World, mass: f32) -> f32 {
    let earth = if world.radius < 100_000.0 {
        6371.0
    } else {
        EARTH_RADIUS as f32
    };
    let small = ((1.0 - world.radius / earth) / 0.5).clamp(0.0, 1.0);
    let island = ((world.ocean - 0.6) / 0.3).clamp(0.0, 1.0);
    let pressure = small.max(island);
    let bigness = (mass.log10() / 1.7).clamp(0.0, 1.0);
    1.0 + pressure * (0.3 - 0.65 * bigness)
}

/// Returns the eye radius over head size from the light of the star and the hours a species keeps.
fn eye(world: &World, activity: Activity) -> f32 {
    let light = world.flux.max(0.01) * (world.star / SUNLIKE).clamp(0.35, 1.3);
    let dark = match activity {
        Activity::Day => 1.0,
        Activity::Twilight => 1.2,
        Activity::Night => 1.5,
    };
    0.16 * (light.powf(-0.25) * dark).clamp(0.7, 2.4)
}

/// Returns the wing span over body length, or nothing when the air is too thin or the body too heavy.
fn wing(world: &World, mass: f32) -> Option<f32> {
    let lift = lift(world);
    let air = world.pressure >= NO_AIR as f32;
    let carried = mass <= FLYER_MASS * lift * lift;
    (air && lift >= LIFT_FLOOR && carried).then(|| (1.6 / lift.sqrt()).clamp(1.2, 3.5))
}
