//! Tests of the close codes: every reason goes and comes back, other codes stand for none.

use super::close::{BROKEN, BYE, LAGGING, code, reason};
use crate::wire::Reason;

/// Every reason survives the trip through its code, and no two share one.
#[test]
fn every_reason_has_its_own_code() {
    let all = [
        Reason::WrongPassword,
        Reason::Full,
        Reason::Outdated,
        Reason::Newer,
        Reason::Flooding,
        Reason::Closing,
        Reason::Replaced,
        Reason::Silent,
    ];
    for one in all {
        assert_eq!(reason(code(one)), Some(one));
    }
}

/// The codes of a normal close and of transport trouble stand for no reason.
#[test]
fn other_codes_stand_for_no_reason() {
    for other in [BYE, LAGGING, BROKEN] {
        assert_eq!(reason(other), None);
    }
}
