//! Tests of the leaf set: merging neighbours, runs, bad keys and the size of a long walk.

use super::{Leaves, Run, key, valid};

/// A run that only contains its own leaves never reports one it did not get.
#[test]
fn leaves_merge_into_runs_and_count_once() {
    let mut set = Leaves::default();
    let base = key(2, 10, 100);
    assert!(set.insert(base) && set.insert(base + 2) && set.insert(base + 1));
    assert!(!set.insert(base + 1));
    assert_eq!((set.len(), set.run_count()), (3, 1));
    assert!(set.contains(base + 2) && !set.contains(base + 3) && !set.contains(base - 1));
}

/// Runs come back as the same set, and a later merge returns only what was new.
#[test]
fn runs_round_trip_and_merge_reports_the_new_leaves() {
    let mut set = Leaves::default();
    for morton in [5, 6, 7, 40, 41, 900] {
        set.insert(key(0, 8, morton));
    }
    assert_eq!(Leaves::from_runs(&set.runs()), set);
    let mut other = Leaves::default();
    for morton in [6, 7, 8, 900, 901] {
        other.insert(key(0, 8, morton));
    }
    let fresh = set.merge(&other);
    assert_eq!(fresh.len(), 2);
    assert!(fresh.contains(key(0, 8, 8)) && fresh.contains(key(0, 8, 901)));
    assert_eq!(set.len(), 8);
}

/// Chunks of runs each stand alone, and together hold the whole set.
#[test]
fn chunks_stand_alone_and_add_up() {
    let mut set = Leaves::default();
    for morton in (0..40).step_by(3) {
        set.insert(key(4, 9, morton));
    }
    let chunks = set.chunks(5);
    assert_eq!(chunks.iter().map(Vec::len).collect::<Vec<_>>(), [5, 5, 4]);
    let mut back = Leaves::default();
    for chunk in &chunks {
        back.merge(&Leaves::from_runs(chunk));
    }
    assert_eq!(back, set);
    assert_eq!(set.iter().count() as u64, set.len());
}

/// Keys that name no leaf are refused, alone or inside a run.
#[test]
fn keys_that_name_no_leaf_are_refused() {
    assert!(valid(key(5, 27, (1 << 54) - 1)));
    assert!(!valid(key(6, 4, 0)) && !valid(key(0, 0, 0)) && !valid(key(0, 28, 0)));
    assert!(!valid(key(1, 3, 64)) && !valid(u64::MAX));
    let mut set = Leaves::default();
    assert!(!set.insert(key(1, 3, 64)));
    let wide = [Run {
        gap: key(1, 3, 60),
        len: 10,
    }];
    assert!(Leaves::from_runs(&wide).is_empty());
    let overflow = [Run {
        gap: u64::MAX,
        len: 5,
    }];
    assert!(Leaves::from_runs(&overflow).is_empty());
}

/// A straight walk of 10,000 km over leaves of 500 m fits in 64 KB as runs.
#[test]
fn a_ten_thousand_kilometre_walk_stays_under_64_kb() {
    let mut set = Leaves::default();
    for x in 0..20_000u64 {
        set.insert(key(0, 15, spread(x) | (spread(1_000) << 1)));
    }
    let bytes = postcard::to_stdvec(&set.runs()).unwrap();
    assert!(bytes.len() < 64 * 1024, "{} bytes", bytes.len());
}

/// Returns `value` with a zero bit between each of its bits: the even bits of a Morton code.
fn spread(value: u64) -> u64 {
    (0..16).fold(0, |code, bit| code | (((value >> bit) & 1) << (2 * bit)))
}
