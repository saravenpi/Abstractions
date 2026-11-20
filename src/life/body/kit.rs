//! The kit a body is assembled with: parts placed parents first, mirrored pairs, chains and features.

use super::frame::mirror;
use super::sink::{limb, root, sink};
use super::{Body, Chain, Feature, FeatureKind, Locomotion, Part, PartKind};

/// Most parts in one body, so that every individual stays cheap to draw and to move.
pub const MAX_PARTS: usize = 16;

/// Returns true when `part` grows from `host` as a limb from the body: a limb of another kind than its
/// host, legs aside.
fn grows(part: &Part, host: &Part) -> bool {
    limb(part.kind) && part.kind != PartKind::Thigh && part.kind != host.kind
}

/// A body under construction.
#[derive(Debug, Clone, PartialEq)]
pub struct Kit {
    /// The body so far.
    pub body: Body,
    /// Most parts this body may hold.
    pub cap: usize,
}

impl Kit {
    /// Returns an empty kit that holds at most `cap` parts.
    pub fn new(cap: usize) -> Self {
        let body = Body {
            parts: Vec::new(),
            legs: Vec::new(),
            spine: None,
            neck: None,
            tail: None,
            tentacles: Vec::new(),
            wings: None,
            locomotion: Locomotion::Walk,
            length: 0.0,
            height: 0.0,
            features: Vec::new(),
        };
        Self { body, cap }
    }

    /// Returns whether `count` more parts fit under the cap.
    pub fn fits(&self, count: usize) -> bool {
        self.body.parts.len() + count <= self.cap
    }

    /// Adds `part` under `parent` and returns its index. The root of a neck, tail, wing or fin grown
    /// from a part of another kind is sunk inside that part by its own radius; legs sink their hips
    /// before they are posed, in `leg`.
    pub fn add(&mut self, parent: Option<u8>, part: Part) -> u8 {
        let index = self.body.parts.len() as u8;
        let host = parent.and_then(|p| self.body.parts.get(usize::from(p)));
        let socket = match host {
            Some(host) if grows(&part, host) => sink(host, part.socket, root(&part)),
            _ => part.socket,
        };
        self.body.parts.push(Part {
            parent,
            socket,
            ..part
        });
        index
    }

    /// Adds `right`, a part on the right side, and its mirrored twin on the left under `parents`
    /// (left parent, right parent). Returns the indices, left first.
    pub fn pair(&mut self, parents: [u8; 2], right: Part) -> [u8; 2] {
        let right = Part { side: 1, ..right };
        let left = self.add(Some(parents[0]), mirror(right));
        [left, self.add(Some(parents[1]), right)]
    }

    /// Adds `links` as a chain under `parent`, or as the root for none, each link the child of the one
    /// before, and returns it.
    ///
    /// Every link hangs, so `lengths[i]` is the socket distance of link `i + 1` and the last entry is the
    /// length of the last link.
    pub fn chain(&mut self, parent: Option<u8>, links: &[Part], limit: f32) -> Chain {
        let mut parts = Vec::with_capacity(links.len());
        let mut above = parent;
        for link in links {
            let index = self.add(above, *link);
            parts.push(index);
            above = Some(index);
        }
        let mut lengths: Vec<f32> = links.iter().skip(1).map(|l| l.socket.length()).collect();
        lengths.extend(links.last().map(|l| l.size.y));
        Chain {
            parts,
            lengths,
            limit,
        }
    }

    /// Lists a feature of `kind` with `size` in metres, made of `tissue`.
    pub fn feature(&mut self, kind: FeatureKind, size: f32, tissue: u8) {
        self.body.features.push(Feature { kind, size, tissue });
    }
}
