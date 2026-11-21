//! Tests of the expression machine and of the mood it is fed.

use bevy::prelude::*;

use super::aboard::{Flight, read};
use super::face::machine::{BLINK_EVERY, BLINK_SECONDS, Face, SHIFT_SECONDS};
use super::face::{Expression, Frame, Lid};
use super::mood::{ChirpKind, ambient, chirp};
use super::sense::CompanionSense;
use super::tune::{SLEEPY_NIGHT, TAKEOFF_CLIMB, WORRY_DEPTH};

/// Frame time of the simulated screen.
const DT: f32 = 1.0 / 60.0;

/// Whatever reaction is asked for, the face is back to neutral once it has run out.
#[test]
fn machine_always_returns_to_neutral() {
    for (index, expr) in Expression::ALL.into_iter().enumerate() {
        let mut face = Face::new(index as u64);
        face.react(expr, 1.5);
        let mut seen = false;
        for _ in 0..(1.4 / DT) as usize {
            face.advance(Expression::Neutral, DT);
            seen |= face.shown() == expr;
        }
        assert!(seen, "{expr:?} never showed");
        for _ in 0..(1.0 / DT) as usize {
            face.advance(Expression::Neutral, DT);
        }
        assert_eq!(face.shown(), Expression::Neutral);
        assert_eq!(face.frame(IVec2::ZERO).expr, Expression::Neutral);
    }
}

/// A change of expression is quick but not instant: the eyes close first and the new face
/// appears within the documented time, announced exactly once.
#[test]
fn changes_are_quick_but_not_instant() {
    let mut face = Face::new(1);
    assert_eq!(face.advance(Expression::Happy, DT), None);
    assert_eq!(face.shown(), Expression::Neutral);
    assert_ne!(face.frame(IVec2::ZERO).lid, Lid::Open);
    let mut announced = Vec::new();
    for _ in 0..(SHIFT_SECONDS / DT) as usize + 2 {
        announced.extend(face.advance(Expression::Happy, DT));
    }
    assert_eq!(announced, [Expression::Happy]);
    assert_eq!(face.frame(IVec2::ZERO).lid, Lid::Open);
    let one_long_frame = Face::new(1).advance(Expression::Happy, 1.0);
    assert_eq!(one_long_frame, None);
}

/// Blinks start at intervals inside the documented range and each lasts the documented time.
#[test]
fn blinks_come_at_natural_intervals() {
    let mut face = Face::new(77);
    let (mut starts, mut shut, mut was_open) = (Vec::new(), 0, true);
    for tick in 0..(600.0 / DT) as usize {
        face.advance(Expression::Neutral, DT);
        let open = face.frame(IVec2::ZERO).lid == Lid::Open;
        if was_open && !open {
            starts.push(tick as f32 * DT);
        }
        shut += usize::from(!open);
        was_open = open;
    }
    let gaps: Vec<f32> = starts.windows(2).map(|pair| pair[1] - pair[0]).collect();
    assert!(gaps.len() > 80, "only {} blinks in ten minutes", gaps.len());
    let (low, high) = (BLINK_EVERY.0 - 2.0 * DT, BLINK_EVERY.1 + 2.0 * DT);
    assert!(
        gaps.iter().all(|gap| (low..=high).contains(gap)),
        "{gaps:?}"
    );
    let spread =
        gaps.iter().copied().fold(0.0, f32::max) - gaps.iter().copied().fold(9.0, f32::min);
    assert!(spread > 2.0, "blinks are too regular");
    let each = shut as f32 * DT / starts.len() as f32;
    assert!(
        (each - BLINK_SECONDS).abs() < 2.0 * DT,
        "a blink lasts {each}"
    );
}

/// The mood goes from the most urgent to the least: worried, surprised, the idle, sleepy, neutral.
#[test]
fn mood_follows_the_surroundings() {
    let calm = CompanionSense::default();
    assert_eq!(ambient(&calm, None), Expression::Neutral);
    assert_eq!(
        ambient(&calm, Some(Expression::Curious)),
        Expression::Curious
    );
    let night = CompanionSense {
        night: SLEEPY_NIGHT + 0.1,
        ..calm
    };
    assert_eq!(ambient(&night, None), Expression::Sleepy);
    assert_eq!(ambient(&night, Some(Expression::Love)), Expression::Love);
    let mut takeoff = night;
    takeoff.climb = TAKEOFF_CLIMB + 1.0;
    let mut clouds = night;
    clouds.in_clouds = true;
    for sense in [takeoff, clouds] {
        assert_eq!(
            ambient(&sense, Some(Expression::Happy)),
            Expression::Surprised
        );
    }
    let dive = CompanionSense {
        depth: WORRY_DEPTH + 0.1,
        ..takeoff
    };
    assert_eq!(ambient(&dive, None), Expression::Worried);
    assert_eq!(chirp(Expression::Neutral), None);
    assert_eq!(chirp(Expression::Love), Some(ChirpKind::Delight));
}

/// Depth comes from the altitude of the eyes under the sea, the climb from the altimeter of the
/// flying ship and the clouds from the layer the sky draws, both only while seated, and nothing
/// is sensed away from a planet surface.
#[test]
fn senses_read_the_eyes_and_the_ship() {
    let on_foot = Flight::default();
    assert_eq!(read(Some(-3.0), None, &on_foot, DT), (3.0, 0.0, false));
    let walking = Flight {
        height: Some(10.5),
        cloud: 0.8,
        ..on_foot
    };
    assert_eq!(
        read(Some(10.0), Some(10.0), &walking, 0.05),
        (0.0, 0.0, false)
    );
    let flying = Flight {
        seated: true,
        ..walking
    };
    let (depth, climb, cloudy) = read(Some(10.0), Some(10.0), &flying, 0.05);
    assert!(depth == 0.0 && (climb - 10.0).abs() < 1e-3 && cloudy);
    let clear = Flight {
        cloud: 0.0,
        ..flying
    };
    assert_eq!(read(None, None, &clear, DT), (0.0, 0.0, false));
    let parked = Flight {
        height: None,
        ..clear
    };
    assert_eq!(read(Some(5.0), Some(900.0), &parked, DT), (0.0, 0.0, false));
}

/// Every picture of the face crosses the wire as two bytes and comes back the same.
#[test]
fn every_frame_packs_into_two_bytes_and_back() {
    for expr in Expression::ALL {
        for lid in Lid::ALL {
            for (x, y, beat) in [(-1, 1, false), (0, 0, true), (1, -1, true)] {
                let frame = Frame {
                    expr,
                    lid,
                    gaze: IVec2::new(x, y),
                    beat,
                };
                let (expr, face) = frame.pack();
                assert_eq!(Frame::unpack(expr, face), frame);
            }
        }
    }
}
