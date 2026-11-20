//! Making a tool: what a proposal takes from the pack, and the tool it leaves in it.

use super::Pack;
use super::tune::BINDING;
use crate::craft::build::{Proposal, Stock};
use crate::craft::matter::{Material, Stuff, Treatment};

/// Returns what `proposal` takes from `stock`: the mass of each stock entry it uses, by index. The
/// shaft and grip take from the shaft material, the head from the head material, and a binding takes
/// a fixed small mass.
pub fn cost(proposal: &Proposal, stock: &[Stock]) -> Vec<(usize, f32)> {
    let pieces = proposal.build.pieces();
    let [shaft, head, bind, _] = proposal.design.of.map(usize::from);
    let uses = [
        (shaft, pieces[0].mass + pieces[1].mass),
        (head, pieces[2].mass),
        (bind, BINDING),
    ];
    let mut out: Vec<(usize, f32)> = Vec::with_capacity(3);
    for (at, kg) in uses.into_iter().filter(|(at, _)| *at < stock.len()) {
        match out.iter_mut().find(|(i, _)| *i == at) {
            Some(entry) => entry.1 += kg,
            None => out.push((at, kg)),
        }
    }
    out
}

impl Pack {
    /// Makes `proposal` from `stock`, the pack's own materials: takes what it costs, keeps the tool
    /// and returns its id, or returns nothing and takes nothing when a material runs short.
    pub fn make(&mut self, proposal: &Proposal, stock: &[Stock]) -> Option<u32> {
        let cost = cost(proposal, stock);
        let short = |(at, kg): &(usize, f32)| {
            let heap = self.stacks.iter().find(|s| s.stuff == stock[*at].stuff);
            heap.is_none_or(|s| s.mass + 1.0e-4 < *kg)
        };
        if cost.iter().any(short) {
            return None;
        }
        for (at, kg) in &cost {
            self.take(&stock[*at].stuff, *kg);
        }
        let recipe = proposal.recipe(stock);
        let material = |stuff| stock.iter().find(|s| s.stuff == stuff).map(|s| s.material);
        let materials: Option<Vec<Material>> = recipe.stuff.iter().map(|s| material(*s)).collect();
        Some(self.keep(recipe, materials?))
    }

    /// Gives the whole stack of the tissue `stuff` `treatment`, merging it with a stack that already
    /// had it, and returns false, changing nothing, for a stone or a tissue that had it already.
    pub fn treat(&mut self, stuff: &Stuff, treatment: Treatment) -> bool {
        let Stuff::Tissue { treated, .. } = *stuff else {
            return false;
        };
        let done = treated.with(treatment);
        let Some(at) = self.stacks.iter().position(|s| s.stuff == *stuff) else {
            return false;
        };
        if done == treated {
            return false;
        }
        let old = self.stacks.remove(at);
        let mut new = old;
        if let Stuff::Tissue { treated, .. } = &mut new.stuff {
            *treated = done;
        }
        new.material = treatment.apply(old.material);
        self.stow(new, f32::INFINITY);
        true
    }
}
