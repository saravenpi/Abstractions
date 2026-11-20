//! What the parts of a body add up to: how long its neck and tail are, whether it has a mouth and
//! claws. Attacks read it.

use super::limbs::Broken;
use crate::life::body::{Body, PartKind};
use crate::life::{Guild, Species};

/// Returns the length of the neck of `body` in metres, 0 without one.
pub fn neck(body: &Body) -> f32 {
    body.neck.as_ref().map_or(0.0, |c| c.lengths.iter().sum())
}

/// Returns the length of the tail of `body` in metres, 0 without one.
pub fn tail(body: &Body) -> f32 {
    body.tail.as_ref().map_or(0.0, |c| c.lengths.iter().sum())
}

/// Returns true when `body` has a jaw or a head that is not broken.
pub fn mouth(body: &Body, broken: Broken) -> bool {
    broken.whole(body, PartKind::Jaw)
        || (!broken.all(body, PartKind::Head)
            && body.parts.iter().any(|p| p.kind == PartKind::Head))
}

/// Returns true when `species` hunts with a front pair of legs.
pub fn claws(species: &Species) -> bool {
    let hunter = matches!(species.role.guild, Guild::Hunter | Guild::Ambusher);
    hunter && species.body.legs.iter().any(|l| l.rank == 0)
}
