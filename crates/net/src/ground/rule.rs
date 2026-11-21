//! The rule of the ground at work: where a thing put down goes, and what a take gets.

use super::{CELL, CELL_MAX, CRUMB, GROUND_MAX, MERGE, Pile, Placed, Took};
use crate::ids::BodyId;
use crate::wire::{HEAP, Lying, TOOL};

/// Returns the cube of the ground `lying` is in.
fn cell(lying: &Lying) -> (BodyId, [i64; 3]) {
    (lying.body, lying.at.map(|x| (x / CELL).floor() as i64))
}

/// Returns the distance in metres between two things.
fn apart(a: &Lying, b: &Lying) -> f64 {
    let squared: f64 = a.at.iter().zip(b.at).map(|(x, y)| (x - y) * (x - y)).sum();
    squared.sqrt()
}

impl Pile {
    /// Returns where a heap of the material of `heap` lies that it joins: within [`MERGE`] metres,
    /// or anywhere in its cube when `anywhere`.
    fn kin(&self, heap: &Lying, anywhere: bool) -> Option<usize> {
        let same =
            |l: &&Lying| l.kind == HEAP && l.body == heap.body && l.material == heap.material;
        let near = |l: &Lying| apart(l, heap) < MERGE || (anywhere && cell(l) == cell(heap));
        let found = self.lying.iter().enumerate().filter(|(_, l)| same(l));
        found
            .filter(|(_, l)| near(*l))
            .min_by(|a, b| apart(a.1, heap).total_cmp(&apart(b.1, heap)))
            .map(|(at, _)| at)
    }

    /// Adds `heap` to the heap at `at`, which then carries the name of its drop, and returns it.
    fn join(&mut self, at: usize, heap: &Lying) -> Placed {
        let kept = &mut self.lying[at];
        kept.mass += heap.mass;
        (kept.by, kept.tag) = (heap.by, heap.tag);
        Placed::Joined(kept.clone())
    }

    /// Puts `lying` down under the rule of the module, giving it an id when it lies on its own.
    pub fn lay(&mut self, lying: Lying) -> Placed {
        let heap = lying.kind == HEAP;
        if let Some(at) = self.kin(&lying, false).filter(|_| heap) {
            return self.join(at, &lying);
        }
        let here = cell(&lying);
        let crowd = self.lying.iter().filter(|l| cell(l) == here).count();
        if crowd < CELL_MAX && self.lying.len() < GROUND_MAX {
            let id = self.next.max(1);
            self.next = id + 1;
            let laid = Lying { id, ..lying };
            self.lying.push(laid.clone());
            return Placed::Laid(laid);
        }
        match self.kin(&lying, true).filter(|_| heap) {
            Some(at) => self.join(at, &lying),
            None => Placed::Refused,
        }
    }

    /// Takes the thing `id` off the ground: a tool whole, a heap up to `mass` kilograms. Returns
    /// nothing when there is no such thing or `mass` is not a positive number.
    pub fn take(&mut self, id: u64, mass: f32) -> Option<Took> {
        let at = self.lying.iter().position(|l| l.id == id)?;
        if !(mass.is_finite() && mass > 0.0) {
            return None;
        }
        let thing = &mut self.lying[at];
        if thing.kind == TOOL || mass + CRUMB >= thing.mass {
            let got = self.lying.remove(at);
            return Some(Took { got, left: None });
        }
        thing.mass -= mass;
        let got = Lying {
            mass,
            ..thing.clone()
        };
        Some(Took {
            got,
            left: Some(thing.clone()),
        })
    }
}
