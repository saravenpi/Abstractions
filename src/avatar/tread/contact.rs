//! When and how hard a foot meets the ground, from the gait and from physics.
//!
//! A foot plants where its own phase of the leg cycle starts again: the left one as the phase
//! wraps, the right one half a stride later. The sole comes down with the speed of its swing
//! and the speed the body drops in its bob, `sqrt(4 g h)` for a bob of height `h`, so a step is
//! harder on a heavier world and at a run. While it is down the foot carries the weight of the
//! body over the share of the stride it is down for. A landing comes down with the speed of the
//! fall and stops within the give of the knees; a slide scrapes at its speed.

use std::f32::consts::PI;

use bevy::prelude::Component;

use super::tune::{BODY_KG, CREEP, HARD, HUSH, LAND_FROM, LEG_SHARE, ROLL, SCRAPE, STANCE};
use crate::audio::{Contact, Step, StepSurface};
use crate::avatar::Mode;
use crate::avatar::animate::Anim;
use crate::avatar::gait::{duty, run, stride_length};
use crate::avatar::tune::{BOB, CROUCH, STEP_LIFT};
use crate::universe::STANDARD_GRAVITY;

/// What the feet of one body remember between two frames.
#[derive(Component, Debug, Clone, Copy, PartialEq, Default)]
pub struct Footfall {
    /// Phase of the leg cycle on the last frame.
    phase: f32,
    /// Mode of the body on the last frame.
    mode: Mode,
    /// Speed along the local up on the last frame, negative when falling.
    rise: f32,
    /// Seconds left in which the gait sends no step.
    hush: f32,
    /// Seconds until the next scrape of a slide.
    scrape: f32,
}

/// Returns which feet plant as the leg cycle goes from phase `before` to `after`: the left,
/// then the right.
pub fn planted(before: f32, after: f32) -> [bool; 2] {
    let moved = (after - before).rem_euclid(1.0);
    [0.0, 0.5].map(|edge| {
        let ahead = (edge - before).rem_euclid(1.0);
        moved > 0.0 && ahead > 0.0 && ahead <= moved
    })
}

/// Returns a step with no ground and no place yet: what `contact` gives with a `force`, a `load`
/// in weights at home and a striking `mass`.
fn bare(contact: Contact, force: f32, load: f32, mass: f32) -> Step {
    Step {
        contact,
        force: force.clamp(0.0, 1.0),
        load,
        mass,
        roll: 0.0,
        lift: 0.0,
        ..Step::on(StepSurface::Grass)
    }
}

/// Returns a step of a body walking at `speed`, crouched by `crouch` from 0 to 1, under a
/// `gravity` in metres per second squared.
pub fn stride(speed: f32, crouch: f32, gravity: f32) -> Step {
    let speed = speed.abs().max(0.1);
    let (length, share) = (stride_length(speed), duty(speed));
    let swing = (1.0 - share) * length / speed;
    let lift = STEP_LIFT.0 + (STEP_LIFT.1 - STEP_LIFT.0) * run(speed);
    let bob = BOB.0 + BOB.1 * speed;
    let fall = (PI * lift / swing).hypot((4.0 * gravity * bob).sqrt());
    let stance = share * length / speed;
    let creep = 1.0 - CREEP * crouch.clamp(0.0, 1.0);
    let load = gravity / STANDARD_GRAVITY as f32 / (2.0 * share);
    Step {
        roll: (ROLL.0 * stance / creep).clamp(ROLL.1, ROLL.2),
        lift: (stance / creep).min(STANCE.0),
        ..bare(
            Contact::Stride,
            fall * creep / HARD,
            load,
            LEG_SHARE * BODY_KG,
        )
    }
}

/// Returns the landing of a body that falls onto both feet at `impact` metres per second under
/// a `gravity`, or nothing for a fall too slow to be heard. The knees give more the harder it
/// lands, as the animation shows, and the weight is the fall stopped over that give.
pub fn landing(impact: f32, gravity: f32) -> Option<Step> {
    if impact < LAND_FROM {
        return None;
    }
    let hard = (impact / HARD).min(1.0);
    let give = CROUCH.0 + (CROUCH.1 - CROUCH.0) * hard;
    let load = (gravity + impact * impact / (2.0 * give)) / STANDARD_GRAVITY as f32;
    let step = bare(Contact::Landing, hard, load, 2.0 * LEG_SHARE * BODY_KG);
    Some(Step {
        lift: STANCE.1,
        ..step
    })
}

/// Returns one scrape of a body that slides at `speed` under a `gravity`.
pub fn skid(speed: f32, gravity: f32) -> Step {
    let load = gravity / STANDARD_GRAVITY as f32;
    bare(Contact::Skid, SCRAPE.1 * speed.abs() / HARD, load, BODY_KG)
}

/// Returns whether `mode` holds the body off the ground.
fn aloft(mode: Mode) -> bool {
    matches!(
        mode,
        Mode::Jump | Mode::Fall | Mode::BootFall | Mode::Glide | Mode::Fly
    )
}

/// Returns the contacts a body posed by `anim` makes with the ground this frame, `dt` seconds
/// after the last, under a `gravity`: a step of each foot that plants, a landing, or a scrape of
/// a slide. They have no ground and no place yet.
pub fn contacts(feet: &mut Footfall, anim: &Anim, gravity: f32, dt: f32) -> [Option<Step>; 2] {
    let pose = &anim.input;
    let walking = matches!(pose.mode, Mode::Idle | Mode::Walk | Mode::Run);
    let landed = aloft(feet.mode) && (walking || pose.mode == Mode::Land || pose.mode.floored());
    feet.hush = (feet.hush - dt).max(0.0);
    feet.scrape = (feet.scrape - dt).max(0.0);
    let mut out = [None, None];
    if landed {
        out[0] = landing(-feet.rise, gravity);
        feet.hush = HUSH;
    } else if pose.mode == Mode::Skid && feet.scrape <= 0.0 {
        out[0] = Some(skid(pose.speed, gravity));
        feet.scrape = SCRAPE.0;
    } else if walking && feet.hush <= 0.0 {
        let step = stride(pose.speed, pose.crouch, gravity);
        let down = planted(feet.phase, anim.clock.phase);
        out = [0, 1].map(|foot| {
            down[foot].then_some(Step {
                left: foot == 0,
                ..step
            })
        });
    }
    (feet.phase, feet.mode, feet.rise) = (anim.clock.phase, pose.mode, pose.vertical_speed);
    out
}
