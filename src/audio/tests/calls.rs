//! Animal calls cross to the audio thread whole, render clean and audible, fall in pitch with size,
//! and never take the voices every other cue needs.

use super::{RATE, frames};
use crate::audio::call::Call;
use crate::audio::cue::AudioCue;
use crate::audio::cues::Pool;
use crate::audio::cues::calls::call;
use crate::audio::dsp::rand::Rand;
use crate::life::signs::calls::{CRIES, Cry, TIMBRES, Timbre, Voice};

/// Returns a call of `timbre` at `hz`, from straight ahead at full level.
fn calling(timbre: Timbre, hz: f32, cry: Cry) -> Call {
    Call {
        voice: Voice {
            hz,
            timbre,
            notes: 3,
            muffled: false,
        },
        cry,
        side: 0.0,
        near: 1.0,
    }
}

/// A call packed into a cue comes back with its pitch within an eighth of an octave, its place
/// within a fifteenth, and every cry and the muffling whole.
#[test]
fn calls_cross_whole() {
    let voices = [
        (Timbre::Low, 110.0),
        (Timbre::Growl, 60.0),
        (Timbre::Rasp, 3000.0),
    ];
    for ((t, hz), cry) in voices.into_iter().cycle().zip(CRIES) {
        let mut sent = Call {
            side: -0.6,
            near: 0.4,
            ..calling(t, hz, cry)
        };
        sent.voice.muffled = cry == Cry::Young;
        let Some(AudioCue::Call(got)) = AudioCue::decode(AudioCue::Call(sent).encode()) else {
            panic!("the call did not come back");
        };
        assert!(
            (got.voice.hz / hz).log2().abs() < 0.07,
            "{hz} Hz came back {}",
            got.voice.hz
        );
        let kept = (
            got.voice.timbre,
            got.voice.notes,
            got.cry,
            got.voice.muffled,
        );
        assert_eq!(kept, (sent.voice.timbre, 3, cry, sent.voice.muffled));
        assert!((got.side + 0.6).abs() < 0.07 && (got.near - 0.4).abs() < 0.04);
    }
}

/// Every timbre and cry renders a finite, audible buffer that ends on silence, with no click.
#[test]
fn every_call_renders() {
    for t in TIMBRES {
        for cry in CRIES {
            let mut pool = Pool::new(RATE);
            pool.play(AudioCue::Call(calling(t, 220.0, cry)));
            let buffer: Vec<f32> = (0..frames(4.0)).map(|_| pool.frame()[0]).collect();
            let peak = buffer.iter().fold(0.0_f32, |p, x| p.max(x.abs()));
            let jump = buffer
                .windows(2)
                .fold(0.0_f32, |j, w| j.max((w[1] - w[0]).abs()));
            assert!(buffer.iter().all(|x| x.is_finite()));
            assert!(peak > 0.01 && peak < 0.9, "{:?} {cry:?} peak {peak}", t);
            assert!(jump < 0.25, "{:?} {cry:?} jump {jump}", t);
            assert!(!pool.busy(), "{:?} {cry:?} still sounds", t);
        }
    }
}

/// The notes of a lower voice are lower, for every timbre.
#[test]
fn call_pitch_follows_the_voice() {
    let mut rand = Rand::new(7, 0);
    for t in TIMBRES {
        let (low, _) = call(&calling(t, 80.0, Cry::Contact), &mut rand);
        let (high, _) = call(&calling(t, 900.0, Cry::Contact), &mut rand);
        assert!(low[0].from < high[0].from, "{:?}", t);
    }
}

/// However many animals call at once, calls leave voices free for the other cues.
#[test]
fn calls_leave_voices_free() {
    let mut pool = Pool::new(RATE);
    for k in 0..20 {
        pool.play(AudioCue::Call(calling(
            TIMBRES[k % TIMBRES.len()],
            300.0,
            Cry::Alarm,
        )));
    }
    assert!(pool.free() >= 6, "{} voices free", pool.free());
    pool.play(AudioCue::UiConfirm);
    assert!(pool.busy());
}

/// A call from under water is quieter than the same call in the air.
#[test]
fn muffled_calls_are_quieter() {
    let mut rand = Rand::new(3, 0);
    let open = calling(Timbre::Whistle, 400.0, Cry::Contact);
    let mut under = open;
    under.voice.muffled = true;
    let (a, _) = call(&open, &mut rand);
    let (b, _) = call(&under, &mut rand);
    assert!(b[0].gain < a[0].gain);
}
