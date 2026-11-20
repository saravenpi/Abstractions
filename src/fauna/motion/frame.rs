//! What each part of a body is for: which motion places it.

use crate::life::body::{Body, Chain, PartKind};

/// Which motion drives a part.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// Carried rigidly by its parent.
    Free,
    /// The root part, which breathes.
    Torso,
    /// Bone `bone` of leg `leg`, placed by the leg solver.
    Bone(u8, u8),
    /// Joint `joint` of strand `strand`, placed by its chain.
    Link(u8, u8),
    /// The head, which looks around.
    Head,
    /// A lower jaw, which drops when the body calls.
    Jaw,
    /// A wing on side -1 or 1.
    Wing(i8),
    /// An antenna on side -1, 0 or 1.
    Antenna(i8),
    /// A bell or a blob, which squashes and stretches.
    Squash,
}

/// Which chain a strand follows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// The spine, pulled along behind its root.
    Spine,
    /// The neck, reaching for where the head should be.
    Neck,
    /// The tail, lagging on springs.
    Tail,
    /// A tentacle, trailing with a sway.
    Tentacle,
}

/// Returns the chains of `body` with their kinds: spine, neck, tail, then the tentacles.
pub fn chains(body: &Body) -> Vec<(Kind, &Chain)> {
    let named = [
        (Kind::Spine, &body.spine),
        (Kind::Neck, &body.neck),
        (Kind::Tail, &body.tail),
    ];
    let named = named
        .into_iter()
        .filter_map(|(kind, chain)| chain.as_ref().map(|c| (kind, c)));
    named
        .chain(body.tentacles.iter().map(|c| (Kind::Tentacle, c)))
        .collect()
}

/// Returns the role of every part of `body`.
pub fn roles(body: &Body) -> Vec<Role> {
    let mut roles: Vec<Role> = body.parts.iter().map(|p| own(p.kind, p.side)).collect();
    if let Some(first) = roles.first_mut().filter(|r| **r == Role::Free) {
        *first = Role::Torso;
    }
    for (strand, (_, chain)) in chains(body).iter().enumerate() {
        for (joint, &part) in chain.parts.iter().enumerate() {
            set(&mut roles, part, Role::Link(strand as u8, joint as u8));
        }
    }
    for (leg, limb) in body.legs.iter().enumerate() {
        for (bone, &part) in limb.bones.iter().enumerate() {
            set(&mut roles, part, Role::Bone(leg as u8, bone as u8));
        }
    }
    if let Some(wings) = body.wings {
        set(&mut roles, wings.parts[0], Role::Wing(-1));
        set(&mut roles, wings.parts[1], Role::Wing(1));
    }
    roles
}

/// Returns the role a part of `kind` on `side` has on its own.
fn own(kind: PartKind, side: i8) -> Role {
    match kind {
        PartKind::Head => Role::Head,
        PartKind::Jaw => Role::Jaw,
        PartKind::Antenna => Role::Antenna(side),
        PartKind::Bell | PartKind::Blob => Role::Squash,
        _ => Role::Free,
    }
}

/// Sets the role of part `index` in `roles`, if there is such a part.
fn set(roles: &mut [Role], index: u8, role: Role) {
    if let Some(slot) = roles.get_mut(usize::from(index)) {
        *slot = role;
    }
}
