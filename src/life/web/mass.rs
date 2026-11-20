//! The adult mass of a consumer: drawn from its place in the web, heavier for plant eaters that
//! walk, and capped by its plan, its habitat, the rung and gravity.

use super::super::chem::Setting;
use super::super::{Habitat, Plan, Rung, body};
use crate::universe::hash::Rng;

/// Gravity of the home planet in metres per second squared, the unit of the square-cube cap.
const HOME_GRAVITY: f32 = crate::universe::HOME_GRAVITY as f32;

/// Decades of mass a plant eater gains over the niche order. The niche model ranks species by
/// body size as in water, where eaters outweigh what they eat; on land the plant eaters reach the
/// largest sizes, from rodents to elephants (Owen-Smith 1988, "Megaherbivores", Cambridge), so
/// they are drawn about a hundred times heavier than their niche value alone gives.
const PLANT_EATER: f32 = 2.0;

/// The skeletal archetype that grows wings where the air can carry it.
const FLYER: u8 = 3;

/// Trophic level under which a consumer eats plants, as `guild::of` reads it.
const PLANTS: f32 = 2.3;

/// Returns the decades of mass a consumer of `form`, its plan and archetype, in `habitat` at
/// trophic `level` gains for eating plants: [`PLANT_EATER`] out of the water without wings, else
/// none.
fn grazes(form: (Plan, u8), habitat: Habitat, level: f32) -> f32 {
    let wings = form.0 == Plan::Skeletal && form.1 % body::archetypes(form.0) == FLYER;
    let walks = habitat != Habitat::Water && !wings;
    if level < PLANTS && walks {
        PLANT_EATER
    } else {
        0.0
    }
}

/// Returns the adult mass in kilograms of a consumer of `form`, its plan and archetype, in
/// `habitat` at niche `niche` and trophic `level`: log-uniform over two decades that rise with
/// both, heavier for plant eaters by [`grazes`], then capped by the plan, the rung and gravity,
/// the land cap falling as the cube of gravity by the square-cube law.
pub(super) fn mass(
    form: (Plan, u8),
    role: (Habitat, f32, f32),
    setting: &Setting,
    rng: &mut Rng,
) -> f32 {
    let (plan, (habitat, niche, level)) = (form.0, role);
    let low = -3.0 + 3.5 * niche + 0.6 * (level - 2.0) + grazes(form, habitat, level);
    let drawn = 10f32.powf(low + 2.0 * rng.unit() as f32);
    let heavy = (HOME_GRAVITY / setting.world.gravity.max(1.0)).powi(3);
    let place = match habitat {
        Habitat::Air => 30.0 * setting.world.pressure * heavy.cbrt(),
        Habitat::Water => 20_000.0,
        Habitat::Ground | Habitat::Shore => (4000.0 * heavy).min(20_000.0),
    };
    let body = match plan {
        Plan::Skeletal => f32::MAX,
        Plan::Amorphous | Plan::Crystalline => 400.0,
        Plan::Radial => 200.0,
        Plan::Segmented => 60.0,
        Plan::Colonial => 0.05,
    };
    let rung = if setting.rung <= Rung::Soft {
        50.0
    } else {
        f32::MAX
    };
    drawn.min(place).min(body).min(rung).max(1e-4)
}
