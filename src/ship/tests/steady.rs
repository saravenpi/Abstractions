//! The rule of the turn on a route: the ship aligns before the drive spools, and from the first
//! frame of the spool to the last of the warp it never turns.

use bevy::prelude::App;

use super::retarget::{Seen, course, flight, frames, see_ship};
use crate::computer::Target;
use crate::ship::pilot::{Autopilot, Phase};
use crate::ship::warp::Warp;

/// Largest angle in radians between the nose and the course at the first frame of the spool.
const TIGHT: f64 = 1.0e-3;

/// Largest angle in radians the nose may move from one frame of the drive to the next.
const NONE: f64 = 1.0e-12;

/// Returns the planet whose straight course passes the star at the greatest distance, so that
/// no star is in the way and the course is a straight line.
fn open_course(app: &mut App, targets: &[Target]) -> Target {
    let pos = see_ship(app).pos;
    let miss = |app: &mut App, target: &Target| pos.cross(course(app, *target)).length();
    let mut best = targets[0];
    for target in targets {
        if miss(app, target) > miss(app, &best) {
            best = *target;
        }
    }
    best
}

/// Flies a route to the planet with the straightest course and returns every frame of it, and
/// the angle between the nose and the course at the first frame that spools.
fn voyage() -> (Vec<Seen>, f64) {
    let (mut app, targets) = flight();
    let target = open_course(&mut app, &targets);
    app.world_mut().resource_mut::<Autopilot>().engage(target);
    let mut seen = frames(&mut app, 3000, Some(Phase::Spooling));
    let off = course(&mut app, target).angle_between(frames(&mut app, 1, None)[0].nose);
    seen.extend(frames(&mut app, 3000, Some(Phase::Arrived)));
    (seen, off)
}

/// The drive does not spool before the nose is on the course: every frame before the spool
/// aligns or waits, and the first spool starts within the tight tolerance.
#[test]
fn the_drive_does_not_spool_before_the_alignment_is_done() {
    let (seen, off) = voyage();
    let first = seen
        .iter()
        .position(|frame| matches!(frame.warp, Warp::Spooling { .. }))
        .expect("a spool");
    assert!(seen[..first].iter().all(|frame| frame.warp == Warp::Off));
    assert!(
        seen[..first]
            .iter()
            .any(|frame| frame.phase == Phase::Aligning)
    );
    assert!(off < TIGHT, "{off} rad off at the spool");
}

/// From the first frame of the spool to the last frame of the warp the heading is constant.
#[test]
fn the_heading_is_constant_from_the_spool_to_the_end_of_the_warp() {
    let (seen, _) = voyage();
    let drive: Vec<_> = seen
        .iter()
        .skip_while(|frame| frame.warp == Warp::Off)
        .take_while(|frame| frame.warp != Warp::Off)
        .collect();
    assert!(drive.iter().any(|f| matches!(f.warp, Warp::Active { .. })));
    let nose = drive[0].nose;
    assert!(
        drive
            .iter()
            .all(|frame| frame.nose.angle_between(nose) < NONE),
        "the nose moved in warp"
    );
}
