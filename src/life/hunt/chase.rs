//! The path of a hunter herd during a hunt: its route bent towards the prey and back.
//!
//! The herd centre is the point a share `w` of the way from where its route would put it to the end
//! of the chase. `w` eases from 0 to 0.5 during the stalk (ambushers wait, with `w` at 0), to 1
//! during the chase, stays 1 while the hunters feed, and eases back to 0 as they rejoin the route.
//! Each step eases with zero speed at both ends, so the path is continuous and smooth.

use super::Hunt;
use crate::life::Guild;
use crate::life::herd::{HerdState, Pose, Scene, bearing, slerp};

/// Returns the share eased in at `u`, 0 to 1, and its rate per unit of `u`.
fn ease(u: f64) -> (f64, f64) {
    let u = u.clamp(0.0, 1.0);
    (u * u * (3.0 - 2.0 * u), 6.0 * u * (1.0 - u))
}

/// Returns the share of the way to the chase end, its rate per second and the state of the hunters
/// at `time` in `hunt`.
fn share(hunt: &Hunt, ambush: bool, time: f64) -> (f64, f64, HerdState) {
    let held = if ambush { 0.0 } else { 0.5 };
    let into = time - hunt.start;
    let chased = into - hunt.stalk;
    let fed = chased - hunt.chase - hunt.feed;
    if into < hunt.stalk {
        let (e, rate) = ease(into / hunt.stalk);
        (held * e, held * rate / hunt.stalk, HerdState::Stalk)
    } else if chased < hunt.chase {
        let (e, rate) = ease(chased / hunt.chase);
        (
            held + (1.0 - held) * e,
            (1.0 - held) * rate / hunt.chase,
            HerdState::Chase,
        )
    } else if fed < 0.0 {
        (1.0, 0.0, HerdState::Feed)
    } else {
        let (e, rate) = ease(fed / hunt.back);
        (1.0 - e, -rate / hunt.back, HerdState::Travel)
    }
}

/// Returns the pose of the hunter herd of `hunt` at `time`, when its route alone gives `base`.
///
/// The speed is an upper bound of the true speed of the centre: the route's share plus the pull.
pub(super) fn bend(scene: &Scene, base: &Pose, hunt: &Hunt, time: f64) -> Pose {
    let ambush = scene
        .life
        .species
        .get(usize::from(hunt.hunter_species))
        .is_some_and(|s| s.role.guild == Guild::Ambusher);
    let (w, rate, state) = share(hunt, ambush, time);
    let centre = slerp(base.centre, hunt.to, w);
    let pull = scene.metres(base.centre, hunt.to) * rate.abs();
    let heading = match state {
        HerdState::Travel => bearing(centre, hunt.to, base.centre),
        HerdState::Feed => base.heading,
        _ => bearing(centre, base.centre, hunt.to),
    };
    let state = if w <= 0.0 && state == HerdState::Travel {
        base.state
    } else {
        state
    };
    Pose {
        centre,
        heading,
        speed: ((f64::from(base.speed) * (1.0 - w) + pull) * 1.02) as f32,
        state,
    }
}
