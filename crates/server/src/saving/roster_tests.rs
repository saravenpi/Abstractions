//! Tests of the roster: the log lines of joins and leaves.

use abstractions_net::log::PlayerId;
use abstractions_net::wire::PlayerInfo;

use super::roster::Roster;

/// Returns player `id` named `name`.
fn player(id: u32, name: &str) -> PlayerInfo {
    PlayerInfo {
        id: PlayerId(id),
        name: name.to_string(),
    }
}

/// A join and a leave each make one line with the count after them.
#[test]
fn joins_and_leaves_are_logged_with_the_count() {
    let mut roster = Roster::new(16);
    assert_eq!(roster.update(vec![player(1, "Ada")]), ["Ada joined (1/16)"]);
    let both = vec![player(1, "Ada"), player(2, "Bot 1")];
    assert_eq!(roster.update(both.clone()), ["Bot 1 joined (2/16)"]);
    assert!(roster.update(both).is_empty());
    assert_eq!(roster.update(vec![player(2, "Bot 1")]), ["Ada left (1/16)"]);
    assert!(!roster.is_empty());
    assert_eq!(roster.update(Vec::new()), ["Bot 1 left (0/16)"]);
    assert!(roster.is_empty());
}

/// A player replaced by another between two looks makes a leave and a join.
#[test]
fn a_seat_taken_by_someone_else_logs_both() {
    let mut roster = Roster::new(2);
    roster.update(vec![player(1, "Ada")]);
    assert_eq!(
        roster.update(vec![player(3, "Cy")]),
        ["Ada left (1/2)", "Cy joined (1/2)"]
    );
}
