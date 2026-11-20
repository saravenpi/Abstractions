//! Tests of the solver: its three picks, its determinism, its refusals and its budget.

use std::time::Instant;

use super::rig::{earth, stock, tissue};
use super::*;
use crate::craft::matter::Class;

/// Each pick is best at its own measure, and every intent gets three proposals from the stock.
#[test]
fn proposals_are_best_at_their_measure() {
    for intent in [Intent::Cut, Intent::Pierce, Intent::Crush, Intent::Throw] {
        let found = solve(intent, &stock(), &earth()).expect("the stock builds something");
        let all = [&found.lightest, &found.hardest, &found.durable];
        let mass = |p: &Proposal| super::inertia::mass(&p.build.pieces());
        let life = |p: &Proposal| p.lives.into_iter().fold(f32::INFINITY, f32::min);
        assert!(
            all.iter().all(|p| mass(&found.lightest) <= mass(p) + 1e-6),
            "{intent:?}"
        );
        assert!(
            all.iter()
                .all(|p| found.hardest.strike.score() >= p.strike.score()),
            "{intent:?}"
        );
        assert!(
            all.iter().all(|p| life(&found.durable) >= life(p)),
            "{intent:?}"
        );
        assert!(
            found.hardest.strike.passed > 0,
            "{intent:?}: {:?}",
            found.hardest.strike
        );
        assert!(
            all.iter()
                .all(|p| p.why.iter().any(|w| matches!(w, Why::WeakestJoint(_))))
        );
    }
}

/// The same request gives the same answer.
#[test]
fn the_solver_is_deterministic() {
    assert_eq!(
        solve(Intent::Cut, &stock(), &earth()),
        solve(Intent::Cut, &stock(), &earth())
    );
}

/// With only hide carried, nothing holds an edge, but a club can still be made.
#[test]
fn soft_stock_holds_no_edge() {
    let soft = [tissue(Class::Hide, 0, 5.0)];
    assert_eq!(
        solve(Intent::Cut, &soft, &earth()),
        Err(Why::NoEdgeMaterial)
    );
    assert_eq!(solve(Intent::Cut, &[], &earth()), Err(Why::NoEdgeMaterial));
    assert!(solve(Intent::Crush, &soft, &earth()).is_ok());
}

/// A request that scores the full 200 combinations stays under 5 ms.
#[test]
fn the_solver_is_fast() {
    let mut big = stock();
    big.extend(stock());
    solve(Intent::Cut, &big, &earth()).ok();
    let start = Instant::now();
    let rounds = 20;
    for _ in 0..rounds {
        solve(Intent::Cut, &big, &earth()).ok();
    }
    let each = start.elapsed() / rounds;
    eprintln!("solver: {each:?} per request");
    assert!(each.as_secs_f64() < 0.005, "{each:?}");
}
