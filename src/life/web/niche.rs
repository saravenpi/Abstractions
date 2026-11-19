//! The niche model of Williams and Martinez (2000), "Simple rules yield complex food webs",
//! <https://www.sccs.swarthmore.edu/users/08/bblonder/phys120/docs/martinez.pdf>.
//!
//! Each species gets a niche value on 0 to 1, a feeding range and a range centre, and eats every
//! species whose niche value falls inside its range. Producers take the lowest niche values and eat
//! nothing.

use crate::universe::hash::Rng;

/// Passes of the prey-averaged trophic level, enough for chains of any length the model draws.
const PASSES: usize = 8;

/// A drawn food web: one entry per species in each list, lowest niche value first.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Web {
    /// Niche values, ascending.
    pub niche: Vec<f32>,
    /// Feeding range widths, 0 for producers.
    pub range: Vec<f32>,
    /// Feeding range centres, 0 for producers.
    pub centre: Vec<f32>,
    /// Indices of the species each one eats.
    pub prey: Vec<Vec<u16>>,
    /// Prey-averaged trophic levels, 1 for producers.
    pub level: Vec<f32>,
    /// How many of the first species are producers.
    pub producers: usize,
}

/// The shape of a web to draw.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Shape {
    /// Number of species.
    pub species: usize,
    /// Number of producers among them, at most `species`.
    pub producers: usize,
    /// Target connectance, links over species squared.
    pub connectance: f32,
    /// Highest range centre over the top producer niche, which keeps chains short; `None` for no cap.
    pub reach: Option<f32>,
}

/// Returns the prey of a consumer of niche `niche` with range `range` around `centre`, among `niches`:
/// every other species inside the range, else the species just below it on the niche axis.
fn prey(index: usize, range: f32, centre: f32, niches: &[f32]) -> Vec<u16> {
    let inside: Vec<u16> = (0..niches.len())
        .filter(|&j| j != index && (niches[j] - centre).abs() <= range / 2.0)
        .map(|j| j as u16)
        .collect();
    match inside.is_empty() {
        true => vec![index.saturating_sub(1) as u16],
        false => inside,
    }
}

/// Returns the prey-averaged trophic level of every species of a web with `prey` and `producers`.
fn levels(prey: &[Vec<u16>], producers: usize) -> Vec<f32> {
    let mut level: Vec<f32> = (0..prey.len())
        .map(|i| if i < producers { 1.0 } else { 2.0 })
        .collect();
    for _ in 0..PASSES {
        for i in producers..prey.len() {
            let sum: f32 = prey[i].iter().map(|&j| level[usize::from(j)]).sum();
            level[i] = 1.0 + sum / prey[i].len().max(1) as f32;
        }
    }
    level
}

/// Returns a web of `shape` drawn from `rng`.
///
/// A consumer of niche value `n` gets a range `r = n * (1 - (1 - y)^(1 / beta))` for a uniform draw
/// `y`, with `beta = 1 / (2 C) - 1`, so ranges average `C` and the web keeps about `C S^2` links,
/// and a centre uniform in `[r / 2, n]`.
pub(super) fn draw(shape: Shape, rng: &mut Rng) -> Web {
    let mut niche: Vec<f32> = (0..shape.species).map(|_| rng.unit() as f32).collect();
    niche.sort_by(f32::total_cmp);
    let beta = 1.0 / (2.0 * shape.connectance) - 1.0;
    let top = niche[shape.producers.max(1) - 1];
    let (mut range, mut centre) = (vec![0.0; shape.species], vec![0.0; shape.species]);
    for i in shape.producers..shape.species {
        let y = rng.unit() as f32;
        range[i] = niche[i] * (1.0 - (1.0 - y).powf(1.0 / beta));
        let c = range[i] / 2.0 + (niche[i] - range[i] / 2.0) * rng.unit() as f32;
        centre[i] = shape.reach.map_or(c, |reach| c.min(top + reach));
    }
    let prey: Vec<Vec<u16>> = (0..shape.species)
        .map(|i| match i < shape.producers {
            true => Vec::new(),
            false => prey(i, range[i], centre[i], &niche),
        })
        .collect();
    Web {
        level: levels(&prey, shape.producers),
        niche,
        range,
        centre,
        prey,
        producers: shape.producers,
    }
}
