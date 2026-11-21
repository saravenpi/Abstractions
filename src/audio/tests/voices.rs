//! The voices follow the bodies and the cries: a big animal calls lower, with lower formants and
//! slower notes; each body finds its organ; each cry the game models has its own shape; and two
//! animals of one species never sound quite the same.

use super::calls::{calling, render};
use crate::audio::call::Call;
use crate::audio::dsp::rand::Rand;
use crate::audio::throat::{Source, plan};
use crate::life::Species;
use crate::life::herd::fixtures::{role, species};
use crate::life::signs::calls::{Cry, TIMBRES, Timbre, voice};
use crate::life::{Activity, Guild, Habitat, Rung};

/// Returns the call of `cry` of `species`, from ten metres ahead at full level.
fn heard(species: &Species, cry: Cry) -> Call {
    Call {
        voice: voice(species, Rung::Skeletal),
        cry,
        one: 0,
        side: 0.0,
        near: 1.0,
        metres: 10.0,
    }
}

/// Returns how bright `sound` is: the root mean square of its slope over its own, which grows with
/// the frequency it carries most.
fn brightness(sound: &[f32]) -> f64 {
    let slope: Vec<f32> = sound.windows(2).map(|w| w[1] - w[0]).collect();
    super::rig::rms(&slope) / super::rig::rms(sound).max(1e-9)
}

/// A huge, long grazer calls lower, through lower formants and with longer notes, than a small,
/// short one, and sounds darker.
#[test]
fn big_animals_call_lower_and_longer() {
    let grazer = role(Guild::Grazer, Habitat::Ground, Activity::Day, 2.0);
    let mut small = species(1, grazer, 5.0);
    small.body.length = 0.4;
    let mut huge = species(2, grazer, 3000.0);
    huge.body.length = 6.0;
    let mut rand = Rand::new(5, 0);
    let (a, b) = (
        plan(&heard(&small, Cry::Contact), &mut rand),
        plan(&heard(&huge, Cry::Contact), &mut rand),
    );
    assert!(b.hz < 0.5 * a.hz, "{} {}", b.hz, a.hz);
    assert!(
        b.formants[0] < 0.5 * a.formants[0],
        "{:?} {:?}",
        b.formants,
        a.formants
    );
    assert!(b.seconds() > a.seconds());
    let (low, high) = (
        render(heard(&huge, Cry::Contact), 6.0),
        render(heard(&small, Cry::Contact), 6.0),
    );
    assert!(brightness(&low) < brightness(&high));
}

/// Air-breathers voice through a larynx, flyers sing through a syrinx, shells stridulate on a file,
/// and an ambusher hisses with breath alone.
#[test]
fn every_body_finds_its_organ() {
    let mut rand = Rand::new(9, 0);
    let organ = |t: Timbre, rand: &mut Rand| plan(&calling(t, 300.0, Cry::Contact), rand).source;
    for t in TIMBRES {
        let want = match t {
            Timbre::Whistle => Source::Syrinx,
            Timbre::Click | Timbre::Rasp => Source::File,
            Timbre::Hiss => Source::Breath,
            _ => Source::Larynx,
        };
        assert_eq!(organ(t, &mut rand), want, "{t:?}");
    }
}

/// Each cry has its shape: an alarm is higher and louder than a contact call, a threat snorts and then
/// winds up from clean to rough, distress is rougher than contact, a young has a shorter tract, and a
/// death cry falls away and ends on a last breath.
#[test]
fn cries_follow_the_state() {
    let mut rand = Rand::new(11, 0);
    let mut of = |cry| plan(&calling(Timbre::Low, 120.0, cry), &mut rand);
    let (contact, alarm, threat) = (of(Cry::Contact), of(Cry::Alarm), of(Cry::Threat));
    let (distress, young, death) = (of(Cry::Distress), of(Cry::Young), of(Cry::Death));
    assert!(alarm.hz > contact.hz && alarm.gain > contact.gain);
    assert!(!threat.notes[0].voiced, "a threat starts with a snort");
    let growl = threat.notes[1];
    assert!(
        growl.voiced && growl.rough[1] > growl.rough[0] + 0.5,
        "{growl:?}"
    );
    assert!(distress.notes[0].rough[0] > contact.notes[0].rough[0] + 0.3);
    assert!(young.formants[0] > contact.formants[0] && young.hz > contact.hz);
    let last = death.notes[death.count - 1];
    let cry = death.notes[death.count - 2];
    assert!(
        !last.voiced && cry.pitch[2] < cry.pitch[0],
        "{:?}",
        death.notes
    );
    assert!(contact.gain < 0.7 * distress.gain);
}

/// Two members of one herd call at pitches a few percent apart; the same member stays within the
/// small spread of one call to the next.
#[test]
fn individuals_differ() {
    let mut rand = Rand::new(13, 0);
    let base = calling(Timbre::Hoot, 400.0, Cry::Contact);
    let mut hz = |one| plan(&Call { one, ..base }, &mut rand).hz;
    let (a, b, again) = (hz(0x00), hz(0xFF), hz(0x00));
    assert!(b / a > 1.05, "{a} {b}");
    assert!((again / a - 1.0).abs() < 0.07);
}

/// The voice print of a species shapes its song: two prints that pick different melodies sweep
/// differently.
#[test]
fn prints_pick_the_song() {
    let mut rand = Rand::new(17, 0);
    let song = |print: u8, rand: &mut Rand| {
        let mut call = calling(Timbre::Whistle, 2000.0, Cry::Contact);
        call.voice.print = print;
        plan(&call, rand).notes[0].pitch
    };
    let (rising, falling) = (song(0x00, &mut rand), song(0x10, &mut rand));
    assert!(rising[2] > rising[0] && falling[2] < falling[0]);
}
