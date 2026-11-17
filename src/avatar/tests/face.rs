//! The face: which state shows which face, the blinks and reactions of its machine, and its pictures.

use std::collections::HashSet;

use bevy::prelude::*;

use super::FRAME;
use crate::avatar::face::machine::{BLINK_EVERY, SHIFT_SECONDS, Visor, interval};
use crate::avatar::face::paint::{Canvas, HEIGHT, WIDTH, paint, rgba};
use crate::avatar::face::{Frame, Lid, Mien, ambient, gaze, landing};
use crate::avatar::tune::{FAST_FLIGHT, FREE_FALL, SLEEPY_AFTER};
use crate::avatar::{AvatarPose, Mode};

/// Returns the picture of a face with open eyes looking ahead.
fn open(mien: Mien, beat: bool) -> Canvas {
    paint(&Frame {
        mien,
        beat,
        ..default()
    })
}

/// Each state shows its face: focused at a sprint or in fast flight, wide-eyed in free fall,
/// sleepy after a long rest, the underwater look in a dive, calm otherwise.
#[test]
fn the_state_picks_the_face() {
    let of = |mode: Mode, fall: f32, airspeed: f32, still: f32| {
        let input = AvatarPose {
            mode,
            vertical_speed: -fall,
            airspeed,
            ..default()
        };
        ambient(&input, still)
    };
    assert_eq!(of(Mode::Idle, 0.0, 0.0, 1.0), Mien::Neutral);
    assert_eq!(of(Mode::Idle, 0.0, 0.0, SLEEPY_AFTER + 1.0), Mien::Sleepy);
    assert_eq!(of(Mode::Walk, 0.0, 5.0, SLEEPY_AFTER + 1.0), Mien::Neutral);
    assert_eq!(of(Mode::Run, 0.0, 8.5, 0.0), Mien::Focused);
    assert_eq!(of(Mode::Fall, FREE_FALL - 1.0, 5.0, 0.0), Mien::Neutral);
    assert_eq!(of(Mode::Fall, FREE_FALL + 1.0, 50.0, 0.0), Mien::Wide);
    assert_eq!(of(Mode::Dive, 0.0, 2.0, 0.0), Mien::Submerged);
    assert_eq!(of(Mode::Swim, 0.0, 2.0, 0.0), Mien::Neutral);
    for mode in [Mode::Glide, Mode::Fly, Mode::BootFall] {
        assert_eq!(of(mode, 3.0, FAST_FLIGHT + 1.0, 0.0), Mien::Focused);
        assert_eq!(of(mode, 3.0, FAST_FLIGHT - 1.0, 0.0), Mien::Neutral);
    }
    assert_eq!((landing(0.1), landing(0.9)), (Mien::Happy, Mien::Wince));
    assert_eq!(gaze(Vec2::new(0.6, -0.6)), IVec2::new(1, -1));
    assert_eq!(gaze(Vec2::new(-0.6, 0.1)), IVec2::new(-1, 0));
}

/// A change of face passes through shut eyes and is done within the shift time.
#[test]
fn faces_change_through_a_blink() {
    let mut visor = Visor::default();
    let mut lids = HashSet::new();
    for _ in 0..(SHIFT_SECONDS / FRAME) as usize + 3 {
        visor.advance(Mien::Focused, FRAME);
        lids.insert(visor.frame(IVec2::ZERO).lid);
    }
    assert_eq!(visor.shown(), Mien::Focused);
    assert!(
        lids.contains(&Lid::Closed) && lids.contains(&Lid::Half),
        "{lids:?}"
    );
    assert_eq!(visor.frame(IVec2::ZERO).lid, Lid::Open);
}

/// A reaction overrides the ambient face for its time, then the face falls back.
#[test]
fn reactions_last_and_fall_back() {
    let mut visor = Visor::default();
    visor.react(Mien::Wince, 1.0);
    for _ in 0..30 {
        visor.advance(Mien::Neutral, FRAME);
    }
    assert_eq!(visor.shown(), Mien::Wince);
    for _ in 0..60 {
        visor.advance(Mien::Neutral, FRAME);
    }
    assert_eq!(visor.shown(), Mien::Neutral);
}

/// The eyes blink now and then, at intervals inside the documented span, the same on every run.
#[test]
fn blinks_come_at_natural_intervals() {
    for count in 0..200 {
        let wait = interval(count);
        assert!(
            (BLINK_EVERY.0..BLINK_EVERY.1).contains(&wait),
            "blink {count} after {wait} s"
        );
    }
    assert!(
        (0..200)
            .map(|n| (interval(n) * 10.0) as i32)
            .collect::<HashSet<_>>()
            .len()
            > 20
    );
    let (mut visor, mut blinks, mut shut) = (Visor::default(), 0, false);
    for _ in 0..(60.0 / FRAME) as usize {
        visor.advance(Mien::Neutral, FRAME);
        let closed = visor.frame(IVec2::ZERO).lid == Lid::Closed;
        blinks += usize::from(closed && !shut);
        shut = closed;
    }
    assert!((8..=24).contains(&blinks), "{blinks} blinks in a minute");
    assert_eq!(visor, {
        let mut again = Visor::default();
        (0..(60.0 / FRAME) as usize).for_each(|_| again.advance(Mien::Neutral, FRAME));
        again
    });
}

/// Returns the number of lit pixels of the face `mien` looking by `gaze`.
fn lit(mien: Mien, beat: bool, gaze: (i32, i32)) -> u32 {
    let gaze = IVec2::new(gaze.0, gaze.1);
    let canvas = paint(&Frame {
        mien,
        beat,
        gaze,
        ..default()
    });
    canvas.iter().map(|row| row.count_ones()).sum()
}

/// Every face is its own picture, symmetric, with lit pixels, and fits the screen wherever it looks.
#[test]
fn every_face_is_a_distinct_picture() {
    let pictures: HashSet<Canvas> = Mien::ALL
        .into_iter()
        .map(|mien| open(mien, false))
        .collect();
    assert_eq!(pictures.len(), Mien::ALL.len());
    for (mien, beat) in Mien::ALL.into_iter().flat_map(|m| [(m, false), (m, true)]) {
        let canvas = open(mien, beat);
        let ahead = lit(mien, beat, (0, 0));
        assert!((10..120).contains(&ahead), "{mien:?} lights {ahead} pixels");
        assert!(
            canvas.iter().all(|row| *row == row.reverse_bits()),
            "{mien:?} is lopsided"
        );
        assert_eq!(rgba(&canvas).len(), WIDTH * HEIGHT * 4);
        for gaze in (-1..=1).flat_map(|x| (-1..=1).map(move |y| (x, y))) {
            assert_eq!(
                lit(mien, beat, gaze),
                ahead,
                "{mien:?} loses pixels looking {gaze:?}"
            );
        }
    }
    assert_ne!(open(Mien::Happy, false), open(Mien::Happy, true));
    let blink = paint(&Frame {
        lid: Lid::Closed,
        ..default()
    });
    assert_ne!(blink, open(Mien::Neutral, false));
}
