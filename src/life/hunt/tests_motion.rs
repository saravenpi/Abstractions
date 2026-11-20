//! Tests of herds under hunting: smooth chases.

use super::tests::{RADIUS, herd, hunts};
use crate::life::herd::fixtures::meadow_life;
use crate::life::herd::{Herd, arc};

/// The hunter herd moves continuously through the start of the stalk, the chase, the end of the feed
/// and its return to the route.
#[test]
fn chases_are_continuous() {
    let life = meadow_life();
    let hunt = hunts(&life).into_iter().next().expect("a hunt");
    let chase = hunt.start + hunt.stalk;
    let fed = hunt.outcome_time() + hunt.feed;
    let spans = [
        (hunt.start - 1.0, 3.0),
        (chase - 1.0, hunt.chase + 2.0),
        (fed - 1.0, 3.0),
        (hunt.end() - 1.0, 3.0),
    ];
    for (from, length) in spans {
        let mut last: Option<Herd> = None;
        for step in 0..=(length / 0.1) as u32 {
            let time = from + 0.1 * f64::from(step);
            let now = herd(&life, hunt.hunter, hunt.from, time).expect("the hunter");
            if let Some(before) = last {
                let jump = arc(before.centre, now.centre) * RADIUS;
                let bound = f64::from(before.speed.max(now.speed)) * 0.1 * 1.05 + 0.01;
                assert!(jump <= bound, "jumped {jump} m at {time} s (bound {bound})");
            }
            last = Some(now);
        }
    }
}
