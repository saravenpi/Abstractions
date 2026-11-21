//! Animal calls cross to the audio thread whole, render clean and audible for every organ and cry,
//! take turns in a chorus without holding back an alarm, and lose level and highs under water and
//! with distance.

use bevy::prelude::*;

use super::rig::{band, rms};
use super::{RATE, frames};
use crate::audio::call::Call;
use crate::audio::cue::AudioCue;
use crate::audio::cues::Pool;
use crate::audio::spatial::{Between, Listener, Origin, Spot, spot};
use crate::audio::throat::tune::CHORUS;
use crate::life::signs::calls::{CRIES, Cry, TIMBRES, Timbre, Voice};

/// Returns a call of `timbre` at `hz`.
pub(super) fn calling(timbre: Timbre, hz: f32, cry: Cry) -> Call {
    Call {
        voice: Voice {
            hz,
            timbre,
            notes: 3,
            muffled: false,
            formant: 900.0,
            print: 0x5A,
        },
        cry,
        one: 7,
    }
}

/// Returns the left channel of `seconds` of `call` played alone, from straight ahead at its level.
pub(super) fn render(call: Call, seconds: f32) -> Vec<f32> {
    let mut pool = Pool::new(RATE);
    pool.play(AudioCue::Call(call), &Spot::FLAT);
    (0..frames(seconds)).map(|_| pool.frame()[0]).collect()
}

/// Returns `sent` as it comes out of a cue packed and unpacked.
fn crossed(sent: Call) -> Call {
    match AudioCue::decode(AudioCue::Call(sent).encode()) {
        Some(AudioCue::Call(got)) => got,
        _ => panic!("the call did not come back"),
    }
}

/// A call packed into a cue comes back with its pitch within a sixteenth of an octave, its formants
/// within an eighth, and every cry, print, individual and the muffling whole. Where it is travels in
/// its spot, not in the call.
#[test]
fn calls_cross_whole() {
    let voices = [
        (Timbre::Low, 110.0),
        (Timbre::Growl, 60.0),
        (Timbre::Rasp, 3000.0),
    ];
    let octave = |a: f32, b: f32| (a / b).log2().abs();
    for ((t, hz), cry) in voices.into_iter().cycle().zip(CRIES) {
        let mut sent = Call {
            one: 200,
            ..calling(t, hz, cry)
        };
        sent.voice.muffled = cry == Cry::Young;
        let got = crossed(sent);
        let v = got.voice;
        assert!(octave(v.hz, hz) < 0.035, "{hz} Hz came back {}", v.hz);
        assert!(octave(v.formant, 900.0) < 0.07);
        assert_eq!((v.timbre, v.notes, v.muffled), (t, 3, sent.voice.muffled));
        assert_eq!((got.cry, v.print, got.one), (cry, 0x5A, 200));
    }
}

/// Every timbre and cry renders a finite, audible buffer that ends on silence, with no click.
#[test]
fn every_call_renders() {
    for t in TIMBRES {
        for cry in CRIES {
            let mut pool = Pool::new(RATE);
            pool.play(AudioCue::Call(calling(t, 220.0, cry)), &Spot::FLAT);
            let buffer: Vec<f32> = (0..frames(8.0)).map(|_| pool.frame()[0]).collect();
            let peak = buffer.iter().fold(0.0_f32, |p, x| p.max(x.abs()));
            let jump = buffer
                .windows(2)
                .fold(0.0_f32, |j, w| j.max((w[1] - w[0]).abs()));
            assert!(buffer.iter().all(|x| x.is_finite()), "{t:?} {cry:?}");
            assert!(peak > 0.01 && peak < 0.9, "{t:?} {cry:?} peak {peak}");
            assert!(jump < 0.25, "{t:?} {cry:?} jump {jump}");
            assert!(!pool.busy(), "{t:?} {cry:?} still sounds");
        }
    }
}

/// However many animals of many species keep in touch at once, no more than [`CHORUS`] contact calls
/// sound together, one species does not call over itself, and an alarm still gets a voice.
#[test]
fn a_chorus_takes_turns() {
    let mut pool = Pool::new(RATE);
    for k in 0..20_u8 {
        let mut call = calling(TIMBRES[usize::from(k) % 4], 300.0, Cry::Contact);
        call.voice.print = k;
        pool.play(AudioCue::Call(call), &Spot::FLAT);
    }
    let chorus = pool.throats().sounding();
    assert!((2..=CHORUS).contains(&chorus), "{chorus} calls at once");
    let mut same = Pool::new(RATE);
    for _ in 0..5 {
        same.play(
            AudioCue::Call(calling(Timbre::Hoot, 300.0, Cry::Contact)),
            &Spot::FLAT,
        );
    }
    assert_eq!(same.throats().sounding(), 1);
    for _ in 0..8 {
        pool.play(
            AudioCue::Call(calling(Timbre::Growl, 120.0, Cry::Alarm)),
            &Spot::FLAT,
        );
    }
    assert!(pool.throats().sounding() > chorus);
    pool.play(AudioCue::UiConfirm, &Spot::FLAT);
    assert!(
        pool.free() < 16,
        "the interface lost its voice to the animals"
    );
}

/// A call from under water is quieter and duller than the same call in the air.
#[test]
fn muffled_calls_are_quieter_and_duller() {
    let open = calling(Timbre::Low, 200.0, Cry::Contact);
    let mut under = open;
    under.voice.muffled = true;
    let (a, b) = (render(open, 3.0), render(under, 3.0));
    assert!(rms(&b) < 0.7 * rms(&a), "{} {}", rms(&b), rms(&a));
    let bright = |s: &[f32]| band(s, 2500.0) / rms(s).max(1e-9);
    assert!(bright(&b) < 0.5 * bright(&a));
}

/// The same call from far off has lost more of its highs to the air than from near.
#[test]
fn far_calls_lose_their_highs() {
    let call = calling(Timbre::Growl, 150.0, Cry::Threat);
    let ear = Listener::default();
    let heard = |metres: f32| {
        let spot = spot(
            &ear,
            Origin::At(Vec3::NEG_Z * metres, 7.0),
            Between::default(),
        );
        let mut pool = Pool::new(RATE);
        pool.play(AudioCue::Call(call), &spot);
        (0..frames(4.0))
            .map(|_| pool.frame()[0])
            .collect::<Vec<f32>>()
    };
    let bright = |s: &[f32]| band(s, 5000.0) / rms(s).max(1e-9);
    let (a, b) = (heard(10.0), heard(240.0));
    assert!(
        bright(&b) < 0.7 * bright(&a),
        "{} {}",
        bright(&b),
        bright(&a)
    );
}
