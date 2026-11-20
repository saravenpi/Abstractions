//! Tests of the call layer: a calling body drops its jaw and lifts its head on a note, rises for a
//! threat, and falls quiet once the call is over.

use bevy::math::{Quat, Vec2, Vec3};

use super::fixtures::part;
use super::fixtures_legs::biped;
use super::rig::Motion;
use super::song::Song;
use super::tests_plans::drive;
use crate::life::body::{Body, PartKind};
use crate::life::signs::calls::{Cry, Rhythm};

/// Returns the biped with a jaw under its head.
fn singer() -> Body {
    let mut body = biped();
    let head = (body.parts.len() - 1) as u8;
    let socket = (Vec3::new(0.0, -0.04, -0.04), Quat::IDENTITY);
    body.parts.push(part(
        PartKind::Jaw,
        Some(head),
        socket,
        Vec3::new(0.08, 0.03, 0.1),
    ));
    body
}

/// Returns a song of `cry` with three notes of half a second, `age` seconds in.
fn song(cry: Cry, age: f32) -> Song {
    let rhythm = Rhythm {
        note: 0.5,
        gap: 0.1,
        notes: 3,
    };
    Song { age, rhythm, cry }
}

/// Returns the pose of `body` standing still, making `song`.
fn posed(body: &Body, song: Option<Song>) -> Vec<super::Placed> {
    let mut motion = Motion::new(body, Vec3::ZERO, Vec3::NEG_Z, 4);
    motion.song = song;
    motion.pose(body)
}

/// At the height of a note the jaw drops, the head turns and the chest swells; between notes the
/// jaw is shut.
#[test]
fn a_note_opens_the_mouth() {
    let body = singer();
    let jaw = body.parts.len() - 1;
    let quiet = posed(&body, None);
    let sung = posed(&body, Some(song(Cry::Contact, 0.25)));
    let gap = posed(&body, Some(song(Cry::Contact, 0.55)));
    let gape = |p: &[super::Placed]| p[jaw - 1].1.inverse() * p[jaw].1;
    assert!(gape(&sung).angle_between(gape(&quiet)) > 0.3);
    assert!(sung[jaw - 1].1.angle_between(quiet[jaw - 1].1) > 0.2);
    assert!(sung[0].2.x > quiet[0].2.x);
    assert!(gape(&gap).angle_between(gape(&quiet)) < 1e-3);
}

/// A threat raises the body above its rest.
#[test]
fn a_threat_raises_the_body() {
    let body = singer();
    let quiet = posed(&body, None);
    let shown = posed(&body, Some(song(Cry::Threat, 0.8)));
    assert!(
        shown[0].0.y > quiet[0].0.y + 0.02,
        "{} {}",
        shown[0].0.y,
        quiet[0].0.y
    );
}

/// Once its notes are over, the song is dropped.
#[test]
fn songs_end() {
    let body = singer();
    let mut motion = Motion::new(&body, Vec3::ZERO, Vec3::NEG_Z, 4);
    motion.song = Some(song(Cry::Alarm, 0.0));
    for _ in 0..120 {
        motion.step(&body, &drive(Vec3::ZERO, 0.0), &|_: Vec2| 0.0, 1.0 / 60.0);
    }
    assert!(motion.song.is_none());
}
