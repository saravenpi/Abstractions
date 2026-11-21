//! Tests of the pack list: rows for tools and stacks grouped by kind, and the screen opening and
//! closing on its keys.

use super::Workshop;
use super::kind::Kind;
use super::lines::{What, lines};
use super::open::next_state;
use crate::craft::build::{Intent, solve};
use crate::craft::stage::{earth, pack};

/// Tools come first and show their wear and their intent; the materials follow in groups, in the
/// order of the kinds, each group heaviest first, with their grade.
#[test]
fn rows_are_grouped_by_kind() {
    let mut pack = pack();
    let stock = pack.stock();
    let made = solve(Intent::Cut, &stock, &earth()).expect("a cut");
    let id = pack.make(&made.hardest, &stock).expect("made");
    let all = lines(&pack, &earth(), None);
    assert_eq!(all[0].what, What::Item(id));
    assert_eq!((all[0].kind, all[0].wear), (Kind::Tool, Some(0.0)));
    assert!(all[0].icon.is_some() && all[0].shape.is_some());
    assert!(
        all.iter()
            .skip(1)
            .all(|l| l.wear.is_none() && l.icon.is_none())
    );
    assert!(all.windows(2).all(|w| w[0].kind <= w[1].kind));
    let stacks = &all[1..];
    assert!(
        stacks
            .windows(2)
            .all(|w| w[0].kind != w[1].kind || w[0].mass >= w[1].mass)
    );
    assert!(
        all.iter()
            .any(|l| l.kind == Kind::Stone && l.signature == "stone")
    );
    assert!(
        all.iter()
            .any(|l| l.kind == Kind::Hide && l.signature == "clean")
    );
    assert!(all.iter().any(|l| l.kind == Kind::Wood) && all.iter().any(|l| l.kind == Kind::Bone));
}

/// The key opens the screen where it can be open; the key, Escape or losing the ground close it.
#[test]
fn the_screen_opens_and_closes() {
    assert_eq!(next_state(false, true, (true, false)), Some(Workshop::Open));
    assert_eq!(next_state(false, false, (true, false)), None);
    assert_eq!(
        next_state(true, true, (false, true)),
        Some(Workshop::Closed)
    );
    assert_eq!(
        next_state(true, false, (false, false)),
        Some(Workshop::Closed)
    );
    assert_eq!(next_state(true, true, (false, false)), None);
}
