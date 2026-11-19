//! Building a body from its plan, archetype, role, history and world.

use super::super::{History, Plan, Role, World};
use super::{Body, Locomotion};

/// Returns the body of a species whose `form` is its plan and archetype, with `role` and `history` on `world`.
///
/// Phase 0 stub: an empty body.
pub fn build(
    seed: u64,
    plan: Plan,
    archetype: u8,
    role: &Role,
    history: &History,
    world: &World,
) -> Body {
    let _ = (seed, plan, archetype, role, world);
    Body {
        parts: Vec::new(),
        legs: Vec::new(),
        spine: None,
        neck: None,
        tail: None,
        tentacles: Vec::new(),
        wings: None,
        locomotion: Locomotion::Walk,
        length: history.mass.cbrt(),
        height: 0.5 * history.mass.cbrt(),
        features: Vec::new(),
    }
}
