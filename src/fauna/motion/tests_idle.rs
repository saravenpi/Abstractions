//! Tests of the idle clocks: a herd never breathes, steps or swishes in step, breathing slows with
//! mass, and a body pants after a run and calms down over the following seconds.

use bevy::math::Vec3;

use super::fixtures::quadruped_with;
use super::idle::Idle;
use super::rig::Motion;
use super::tune::{BREATH_JITTER, Tune};

/// Seconds after which the own clock of a rig wraps, as `super::rig` wraps it.
const WRAP: f32 = 200.0 * std::f32::consts::PI;

/// Identity of the herd of the tests.
const HERD: u64 = 0x5EED_4E7D;

/// Returns the seeds of a herd of 20, as the walk seeds its members: the herd xor the slot.
fn herd() -> Vec<u64> {
    (0..20u64).map(|slot| HERD ^ slot).collect()
}

/// Returns the least gap in radians between two of `phases` around a turn.
fn closest(phases: impl IntoIterator<Item = f32>) -> f32 {
    let phases: Vec<f32> = phases.into_iter().collect();
    let mut least = f32::MAX;
    for (i, a) in phases.iter().enumerate() {
        for b in &phases[i + 1..] {
            let gap = (a - b).rem_euclid(std::f32::consts::TAU);
            least = least.min(gap.min(std::f32::consts::TAU - gap));
        }
    }
    least
}

/// In a herd of 20 no two members breathe at the same phase, at the start or after a minute of rest,
/// and no two start their stride, their tail swish or their own clock together.
#[test]
fn a_herd_never_breathes_in_step() {
    let tune = Tune::of_mass(70.0);
    let mut idles: Vec<Idle> = herd().into_iter().map(Idle::new).collect();
    assert!(closest(idles.iter().map(Idle::phase)) > 1e-5);
    for _ in 0..1800 {
        idles
            .iter_mut()
            .for_each(|i| i.step(&tune, (0.0, 0.0), 1.0 / 30.0));
    }
    assert!(closest(idles.iter().map(Idle::phase)) > 1e-5);
    let body = quadruped_with(2, 0.3);
    let motions: Vec<Motion> = herd()
        .into_iter()
        .map(|seed| Motion::new(&body, Vec3::ZERO, Vec3::NEG_Z, seed))
        .collect();
    let turn = |x: f32| x * std::f32::consts::TAU;
    assert!(closest(motions.iter().map(|m| turn(m.clock.phase))) > 1e-5);
    assert!(closest(motions.iter().map(|m| turn(m.beat.cycle))) > 1e-5);
    assert!(closest(motions.iter().map(|m| turn(m.time / WRAP))) > 1e-5);
}

/// A resting body breathes slower the heavier it is: 12 to 20 breaths a minute at 70 kg, under 10
/// at 600 kg, over 30 at 1 kg.
#[test]
fn breathing_slows_with_mass() {
    let per_minute = |mass: f32| {
        let tune = Tune::of_mass(mass);
        60.0 * Idle::new(HERD).rate(&tune)
    };
    let spread = (1.0 - BREATH_JITTER, 1.0 + BREATH_JITTER);
    let man = 60.0 * Tune::of_mass(70.0).breath();
    assert!(
        (12.0..=20.0).contains(&man),
        "{man} breaths a minute at 70 kg"
    );
    assert!(per_minute(70.0) >= man * spread.0 - 1e-3 && per_minute(70.0) <= man * spread.1 + 1e-3);
    assert!(60.0 * Tune::of_mass(600.0).breath() < 10.0);
    assert!(60.0 * Tune::of_mass(1.0).breath() > 30.0);
    let masses = [1.0, 10.0, 70.0, 600.0, 3000.0];
    let rates: Vec<f32> = masses.iter().map(|m| Tune::of_mass(*m).breath()).collect();
    assert!(rates.windows(2).all(|w| w[1] < w[0]), "{rates:?}");
}

/// After ten seconds of running a body breathes at least 1.8 times its resting rate; ten seconds
/// into its rest it still pants, and a minute later it is back within a quarter of its rest.
#[test]
fn a_run_raises_the_breath_and_it_calms() {
    let tune = Tune::of_mass(70.0);
    let rest = tune.breath();
    let mut idle = Idle::new(HERD);
    let run = |idle: &mut Idle, work: f32, seconds: f32| {
        for _ in 0..(seconds * 30.0) as u32 {
            idle.step(&tune, (0.0, work), 1.0 / 30.0);
        }
        idle.rate(&tune)
    };
    let ran = run(&mut idle, 1.0, 10.0);
    let soon = run(&mut idle, 0.0, 10.0);
    let late = run(&mut idle, 0.0, 60.0);
    println!("rest {rest:.3} Hz, ran {ran:.3}, 10 s later {soon:.3}, a minute later {late:.3}");
    assert!(
        ran > 1.8 * rest && soon > 1.3 * rest && soon < ran,
        "{ran} then {soon}"
    );
    assert!(
        late < 1.25 * rest,
        "{late} Hz a minute after the run, rest {rest}"
    );
}
