//! Tests of the singing: a call moves only its caller, or the whole herd for a herd cry, for as long
//! as it lasts, and old calls are forgotten.

use super::sing::Sung;
use crate::life::signs::calls::{Calling, Cry, Rhythm};

/// Returns a call of herd 9, member 2, for `cry` at universe time 100.
fn calling(cry: Cry) -> Calling {
    Calling {
        herd: 9,
        species: 1,
        slot: 2,
        cry,
        at: 100.0,
    }
}

/// Returns a rhythm of two notes of half a second.
fn beat(_: &Calling) -> Option<Rhythm> {
    Some(Rhythm {
        note: 0.5,
        gap: 0.1,
        notes: 2,
    })
}

/// A contact call moves its caller only, an alarm the whole herd, and neither after it ends.
#[test]
fn calls_move_their_callers() {
    let mut sung = Sung::default();
    sung.remember(100.0, [calling(Cry::Contact)].into_iter(), beat);
    assert!(
        sung.song((9, 2), 100.3)
            .is_some_and(|s| (s.age - 0.3).abs() < 1e-4)
    );
    assert!(sung.song((9, 3), 100.3).is_none());
    assert!(sung.song((8, 2), 100.3).is_none());
    assert!(sung.song((9, 2), 102.0).is_none());
    sung.remember(100.0, [calling(Cry::Alarm)].into_iter(), beat);
    assert!(
        sung.song((9, 3), 100.3)
            .is_some_and(|s| s.cry == Cry::Alarm)
    );
    sung.remember(110.0, std::iter::empty(), beat);
    assert!(sung.calls.is_empty());
}
