//! Tests of voices and calls: the body picks the sound, size the pace, herds sing at dawn and dusk,
//! young answer, and a hurt animal cries more the worse it is.

use super::calls::{Cry, Timbre, chorus, contact, distress, rhythm, voice};
use super::rig::find;
use crate::life::body::{Chain, Locomotion};
use crate::life::herd::fixtures::{meadow_life, role, species};
use crate::life::herd::{Herd, HerdState};
use crate::life::{Activity, Guild, Habitat, Plan, Rung};

/// Shelled bodies stridulate, soft ones gurgle, a hunter that breathes air growls, and a swimmer
/// is muffled.
#[test]
fn the_body_picks_the_sound() {
    let ground = |g| role(g, Habitat::Ground, Activity::Day, 2.0);
    let mut beetle = species(1, ground(Guild::Grazer), 5.0);
    beetle.plan = Plan::Segmented;
    let mut slime = species(2, ground(Guild::Grazer), 5.0);
    slime.plan = Plan::Amorphous;
    let hunter = species(3, ground(Guild::Hunter), 80.0);
    let mut fish = species(
        4,
        role(Guild::Filter, Habitat::Water, Activity::Day, 2.0),
        5.0,
    );
    fish.body.locomotion = Locomotion::Swim;
    assert_eq!(voice(&beetle, Rung::Skeletal).timbre, Timbre::Rasp);
    assert_eq!(voice(&slime, Rung::Skeletal).timbre, Timbre::Croak);
    assert_eq!(voice(&hunter, Rung::Skeletal).timbre, Timbre::Growl);
    assert!(voice(&fish, Rung::Skeletal).muffled);
    assert!(!voice(&hunter, Rung::Skeletal).muffled);
}

/// A long, thin body calls lower than a stout one of the same mass.
#[test]
fn long_bodies_call_lower() {
    let grazer = role(Guild::Grazer, Habitat::Ground, Activity::Day, 2.0);
    let stout = species(1, grazer, 40.0);
    let mut long = stout.clone();
    long.body.length = 3.0;
    assert!(voice(&long, Rung::Skeletal).hz < voice(&stout, Rung::Skeletal).hz);
}

/// A longer body and a long neck lower the formants, and every species carries its own voice print,
/// the same each time it is asked for.
#[test]
fn formants_follow_the_tract() {
    let grazer = role(Guild::Grazer, Habitat::Ground, Activity::Day, 2.0);
    let short = species(1, grazer, 300.0);
    let mut long = short.clone();
    long.body.length = 4.0;
    let mut necked = short.clone();
    necked.body.neck = Some(Chain {
        parts: vec![0, 1],
        lengths: vec![0.6, 0.6],
        limit: 0.5,
    });
    let of = |s: &crate::life::Species| voice(s, Rung::Skeletal);
    assert!(of(&long).formant < of(&short).formant);
    assert!(of(&necked).formant < of(&short).formant);
    assert!((of(&short).formant - 350.0 / 0.3).abs() < 1.0);
    let prints: Vec<u8> = (0..8)
        .map(|i| of(&species(i, grazer, 300.0)).print)
        .collect();
    assert_eq!(prints[3], of(&species(3, grazer, 300.0)).print);
    assert!(prints.iter().any(|p| *p != prints[0]), "{prints:?}");
}

/// Bigger animals call slower, and a threat lasts longer than an alarm.
#[test]
fn big_animals_call_slower() {
    let grazer = role(Guild::Grazer, Habitat::Ground, Activity::Day, 2.0);
    let small = voice(&species(1, grazer, 2.0), Rung::Skeletal);
    let big = voice(&species(1, grazer, 2000.0), Rung::Skeletal);
    let (a, b) = (rhythm(&small, Cry::Contact), rhythm(&big, Cry::Contact));
    assert!(b.note > a.note && b.seconds() > a.seconds(), "{a:?} {b:?}");
    assert!(rhythm(&big, Cry::Threat).note > rhythm(&big, Cry::Alarm).note);
}

/// The chorus is loudest halfway through sunrise and silent by full day or night.
#[test]
fn chorus_peaks_at_twilight() {
    assert!((chorus(0.5) - 1.0).abs() < 1e-6);
    assert!(chorus(0.0) < 1e-6 && chorus(1.0) < 1e-6);
}

/// A herd calls more often in a full chorus than out of it, and herds with young hear them answer.
#[test]
fn herds_sing_at_dawn_and_young_answer() {
    let life = meadow_life();
    let awake = |h: &Herd| !matches!(h.state, HerdState::Sleep | HerdState::Stalk);
    let (herd, time) = find(&life, |h| h.count > 4 && awake(h)).expect("a herd awake");
    let count = |dawn: f32| {
        let spans = (0..3600).map(|k| (time + f64::from(k), time + f64::from(k + 1)));
        let calls: Vec<_> = spans.filter_map(|s| contact(&herd, &|_| dawn, s)).collect();
        calls
    };
    let (day, dawn) = (count(0.0), count(1.0));
    println!("{} calls an hour by day, {} at dawn", day.len(), dawn.len());
    assert!(dawn.len() > 2 * day.len());
    let young = dawn.iter().filter(|c| c.cry == Cry::Young);
    assert!(young.clone().all(|c| c.slot < herd.young));
    assert!(herd.young == 0 || young.count() > 0);
}

/// A badly hurt animal cries in distress more often than a lightly hurt one, the same however the
/// time is cut, and a whole one never.
#[test]
fn distress_follows_the_hurt() {
    let calls = |hurt: f32, step: f64| {
        let spans = (0..(600.0 / step) as i64).map(|k| (k as f64 * step, (k + 1) as f64 * step));
        let calls: Vec<f64> = spans
            .filter_map(|s| distress((7, 1, 2), hurt, s))
            .map(|c| c.at)
            .collect();
        calls
    };
    assert!(calls(0.0, 1.0).is_empty());
    assert_eq!(calls(0.9, 0.25), calls(0.9, 1.0));
    assert!(calls(0.9, 1.0).len() > calls(0.2, 1.0).len());
}
