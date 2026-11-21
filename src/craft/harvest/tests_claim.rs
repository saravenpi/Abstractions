//! Tests of the claims on the parts of a carcass: alone a cut is harvested at once; in a session it
//! waits, a grant gives it once, a refusal gives nothing, and the end of the session gives back
//! what was never answered.

use super::{Claims, Cut};
use crate::craft::matter::Grade;

/// Returns the cut of part `part` of species 3, cleanly killed.
fn cut(part: u8) -> Cut {
    Cut {
        species: 3,
        part,
        kill: Grade::Clean,
    }
}

/// Alone a cut is harvested at once and nothing waits.
#[test]
fn alone_a_cut_is_harvested_at_once() {
    let mut claims = Claims::default();
    assert_eq!(claims.claim((9, 4), cut(4)), Some(cut(4)));
    assert!(claims.pending.is_empty());
}

/// In a session a cut waits for the server: its answer takes it out once, so two answers for the
/// same part give one piece, and the end of the session gives back the cuts never answered.
#[test]
fn in_a_session_a_cut_waits_for_the_server() {
    let mut claims = Claims {
        remote: true,
        ..Claims::default()
    };
    assert_eq!(claims.claim((9, 4), cut(4)), None);
    assert_eq!(claims.claim((9, 5), cut(5)), None);
    assert_eq!(claims.answered((9, 4)), Some(cut(4)));
    assert_eq!(claims.answered((9, 4)), None);
    assert_eq!(claims.answered((8, 5)), None);
    assert_eq!(claims.release(), [cut(5)]);
    assert!(!claims.remote && claims.pending.is_empty());
}
