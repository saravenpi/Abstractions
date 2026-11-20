//! Broken parts: a hit hard enough breaks a part of an animal. A broken leg limps, a broken horn
//! cannot charge, a broken tail cannot sweep, a broken jaw cannot bite.

use super::tune::LAME;
use crate::life::body::{Body, PartKind};

/// The parts of one animal that are broken, a bit for each part index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Broken(pub u64);

impl Broken {
    /// Returns true when part `index` is broken.
    pub fn has(self, index: usize) -> bool {
        index < 64 && self.0 >> index & 1 == 1
    }

    /// Returns these with part `index` broken too.
    pub fn with(self, index: usize) -> Self {
        Self(if index < 64 {
            self.0 | 1 << index
        } else {
            self.0
        })
    }

    /// Returns true when `body` has parts of `kind` and every one of them is broken.
    pub fn all(self, body: &Body, kind: PartKind) -> bool {
        let mut parts = body
            .parts
            .iter()
            .enumerate()
            .filter(|(_, p)| p.kind == kind);
        parts.clone().next().is_some() && parts.all(|(i, _)| self.has(i))
    }

    /// Returns true when `body` has parts of `kind` and one of them is still whole.
    pub fn whole(self, body: &Body, kind: PartKind) -> bool {
        let mut parts = body
            .parts
            .iter()
            .enumerate()
            .filter(|(_, p)| p.kind == kind);
        parts.any(|(i, _)| !self.has(i))
    }

    /// Returns the share of the leg bones of `body` that are broken, 0 to 1.
    pub fn lame(self, body: &Body) -> f32 {
        let bones = body.legs.iter().flat_map(|l| l.bones.iter());
        let (all, broken) = bones.fold((0, 0), |(a, b), i| {
            (a + 1, b + usize::from(self.has(usize::from(*i))))
        });
        if all == 0 {
            0.0
        } else {
            broken as f32 / all as f32
        }
    }

    /// Returns the share of its pace an animal of `body` goes at with these parts broken.
    pub fn pace(self, body: &Body) -> f32 {
        1.0 - (1.0 - LAME) * self.lame(body)
    }
}
