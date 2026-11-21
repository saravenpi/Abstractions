//! Key hint tests: a hint is two texts, the label then the key, the key in the secondary text
//! colour and the set gap away, and a legend keeps its hints apart by the wider gap.

use bevy::prelude::*;

use super::keys::{HINT_GAP, HintLabel, KEY_GAP, KeyName, Legend, hints, legend};
use super::style::{Look, Tone, shade};
use super::theme;

/// Every entry of a legend is two texts: what the key does in the primary text colour, then the
/// name of the key in the secondary one, with the gap of a key between them, and the entries are
/// the gap of a legend apart.
#[test]
fn every_legend_entry_is_a_label_then_a_muted_key() {
    let pairs = [("Go", "G"), ("Zoom or scroll", "Wheel"), ("Close", "Esc")];
    let mut world = World::new();
    let row = world.spawn(legend(hints(&pairs))).id();
    assert!(world.get::<Legend>(row).is_some());
    assert_eq!(world.get::<Node>(row).unwrap().column_gap, px(HINT_GAP));
    let entries = world.get::<Children>(row).unwrap().to_vec();
    assert_eq!(entries.len(), pairs.len());
    for (entry, (words, key)) in entries.into_iter().zip(pairs) {
        assert_eq!(world.get::<Node>(entry).unwrap().column_gap, px(KEY_GAP));
        let parts = world.get::<Children>(entry).unwrap().to_vec();
        assert_eq!(parts.len(), 2);
        let said = |part: Entity| world.get::<Text>(part).unwrap().0.clone();
        let colour = |part: Entity| world.get::<TextColor>(part).unwrap().0;
        assert!(world.get::<HintLabel>(parts[0]).is_some());
        assert_eq!(
            (said(parts[0]), colour(parts[0])),
            (words.to_string(), theme::TEXT)
        );
        assert!(world.get::<KeyName>(parts[1]).is_some());
        assert_eq!(
            (said(parts[1]), colour(parts[1])),
            (key.to_string(), theme::TEXT_2)
        );
        assert_eq!(world.get::<Tone>(parts[1]), Some(&Tone::Key));
        assert!(!said(parts[1]).contains('['));
    }
    assert!(KEY_GAP >= theme::S3 && HINT_GAP > KEY_GAP);
}

/// The key of a button keeps the secondary text colour whatever the look of the button and
/// whether it is selected, so it never competes with the label, and never the colour of a
/// disabled text.
#[test]
fn a_key_stays_muted_in_every_look() {
    for look in [Look::Rest, Look::Active, Look::Pressed] {
        for selected in [false, true] {
            assert_eq!(shade(Tone::Key, look, selected), theme::TEXT_2);
        }
    }
    assert_ne!(theme::TEXT_2, theme::TEXT_OFF);
}
