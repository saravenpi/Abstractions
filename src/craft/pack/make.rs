//! Making a tool: what a proposal takes from the pack, and the tool it leaves in it.

use super::Pack;
use super::tune::BINDING;
use crate::craft::build::{Proposal, Stock};
use crate::craft::matter::{Material, Stuff, Treatment};

/// Returns what `proposal` takes from `stock`: the mass of each stock entry it uses, by index.
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
    /// Makes `proposal` from `stock`, takes discrete units needed, keeps the tool and returns its id.
    pub fn make(&mut self, proposal: &Proposal, stock: &[Stock]) -> Option<u32> {
        self.ensure_grid();
        let cost = cost(proposal, stock);
        let short = |(at, kg): &(usize, f32)| {
            let stuff = &stock[*at].stuff;
            let unit = stuff.unit_mass().max(1e-4);
            let want = ((*kg - 1.0e-4) / unit).ceil().max(1.0) as u32;
            self.count_of(stuff) < want
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

    /// Gives the carried stack of tissue `stuff` `treatment`, merging cleanly into the slot grid.
    pub fn treat(&mut self, stuff: &Stuff, treatment: Treatment) -> bool {
        self.ensure_grid();
        let Stuff::Tissue { treated, .. } = *stuff else {
            return false;
        };
        let done = treated.with(treatment);
        if done == treated {
            return false;
        }
        let Some(old) = self.stacks.iter().find(|s| s.stuff == *stuff).copied() else {
            return false;
        };
        let units = self.count_of(stuff) as u16;
        if units == 0 || !self.take_count(stuff, units) {
            return false;
        }
        let mut next = *stuff;
        if let Stuff::Tissue { treated, .. } = &mut next {
            *treated = done;
        }
        let treated_mat = treatment.apply(old.material);
        let mut remaining = units;
        while remaining > 0 {
            let chunk = remaining.min(super::STACK_MAX);
            if self.add_stack(next, treated_mat, chunk) == 0 {
                break;
            }
            remaining -= chunk;
        }
        true
    }
}
