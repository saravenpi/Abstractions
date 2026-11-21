//! Tests of the pace of blows: a knife slash, a spear stab and a club chop each take as long as a
//! person's from wind-up to carrying again, a miss a little longer, and a click while the tool comes
//! back starts the next blow at once.

use super::act::{Act, Ask, Timing, next};
use super::dir::Dir;
use super::held::Held;
use super::outcome::Outcome;
use super::path::Path;
use super::stage::{club, knife, spear};

/// Returns the seconds a blow of `held` from `dir` takes from the click to carrying again, when
/// it comes to `outcome`.
fn total(held: &Held, dir: Dir, outcome: Outcome) -> f32 {
    let timing = Timing::of(held, &Path::of(held.style, dir)).braked(outcome);
    timing.windup + timing.swing + timing.recover
}

/// A knife slash takes 0.25 to 0.35 s, a spear stab 0.3 to 0.4 s and a club chop 0.45 to 0.6 s,
/// within a few hundredths, whether they land or miss.
#[test]
fn blows_take_human_times() {
    let cases = [
        (knife(), Dir::Right, (0.25, 0.35)),
        (spear(), Dir::Right, (0.3, 0.4)),
        (club(), Dir::Up, (0.45, 0.6)),
    ];
    for (held, dir, (low, high)) in cases {
        for outcome in [Outcome::Clean, Outcome::Miss] {
            let seconds = total(&held, dir, outcome);
            assert!(
                (low - 0.03..=high + 0.03).contains(&seconds),
                "{:?} {outcome:?}: {seconds} s",
                held.style
            );
        }
        assert!(total(&held, dir, Outcome::Miss) > total(&held, dir, Outcome::Clean));
    }
}

/// A click while the tool comes back after a blow winds the next one up at once.
#[test]
fn a_click_cuts_the_recovery_short() {
    let held = knife();
    let timing = Timing::of(&held, &Path::of(held.style, Dir::Right));
    let click = Ask {
        strike: true,
        ..Default::default()
    };
    let (act, _) = next(Act::Recover(0.05), &click, &timing, false, 1.0 / 60.0);
    assert_eq!(act, Act::Windup(0.0));
    let (act, _) = next(
        Act::Recover(0.05),
        &Ask::default(),
        &timing,
        false,
        1.0 / 60.0,
    );
    assert!(matches!(act, Act::Recover(_)), "{act:?}");
}
