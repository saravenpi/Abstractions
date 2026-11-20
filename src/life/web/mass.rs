//! The adult mass of a consumer: drawn from its place in the web, heavier for plant eaters that
//! walk, and capped by its plan, its habitat, the rung and gravity. A body too heavy for its plan is
//! built on a skeleton once skeletons exist.

use super::super::chem::Setting;
use super::super::{Habitat, Plan, Rung, body};
use crate::universe::hash::Rng;

/// Standard gravity in metres per second squared, the unit of the square-cube cap.
const STANDARD_GRAVITY: f32 = crate::universe::STANDARD_GRAVITY as f32;

/// Decades of mass a plant eater gains over the niche order. The niche model ranks species by
/// body size as in water, where eaters outweigh what they eat; on land the plant eaters reach the
/// largest sizes, from rodents to elephants (Owen-Smith 1988, "Megaherbivores", Cambridge), so
/// they are drawn about a hundred times heavier than their niche value alone gives.
const PLANT_EATER: f32 = 2.0;

/// The skeletal archetype that grows wings where the air can carry it.
const FLYER: u8 = 3;

/// Trophic level under which a consumer eats plants, as `guild::of` reads it.
const PLANTS: f32 = 2.3;

/// Returns true when a body of `form`, its plan and archetype, can grow wings.
fn winged(form: (Plan, u8)) -> bool {
    form.0 == Plan::Skeletal && form.1 % body::archetypes(form.0) == FLYER
}

/// Returns the decades of mass a consumer of `form`, its plan and archetype, in `habitat` at
/// trophic `level` gains for eating plants: [`PLANT_EATER`] out of the water without wings, else
/// none.
fn grazes(form: (Plan, u8), habitat: Habitat, level: f32) -> f32 {
    let walks = habitat != Habitat::Water && !winged(form);
    if level < PLANTS && walks {
        PLANT_EATER
    } else {
        0.0
    }
}

/// Returns the heaviest body in kilograms `plan` can build: an exoskeleton or a hydrostatic body
/// cannot carry much weight, only an inner skeleton grows large.
fn most(plan: Plan) -> f32 {
    match plan {
        Plan::Skeletal => f32::MAX,
        Plan::Amorphous | Plan::Crystalline => 400.0,
        Plan::Radial => 200.0,
        Plan::Segmented => 60.0,
        Plan::Colonial => 0.05,
    }
}

/// Returns the form, its plan and archetype, and the adult mass in kilograms of a consumer drawn as
/// `form` in `habitat` at niche `niche` and trophic `level`. The mass is log-uniform over two decades
/// that rise with both, heavier for plant eaters by [`grazes`], and capped by the habitat, the rung
/// and gravity, the land cap falling as the cube of gravity by the square-cube law. A species of the
/// air that cannot grow wings walks, so it takes the land cap. A mass past what the plan can build
/// takes a skeleton where the rung has them, and is capped by the plan otherwise.
pub(super) fn mass(
    form: (Plan, u8),
    role: (Habitat, f32, f32),
    setting: &Setting,
    rng: &mut Rng,
) -> ((Plan, u8), f32) {
    let (habitat, niche, level) = role;
    let low = -3.0 + 3.5 * niche + 0.6 * (level - 2.0) + grazes(form, habitat, level);
    let drawn = 10f32.powf(low + 2.0 * rng.unit() as f32);
    let heavy = (STANDARD_GRAVITY / setting.world.gravity.max(1.0)).powi(3);
    let place = match habitat {
        Habitat::Air if winged(form) => 30.0 * setting.world.pressure * heavy.cbrt(),
        Habitat::Water => 20_000.0,
        Habitat::Ground | Habitat::Shore | Habitat::Air => (4000.0 * heavy).min(20_000.0),
    };
    let rung = if setting.rung <= Rung::Soft {
        50.0
    } else {
        f32::MAX
    };
    let wanted = drawn.min(place).min(rung);
    let plan = match wanted > most(form.0) && setting.rung >= Rung::Skeletal {
        true => Plan::Skeletal,
        false => form.0,
    };
    ((plan, form.1), wanted.min(most(plan)).max(1e-4))
}
