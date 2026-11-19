//! Life history from mass: small species are many, quick and short-lived, large ones few, slow and caring.

use super::{History, Role, Rung, World};

/// Returns the life history of a species of `role` and adult `mass` in kilograms on `world`.
///
/// Phase 0 stub: a lone, slow animal.
pub fn of(seed: u64, role: &Role, mass: f32, rung: Rung, world: &World) -> History {
    let _ = (seed, role, rung, world);
    History {
        mass,
        lifespan: 3600.0,
        clutch: 1,
        group_min: 1,
        group_max: 1,
        care: 0.0,
        speed: 2.0,
        range: 200.0,
    }
}
