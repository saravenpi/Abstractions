//! The progression obeys its voice-leading rules.

use crate::audio::music::harmony::{Harmony, SPAN};
use crate::audio::music::mode::{Mode, Scale};
use crate::audio::music::piece::Piece;
use crate::audio::music::roots::related;
use crate::audio::music::voicing::{LEAP, nearest, triad};

/// Every mode a piece can be in.
const MODES: [Mode; 5] = [
    Mode::Lydian,
    Mode::Ionian,
    Mode::Dorian,
    Mode::Aeolian,
    Mode::Pentatonic,
];

/// Returns the progression of every mode on twelve tonics for forty seeds.
fn progressions() -> impl Iterator<Item = Harmony> {
    (0..40_u64).flat_map(|seed| {
        MODES.into_iter().map(move |mode| {
            let scale = Scale {
                tonic: 36 + (seed % 12) as i32,
                mode,
            };
            Harmony::new(scale, [17, 20, 24][(seed % 3) as usize], seed * 7919 + 1)
        })
    })
}

/// Consecutive chords always share a tone and their roots are a third or a fourth apart.
#[test]
fn chords_share_a_tone() {
    for mut harmony in progressions() {
        let scale = harmony.scale();
        assert!(harmony.chords() >= 4, "the loop fell back to one chord");
        for _ in 0..64 {
            let (before, after) = (harmony.chord(), harmony.advance());
            let (old, new) = (triad(scale, before.degree), triad(scale, after.degree));
            assert!(old.iter().any(|class| new.contains(class)));
            assert!(related(scale, before.degree, after.degree));
            assert!(scale.consonant(after.degree));
        }
    }
}

/// No voice ever leaps more than a fifth, and every voice stays in its register for ever.
#[test]
fn voices_move_by_small_steps() {
    for mut harmony in progressions() {
        let homes = harmony.homes();
        for _ in 0..400 {
            let (before, after) = (harmony.chord(), harmony.advance());
            for (voice, home) in homes.iter().enumerate() {
                assert!((after.notes[voice] - before.notes[voice]).abs() <= LEAP);
                assert!((after.notes[voice] - home).abs() <= SPAN);
            }
        }
    }
}

/// Every voice sounds a note of its chord: the bass the root, the upper voices the whole triad.
#[test]
fn voices_spell_the_chord() {
    for mut harmony in progressions() {
        let scale = harmony.scale();
        for _ in 0..32 {
            let chord = harmony.advance();
            let classes = triad(scale, chord.degree);
            let class = |voice: usize| chord.notes[voice].rem_euclid(12);
            assert_eq!(class(0), classes[0]);
            let mut upper = [class(1), class(2), class(3)];
            let mut triad = classes;
            upper.sort_unstable();
            triad.sort_unstable();
            assert_eq!(upper, triad);
        }
    }
}

/// The nearest note of a pitch class is never more than the reach away.
#[test]
fn nearest_note_stays_in_reach() {
    for (class, prev) in (0..12).flat_map(|class| (30..70).map(move |prev| (class, prev))) {
        for (home, reach) in [(40, 6), (60, 6), (40, 7), (60, 7)] {
            let note = nearest(class, prev, home, reach);
            assert_eq!(note.rem_euclid(12), class);
            assert!((note - prev).abs() <= reach);
        }
    }
}

/// The same seed gives the same piece, other seeds give other pieces, and home is warm.
#[test]
fn pieces_are_functions_of_the_seed() {
    let pieces: Vec<Piece> = (1..=20).map(|seed| Piece::new(seed, false)).collect();
    for (i, piece) in pieces.iter().enumerate() {
        assert_eq!(*piece, Piece::new(i as u64 + 1, false));
        assert!(pieces[..i].iter().all(|other| other != piece));
        assert!((12.0..=24.0).contains(&piece.pace.chord));
        assert!(piece.pace.fade * 2.0 <= piece.pace.chord + 4.0);
    }
    let modes = MODES.map(|mode| {
        pieces
            .iter()
            .filter(|piece| piece.scale.mode == mode)
            .count()
    });
    assert!(modes.iter().filter(|&&count| count > 0).count() >= 4);
    for seed in 0..50 {
        assert_eq!(Piece::new(seed, true).scale.mode, Mode::Ionian);
    }
}
