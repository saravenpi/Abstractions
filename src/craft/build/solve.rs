//! The solver: answers an intent with three builds from what a player carries.
//!
//! It tries every head material, shaft material, head mass and shaft length in a fixed order, at most
//! [`COMBINATIONS`] of them, scores each against the reference target, and keeps the lightest, the
//! hardest-hitting and the most durable. Three choices, not fifty: decision time grows with
//! log₂(n + 1) (Hick's law, https://en.wikipedia.org/wiki/Hick%27s_law). It is deterministic, so a
//! server running it agrees with the client.

use super::blow::blow;
use super::hit::{Layer, Strike, reference, strike};
use super::inertia::mass;
use super::part::Build;
use super::recipe::{Design, Recipe};
use super::swing::{Swing, swing};
use super::tune::{COMBINATIONS, EDGE_MIN, HEAD_MASSES, SHAFT_LENGTHS};
use super::wear::{Wear, lives};
use super::why::{Why, explain};
use super::{Hands, Intent, Place};
use crate::craft::matter::{Class, Material, Stuff};

/// One material a player carries, with its numbers and how much of it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stock {
    /// Where it came from.
    pub stuff: Stuff,
    /// Its numbers, treatments included.
    pub material: Material,
    /// Mass carried, in kilograms.
    pub mass: f32,
}

/// One proposed build and everything the player is shown about it.
#[derive(Debug, Clone, PartialEq)]
pub struct Proposal {
    /// Its shape, with material indices into the stock list.
    pub design: Design,
    /// The build itself.
    pub build: Build,
    /// How it swings here.
    pub swing: Swing,
    /// What it does to the reference target.
    pub strike: Strike,
    /// Blows each joint lasts.
    pub lives: [f32; 4],
    /// Why it is good or bad.
    pub why: Vec<Why>,
}

impl Proposal {
    /// Returns the item recipe of this proposal, its materials taken from `stock`.
    pub fn recipe(&self, stock: &[Stock]) -> Recipe {
        let mut stuff = Vec::with_capacity(4);
        let mut design = self.design;
        for slot in design
            .of
            .iter_mut()
            .filter(|i| usize::from(**i) < stock.len())
        {
            let item = stock[usize::from(*slot)].stuff;
            let at = stuff.iter().position(|s| *s == item).unwrap_or_else(|| {
                stuff.push(item);
                stuff.len() - 1
            });
            *slot = at as u8;
        }
        Recipe {
            design,
            stuff,
            wear: Wear::default(),
        }
    }
}

/// The three builds the solver proposes.
#[derive(Debug, Clone, PartialEq)]
pub struct Proposals {
    /// The lightest build that gets through the first layer.
    pub lightest: Proposal,
    /// The build that goes deepest.
    pub hardest: Proposal,
    /// The build whose weakest joint lasts longest.
    pub durable: Proposal,
}

/// A scored combination: its design, mass, depth on the target and life of its weakest joint.
#[derive(Debug, Clone, Copy)]
struct Scored {
    /// The design.
    design: Design,
    /// Mass in kilograms.
    mass: f32,
    /// Depth and leftover energy on the target.
    strike: Strike,
    /// Blows until the weakest joint gives.
    life: f32,
}

/// Returns the index of the best binding in `stock`, sinew or resin, or [`u8::MAX`] for none.
fn binder(stock: &[Stock]) -> u8 {
    let binds = |s: &Stock| {
        matches!(
            s.stuff,
            Stuff::Tissue {
                class: Class::Hide | Class::Resin,
                ..
            }
        )
    };
    (0..stock.len().min(usize::from(u8::MAX)))
        .filter(|&i| binds(&stock[i]))
        .max_by(|&a, &b| {
            stock[a]
                .material
                .strength
                .total_cmp(&stock[b].material.strength)
                .then(b.cmp(&a))
        })
        .map_or(u8::MAX, |i| i as u8)
}

/// Returns the scored design, or nothing when the stock is short of the mass it needs. Above the one
/// hand grip budget the design takes both hands.
fn score(
    mut design: Design,
    stock: &[Stock],
    materials: &[Material],
    place: &Place,
) -> Option<Scored> {
    let build = design.build(materials)?;
    let pieces = build.pieces();
    let (shaft, head) = (usize::from(design.of[0]), usize::from(design.of[1]));
    let shaft_mass = pieces[0].mass + pieces[1].mass;
    let need = |i: usize| {
        let of = |j: usize, m: f32| if i == j { m } else { 0.0 };
        of(shaft, shaft_mass) + of(head, pieces[2].mass)
    };
    if need(shaft) > stock[shaft].mass || need(head) > stock[head].mass {
        return None;
    }
    if place.gravity * super::inertia::moment(&pieces) > Hands::One.budget() {
        design.hands = Hands::Two;
    }
    let build = Build {
        hands: design.hands,
        ..build
    };
    let swung = swing(&build, place);
    let life = lives(&build, &swung)
        .into_iter()
        .fold(f32::INFINITY, f32::min);
    let strike = strike(&blow(&build, design.intent, place), &reference());
    Some(Scored {
        design,
        mass: mass(&pieces),
        strike,
        life,
    })
}

/// Returns the full proposal for a scored design.
fn propose(
    scored: &Scored,
    materials: &[Material],
    place: &Place,
    layers: &[Layer],
) -> Option<Proposal> {
    let build = scored.design.build(materials)?;
    let build = Build {
        hands: scored.design.hands,
        ..build
    };
    let swing = swing(&build, place);
    let lives = lives(&build, &swing);
    let why = explain(&build, &swing, &scored.strike, layers, &lives);
    Some(Proposal {
        design: scored.design,
        build,
        swing,
        strike: scored.strike,
        lives,
        why,
    })
}

/// Returns the first of `all` that is best by `better`, preferring those through the first layer.
fn best(all: &[Scored], better: impl Fn(&Scored, &Scored) -> bool) -> Option<&Scored> {
    let through: Vec<&Scored> = all.iter().filter(|s| s.strike.passed > 0).collect();
    let pool: Vec<&Scored> = if through.is_empty() {
        all.iter().collect()
    } else {
        through
    };
    pool.into_iter()
        .reduce(|kept, next| if better(next, kept) { next } else { kept })
}

/// Returns the proposal `design` makes from `stock` at `place`, as the solver would show it, or
/// nothing when a material is missing or the stock is short of the mass it needs.
pub fn assess(design: Design, stock: &[Stock], place: &Place) -> Option<Proposal> {
    let materials: Vec<Material> = stock.iter().map(|s| s.material).collect();
    let scored = score(design, stock, &materials, place)?;
    propose(&scored, &materials, place, &reference())
}

/// Returns three builds for `intent` from `stock` at `place`, or why none can be made.
pub fn solve(intent: Intent, stock: &[Stock], place: &Place) -> Result<Proposals, Why> {
    let sharp = intent != Intent::Crush;
    let count = stock.len().min(usize::from(u8::MAX));
    let heads: Vec<usize> = (0..count)
        .filter(|&i| !sharp || stock[i].material.hardness >= EDGE_MIN)
        .collect();
    let materials: Vec<Material> = stock.iter().map(|s| s.material).collect();
    let bind = binder(stock);
    let row = intent as usize;
    let mut all = Vec::with_capacity(COMBINATIONS);
    let combos = heads.iter().flat_map(|&h| (0..count).map(move |s| (h, s)));
    let sizes = HEAD_MASSES[row]
        .into_iter()
        .flat_map(|m| SHAFT_LENGTHS[row].map(|l| (m, l)));
    let tries = combos.flat_map(|(h, s)| sizes.clone().map(move |(m, l)| (h, s, m, l)));
    for (h, s, m, l) in tries.take(COMBINATIONS) {
        let design = Design::of(intent, l, m, [s as u8, h as u8, bind]);
        all.extend(score(design, stock, &materials, place));
    }
    let layers = reference();
    let pick = |better: &dyn Fn(&Scored, &Scored) -> bool| {
        best(&all, better)
            .and_then(|s| propose(s, &materials, place, &layers))
            .ok_or(Why::NoEdgeMaterial)
    };
    Ok(Proposals {
        lightest: pick(&|a, b| a.mass < b.mass)?,
        hardest: pick(&|a, b| {
            (a.strike.score(), a.strike.left) > (b.strike.score(), b.strike.left)
        })?,
        durable: pick(&|a, b| a.life > b.life)?,
    })
}
