//! The feet on the ground: one step per foot that plants, left and right in turn, a landing for
//! a fall, a scrape for a slide, harder steps on a heavier world and at a run, softer ones when
//! crouched.

use super::contact::{landing, planted, stride};
use super::stage::{live, moving};
use crate::audio::Contact;
use crate::avatar::Mode;
use crate::avatar::gait::stride_length;
use crate::universe::STANDARD_GRAVITY;

/// Gravity of home, in metres per second squared.
const HOME: f32 = STANDARD_GRAVITY as f32;

/// A foot plants once as its phase starts again: the left as the cycle wraps, the right half
/// a cycle later, and neither while the cycle stands still.
#[test]
fn each_foot_plants_on_its_edge() {
    assert_eq!(planted(0.9, 0.05), [true, false]);
    assert_eq!(planted(0.45, 0.55), [false, true]);
    assert_eq!(planted(0.1, 0.2), [false, false]);
    assert_eq!(planted(0.3, 0.3), [false, false]);
    assert_eq!(planted(0.0, 0.1), [false, false]);
}

/// Ten seconds of a walk give one step per foot that plants: two per stride of the gait, the
/// feet in turn, no more and no fewer.
#[test]
fn a_walk_makes_one_step_per_contact() {
    for speed in [2.0, 5.0, 8.5] {
        let steps = live(|_| moving(Mode::Walk, speed, 0.0), HOME, 10.0);
        let strides = 10.0 * speed / stride_length(speed);
        let count = steps.len() as f32;
        assert!(
            (count - 2.0 * strides).abs() <= 2.0,
            "{speed}: {count} for {strides}"
        );
        for pair in steps.windows(2) {
            assert_ne!(
                pair[0].1.left, pair[1].1.left,
                "{speed}: the feet go in turn"
            );
            assert_eq!(pair[0].1.contact, Contact::Stride);
        }
    }
}

/// Standing still, sitting or swimming makes no step.
#[test]
fn a_still_body_makes_no_step() {
    for mode in [Mode::Idle, Mode::Seated, Mode::Swim] {
        assert!(
            live(|_| moving(mode, 0.0, 0.0), HOME, 3.0).is_empty(),
            "{mode:?}"
        );
    }
}

/// A step lands harder at a run than at a walk, and harder and heavier on a heavier world.
#[test]
fn speed_and_gravity_make_a_step_harder() {
    let walk = stride(5.0, 0.0, HOME);
    let run = stride(8.5, 0.0, HOME);
    let heavy = stride(5.0, 0.0, 2.5 * HOME);
    let light = stride(5.0, 0.0, 0.4 * HOME);
    assert!(run.force > walk.force && run.load > walk.load);
    assert!(heavy.force > walk.force && heavy.load > 2.0 * walk.load);
    assert!(light.force < walk.force && light.load < walk.load);
    assert!(walk.force > 0.08 && walk.force < 0.2, "{}", walk.force);
}

/// A crouched body creeps: its foot comes down softer and rolls slower than a standing walk.
#[test]
fn a_crouched_step_is_softer() {
    let standing = stride(5.0, 0.0, HOME);
    let crouched = stride(2.25, 1.0, HOME);
    assert!(
        crouched.force < 0.6 * standing.force,
        "{} {}",
        crouched.force,
        standing.force
    );
    assert!(crouched.roll > standing.roll);
}

/// A fall onto the feet lands once with both feet, harder the faster it fell, heavier than any
/// step, and the gait sends no step while the feet are already down. A step down is not heard.
#[test]
fn a_fall_lands_once() {
    let script = |t: f32| match t {
        _ if t < 1.0 => moving(Mode::Walk, 5.0, 0.0),
        _ if t < 1.5 => crate::avatar::AvatarPose {
            vertical_speed: -5.0,
            ..moving(Mode::Fall, 5.0, 0.0)
        },
        _ => moving(Mode::Walk, 5.0, 0.0),
    };
    let steps = live(script, HOME, 2.0);
    let landings: Vec<_> = steps
        .iter()
        .filter(|(_, step)| step.contact == Contact::Landing)
        .collect();
    assert_eq!(landings.len(), 1, "{steps:?}");
    let (at, land) = landings[0];
    assert!(land.force > stride(8.5, 0.0, HOME).force && land.load > 3.0 && land.mass > 20.0);
    let next = steps
        .iter()
        .find(|(t, _)| t > at)
        .expect("the walk goes on");
    assert!(next.0 - at >= 0.19, "{} after the landing", next.0 - at);
    assert!(landing(0.5, HOME).is_none());
    assert!(landing(8.0, HOME).unwrap().force > landing(3.0, HOME).unwrap().force);
}

/// A slide scrapes ten times a second, and a faster slide scrapes harder.
#[test]
fn a_slide_scrapes() {
    let steps = live(|_| moving(Mode::Skid, 8.0, 0.0), HOME, 1.0);
    assert!((9..=11).contains(&steps.len()), "{}", steps.len());
    assert!(steps.iter().all(|(_, step)| step.contact == Contact::Skid));
    let slow = live(|_| moving(Mode::Skid, 3.0, 0.0), HOME, 0.3);
    assert!(slow[0].1.force < steps[0].1.force);
}
