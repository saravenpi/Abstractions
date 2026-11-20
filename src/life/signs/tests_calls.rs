//! Tests of calls: pitch falls with size, herds call now and then at the same moments for everyone,
//! hunts bring a threat before the chase and cries and alarms only through it, and a kill a cry of
//! distress.

use super::calls::{Cry, Timbre, contact, cries, voice};
use super::rig::{chase, find};
use crate::life::herd::fixtures::role;
use crate::life::herd::fixtures::{meadow_life, species};
use crate::life::herd::{Herd, HerdState};
use crate::life::{Activity, Guild, Habitat, Rung};

/// A heavier animal of the same kind calls lower, and shelled life stridulates.
#[test]
fn calls_pitch_scales_with_size() {
    let grazer = role(Guild::Grazer, Habitat::Ground, Activity::Day, 2.0);
    let pitches: Vec<f32> = [0.5, 5.0, 40.0, 400.0, 4000.0]
        .iter()
        .map(|&kg| voice(&species(1, grazer, kg), Rung::Skeletal).hz)
        .collect();
    println!("grazer calls from 0.5 kg to 4 t: {pitches:?} Hz");
    assert!(pitches.windows(2).all(|w| w[1] < w[0]), "{pitches:?}");
    assert!(
        pitches[2] > 150.0 && pitches[2] < 400.0,
        "40 kg calls at {} Hz",
        pitches[2]
    );
    let shelled = voice(&species(1, grazer, 1.0), Rung::Shelled);
    assert!(matches!(shelled.timbre, Timbre::Click | Timbre::Rasp));
}

/// An awake herd calls now and then, at the same moments however the time is cut into hearings.
#[test]
fn herds_call_now_and_then_the_same_for_everyone() {
    let life = meadow_life();
    let awake = |h: &Herd| !matches!(h.state, HerdState::Sleep | HerdState::Stalk);
    let (herd, time) = find(&life, |h| h.count > 4 && awake(h)).expect("an awake herd");
    let hour = |step: f64| {
        let spans = (0..(3600.0 / step) as i64)
            .map(|k| (time + k as f64 * step, time + (k + 1) as f64 * step));
        spans
            .filter_map(|s| contact(&herd, &|_| 0.0, s))
            .map(|c| c.at)
            .collect::<Vec<_>>()
    };
    let (fine, coarse) = (hour(0.25), hour(1.0));
    println!("herd of {} calls {} times an hour", herd.count, fine.len());
    assert_eq!(fine, coarse);
    assert!(fine.len() > 10 && fine.len() < 400, "{} calls", fine.len());
}

/// Hunters threaten before the chase and cry as it starts, and the prey calls the alarm only while
/// it is chased.
#[test]
fn hunts_cry_through_the_chase() {
    let life = meadow_life();
    let hunt = chase(&life).expect("a hunt with a chase");
    let (chase, end) = (hunt.start + hunt.stalk, hunt.outcome_time());
    let mut heard = Vec::new();
    cries(&hunt, (hunt.start - 10.0, end + 60.0), |c| heard.push(c));
    assert!(
        heard
            .iter()
            .any(|c| c.cry == Cry::Chase && c.at == chase && c.herd == hunt.hunter)
    );
    let threat = heard
        .iter()
        .any(|c| c.cry == Cry::Threat && c.at < chase && c.herd == hunt.hunter);
    assert!(
        threat || hunt.stalk <= 1.5,
        "no threat after {:.1} s of stalk",
        hunt.stalk
    );
    let alarms: Vec<_> = heard.iter().filter(|c| c.cry == Cry::Alarm).collect();
    assert!(
        !alarms.is_empty(),
        "no alarm through a chase of {:.1} s",
        hunt.chase
    );
    assert!(
        alarms
            .iter()
            .all(|c| c.herd == hunt.prey && chase <= c.at && c.at <= end)
    );
}
