//! Tests of the four outcomes of a swing: each is chosen from its inputs, and each leaves the tool,
//! the hitstop and the animal different.

use super::act::{Act, Timing};
use super::outcome::{Contact, Outcome, after, classify, rebound};
use super::stage::blade;

/// Returns a contact.
fn touch(blunt: bool, refused: bool, cos: f32) -> Option<Contact> {
    Some(Contact {
        blunt,
        refused,
        cos,
    })
}

/// Every swing result is chosen from the contact it had.
#[test]
fn each_outcome_is_chosen_from_its_inputs() {
    let table = [
        (None, Outcome::Miss),
        (touch(false, false, 0.9), Outcome::Clean),
        (touch(false, false, 0.2), Outcome::Deflected),
        (touch(false, true, 0.9), Outcome::Deflected),
        (touch(true, false, 0.9), Outcome::Stopped),
        (touch(true, true, 0.9), Outcome::Stopped),
        (touch(true, false, 0.2), Outcome::Deflected),
    ];
    for (contact, want) in table {
        assert_eq!(classify(contact), want, "{contact:?}");
    }
}

/// Each outcome gives the hitstop, the camera and the animal its own share: none for a miss, short and
/// light for a glance, full for a clean or a stopped blow.
#[test]
fn each_outcome_reads_differently() {
    assert_eq!(Outcome::Miss.hitstop(), 0.0);
    assert!(Outcome::Deflected.hitstop() < Outcome::Clean.hitstop());
    assert_eq!(Outcome::Clean.hitstop(), Outcome::Stopped.hitstop());
    assert!(Outcome::Deflected.kick() < Outcome::Clean.kick());
    assert_eq!((Outcome::Miss.harm(), Outcome::Clean.harm()), (0.0, 1.0));
    assert!(Outcome::Deflected.harm() > 0.0 && Outcome::Deflected.harm() < 0.5);
}

/// A miss recovers longest and carries on furthest, a stopped tool not at all, and the rest in between.
#[test]
fn a_miss_carries_on_and_recovers_longer() {
    let held = blade();
    let path = super::path::Path::of(held.style, super::dir::Dir::Right);
    let base = Timing::of(&held, &path);
    let miss = base.braked(Outcome::Miss);
    for other in [Outcome::Clean, Outcome::Deflected, Outcome::Stopped] {
        let t = base.braked(other);
        assert!(miss.recover > t.recover && miss.over > t.over, "{other:?}");
    }
    assert_eq!(base.braked(Outcome::Stopped).over, 0.0);
    assert!(base.braked(Outcome::Deflected).over > base.braked(Outcome::Clean).over);
}

/// The tool after a contact: a clean blow stuns for the kickback, a stopped one stops dead and is set
/// back along the path, a glance and a miss carry on.
#[test]
fn the_tool_after_contact_differs() {
    assert_eq!(after(Outcome::Clean, 0.0, 0.5, 0.3), None);
    assert_eq!(
        after(Outcome::Clean, 0.05, 0.5, 0.3),
        Some(Act::Stun(0.05, 0.5))
    );
    let Some(Act::Stun(left, at)) = after(Outcome::Stopped, 0.0, 0.5, 0.3) else {
        panic!("a stopped tool holds");
    };
    assert!(left > 0.0 && at < 0.5);
    assert_eq!(after(Outcome::Deflected, 0.1, 0.5, 0.3), None);
    assert_eq!(after(Outcome::Miss, 0.1, 0.5, 0.3), None);
}

/// Hard contacts bounce more than soft ones.
#[test]
fn hard_things_bounce() {
    assert!(rebound(17.0, 30.0) > 0.3 && rebound(0.1, 30.0) < 0.05);
}
