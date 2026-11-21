//! Tests that every action of the screen shows its key: each button names its key, no two actions
//! of a pane share one, and every move the keys make is in the legend of its pane.

use super::bench::Pane;
use super::click::Press;
use super::keys::{MAKE_BINDS, binds, moves};

/// Every action of each pane has a word and a key name on its button, and its own key.
#[test]
fn every_action_shows_its_own_key() {
    for pane in [Pane::Pack, Pane::Workshop] {
        let all = binds(pane);
        assert!(!all.is_empty(), "{pane:?}");
        for bind in all {
            assert!(!bind.label.is_empty() && !bind.key.is_empty(), "{bind:?}");
        }
        for (at, bind) in all.iter().enumerate() {
            let twin = all[at + 1..]
                .iter()
                .any(|other| other.code == bind.code || other.key == bind.key);
            assert!(!twin, "{pane:?}: {} shares its key", bind.label);
        }
    }
}

/// Make is the first action of the workshop, on Enter, so its button leads the bar.
#[test]
fn make_leads_the_workshop() {
    assert_eq!(MAKE_BINDS[0].press, Press::Make);
    assert_eq!(MAKE_BINDS[0].key, "Enter");
}

/// The legend of each pane names Tab, the arrows it reads and the number keys of the hotbar, each
/// with a word, and no action of the pane is missing a way to be reached.
#[test]
fn every_move_is_in_the_legend() {
    let wanted = [
        (Pane::Pack, ["Tab", "Up Down", "1-6"].as_slice()),
        (
            Pane::Workshop,
            ["Tab", "Left Right", "Up Down", "1-6"].as_slice(),
        ),
    ];
    for (pane, keys) in wanted {
        let legend = moves(pane);
        assert!(
            legend
                .iter()
                .all(|(words, key)| !words.is_empty() && !key.is_empty())
        );
        for key in keys {
            assert!(
                legend.iter().any(|(_, named)| named == key),
                "{pane:?}: {key}"
            );
        }
    }
}
