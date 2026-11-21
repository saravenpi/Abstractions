//! The workshop's offer: the solver's three proposals for the intent, the focused one with its
//! swaps, the names they are shown by and what each takes from the pack.

use super::bench::Bench;
use crate::craft::build::{Place, Why};
use crate::craft::build::{Proposal, Proposals, Stock, assess, solve};
use crate::craft::pack::{called, cost, noun};
use crate::ui::units::fixed;

/// The names of the three proposals, in pick order.
pub const KINDS: [&str; 3] = ["Lightest", "Hardest hitting", "Most durable"];

/// What the workshop shows.
#[derive(Debug, Clone, PartialEq)]
pub struct Offer {
    /// The three proposals, in pick order.
    pub three: [Proposal; 3],
    /// The focused proposal after its swaps, or nothing when the swap cannot be made.
    pub chosen: Option<Proposal>,
}

/// Returns the offer for `bench` from `stock` at `place`, or why nothing can be made.
pub fn offer(bench: &Bench, stock: &[Stock], place: &Place) -> Result<Offer, Why> {
    let Proposals {
        lightest,
        hardest,
        durable,
    } = solve(bench.intent, stock, place)?;
    let three = [lightest, hardest, durable];
    let base = &three[bench.pick.min(2)];
    let chosen = match bench.swap {
        [0, 0] => Some(base.clone()),
        swap => {
            let mut design = base.design;
            let turn = |i: u8, by: usize| ((usize::from(i) + by) % stock.len().max(1)) as u8;
            design.of[0] = turn(design.of[0], swap[0]);
            design.of[1] = turn(design.of[1], swap[1]);
            assess(design, stock, place)
        }
    };
    Ok(Offer { three, chosen })
}

/// Returns the name of `proposal`: its head material from `stock` and the word for its shape.
pub fn title(proposal: &Proposal, stock: &[Stock]) -> String {
    let build = &proposal.build;
    let total = crate::craft::build::mass(&build.pieces()).max(f32::EPSILON);
    let word = noun(
        proposal.design.intent,
        build.head.mass / total,
        build.reach(),
    );
    let head = stock.get(usize::from(proposal.design.of[1]));
    format!("{} {word}", head.map_or("bare", |s| called(&s.stuff)))
}

/// Returns what `proposal` takes from `stock` in words: each material and its mass, head first.
pub fn parts(proposal: &Proposal, stock: &[Stock]) -> String {
    let mut taken = cost(proposal, stock);
    let head = usize::from(proposal.design.of[1]);
    taken.sort_by_key(|(at, _)| *at != head);
    let words: Vec<String> = taken
        .iter()
        .map(|(at, kg)| format!("{} {} kg", called(&stock[*at].stuff), fixed(*kg, 2)))
        .collect();
    format!("Uses {}", words.join(", "))
}
