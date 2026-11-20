//! Tests of the pack list: rows for tools and stacks, the filter and the orders, and the screen
//! opening and closing on its keys.

use super::Workshop;
use super::bench::Sort;
use super::lines::{What, arrange, lines};
use super::open::next_state;
use crate::craft::build::{Intent, solve};
use crate::craft::stage::{earth, pack};

/// Tools come first and show their wear; stacks follow with their grade; the filter reads names
/// and signatures; the orders sort by name and by mass.
#[test]
fn rows_filter_and_sort() {
    let mut pack = pack();
    let stock = pack.stock();
    let made = solve(Intent::Cut, &stock, &earth()).expect("a cut");
    let id = pack.make(&made.hardest, &stock).expect("made");
    let all = lines(&pack, &earth(), None);
    assert_eq!(all[0].what, What::Item(id));
    assert_eq!(all[0].wear, Some(0.0));
    assert!(all[0].shape.is_some());
    assert!(all.iter().skip(1).all(|l| l.wear.is_none()));
    assert!(all.iter().any(|l| l.signature == "stone"));
    assert!(all.iter().any(|l| l.signature == "clean"));
    let flint = arrange(all.clone(), "FLI", Sort::Kept);
    assert!(!flint.is_empty() && flint.iter().all(|l| format!("{l:?}").contains("flint")));
    let named = arrange(all.clone(), "", Sort::Name);
    assert!(
        named
            .windows(2)
            .all(|w| w[0].name.to_lowercase() <= w[1].name.to_lowercase())
    );
    let heavy = arrange(all, "", Sort::Mass);
    assert!(heavy.windows(2).all(|w| w[0].mass >= w[1].mass));
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
