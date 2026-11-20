//! Tests of acting on a stance: fleeing comes within a second and stops once nothing is noticed,
//! a wary animal only looks up, one that threatens stands, a hunter approaches, and shy plant eaters
//! stay alarmed longer than bold hunters.

use bevy::math::Vec3;

use super::react::{Act, Mood, Temper};
use super::senses::Stance;
use crate::life::Guild;

/// Returns what an animal of `temper` does after `seconds` of noticing a threat 8 m away in
/// `stance`.
fn after(temper: &Temper, stance: Stance, seconds: f32) -> Act {
    let mut mood = Mood::default();
    let dt = 0.05;
    for _ in 0..(seconds / dt) as u32 {
        mood.feel(temper, Some((Vec3::new(8.0, 0.0, 0.0), stance)), dt);
    }
    mood.act()
}

/// Each stance shows as its act within a second.
#[test]
fn stances_become_acts() {
    let grazer = Temper::new(Guild::Grazer);
    assert_eq!(after(&grazer, Stance::Flee, 1.0), Act::Flee);
    assert_eq!(after(&grazer, Stance::Wary, 1.0), Act::Alert);
    assert_eq!(after(&grazer, Stance::Threaten, 1.0), Act::Threaten);
    assert_eq!(after(&grazer, Stance::Attack, 1.0), Act::Approach);
    assert_eq!(after(&grazer, Stance::Calm, 1.0), Act::Graze);
}

/// A fleeing animal calms down once nothing is noticed, a grazer slower than a hunter.
#[test]
fn flight_calms_down() {
    let grazer = Temper::new(Guild::Grazer);
    let hunter = Temper::new(Guild::Hunter);
    assert!(hunter.calm < grazer.calm);
    let mut mood = Mood::default();
    for _ in 0..20 {
        mood.feel(&grazer, Some((Vec3::X * 3.0, Stance::Flee)), 0.05);
    }
    assert_eq!(mood.act(), Act::Flee);
    for _ in 0..600 {
        mood.feel(&grazer, None, 0.05);
    }
    assert_eq!(mood.act(), Act::Graze);
}
