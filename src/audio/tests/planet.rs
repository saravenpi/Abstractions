//! What each kind of planet sounds like: its air, its shore and the mood of its piece.

use super::{CLICK, Stats, listen, mixer};
use crate::audio::music::mode::Mode;
use crate::audio::music::mood::mood;
use crate::audio::music::piece::Piece;
use crate::audio::planet::{Surf, class, code, loudness, surf};
use crate::audio::scene::AudioScene;
use crate::settings::Audio as Volumes;
use crate::universe::{PlanetClass, PlanetRecord, Water};
use crate::wind::{PRESSURES, samples};

/// Waves need liquid water and a sea, molten rock rumbles, and ice, steam and dry ground are
/// silent: one answer for each of the six classes.
#[test]
fn the_shore_sounds_as_the_liquid_of_each_class() {
    use Surf::{Molten, Silent, Waves};
    let heard = samples().map(|record| surf(&record));
    assert_eq!(heard, [Waves, Waves, Silent, Silent, Molten, Silent]);
    assert_eq!(surf(&PlanetRecord::TEMPERATE), Waves);
    for record in samples() {
        let dry = PlanetRecord {
            ocean: 0.0,
            ..record
        };
        assert_eq!(surf(&dry), Silent, "{:?}", record.class);
        for water in [Water::None, Water::Ice, Water::Steam] {
            let other = surf(&PlanetRecord { water, ..record });
            let lava = record.class == PlanetClass::Lava && record.ocean > 0.0;
            assert_eq!(other, if lava { Molten } else { Silent });
        }
    }
}

/// The air is silent without air, as loud as at home from the home pressure up, and never
/// quieter as the pressure grows.
#[test]
fn the_air_is_as_loud_as_it_is_thick() {
    let heard = PRESSURES.map(|pressure| {
        loudness(&PlanetRecord {
            pressure,
            ..PlanetRecord::TEMPERATE
        })
    });
    assert_eq!(heard, [0.0, 0.0, 0.3, 1.0, 1.0]);
    let classes = samples().map(|record| loudness(&record));
    assert_eq!(classes, [1.0, 1.0, 0.4, 0.7, 1.0, 0.0]);
}

/// Every class crosses to the audio thread and back, and a number that is no class is home.
#[test]
fn the_class_survives_the_audio_bridge() {
    for kind in PlanetClass::ALL {
        assert_eq!(class(code(kind)), kind);
    }
    assert_eq!(class(f32::NAN), PlanetClass::Island);
    assert_eq!(class(77.0), PlanetClass::Island);
    assert_eq!(AudioScene::default().class, PlanetClass::Island);
}

/// An island world keeps the piece of its seed bit for bit. Every other class keeps its tonic
/// and tempo, an ice world is brighter than a lava world, and the dry classes each have their
/// own mode.
#[test]
fn the_class_sets_the_mood_of_the_piece() {
    for seed in 0..40 {
        for home in [false, true] {
            let drawn = Piece::new(seed, home);
            assert_eq!(Piece::of(seed, home, PlanetClass::Island), drawn);
            let [ocean, _, desert, ice, lava, barren] =
                PlanetClass::ALL.map(|kind| Piece::of(seed, home, kind));
            assert!(ice.timbre.bright > ocean.timbre.bright);
            assert!(ocean.timbre.bright > drawn.timbre.bright);
            assert!(drawn.timbre.bright > desert.timbre.bright);
            assert!(barren.timbre.bright > lava.timbre.bright);
            assert_eq!(
                (ice.scale.mode, lava.scale.mode),
                (Mode::Lydian, Mode::Aeolian)
            );
            assert_eq!(desert.scale.mode, Mode::Dorian);
            assert_eq!(barren.scale.mode, Mode::Pentatonic);
            assert_eq!(
                (lava.scale.tonic, lava.pace),
                (drawn.scale.tonic, drawn.pace)
            );
        }
    }
    assert_eq!(mood(PlanetClass::Island).bright, 1.0);
}

/// Renders twenty seconds of the shore of a planet of `kind`, with no music.
fn shore(kind: PlanetClass) -> Stats {
    let scene = AudioScene {
        class: kind,
        shore: 1.0,
        altitude_fade: 0.0,
        ..AudioScene::default()
    };
    let (mut mixer, _shared) = mixer(&scene);
    let stats = listen(&mut mixer, 20.0);
    assert!(!stats.broken && stats.peak <= 1.0);
    stats
}

/// A lava shore is a rumble: heard, but with far smaller steps from one sample to the next
/// than the wash of water, which is what a sound with nothing above the bass looks like.
#[test]
fn a_molten_shore_rumbles_where_water_washes() {
    let (water, lava) = (shore(PlanetClass::Island), shore(PlanetClass::Lava));
    assert!(water.rms() > 0.005 && lava.rms() > 0.005);
    let rough = |stats: &Stats| f64::from(stats.jump) / stats.rms();
    assert!(
        rough(&lava) < 0.5 * rough(&water),
        "{} {}",
        rough(&lava),
        rough(&water)
    );
    assert_eq!(shore(PlanetClass::Ocean).rms(), water.rms());
}

/// Every class plays its piece without a broken sample or a click, also across a change of
/// class on the same planet.
#[test]
fn every_class_plays_cleanly() {
    let scene = |kind| AudioScene {
        class: kind,
        ..super::planet(11)
    };
    let (mut mixer, shared) = mixer(&scene(PlanetClass::Island));
    for kind in PlanetClass::ALL {
        shared.publish(&scene(kind), &Volumes::default());
        let stats = listen(&mut mixer, 12.0);
        assert!(!stats.broken && stats.jump < CLICK, "{kind:?}");
        assert!(stats.rms() > 0.003, "{kind:?} is silent");
    }
}
