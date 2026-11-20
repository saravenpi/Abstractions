//! The rule of the turn on a route: the ship aligns before the drive spools, and from the first
//! frame of the spool to the last of the warp it never turns.

use super::retarget::{Seen, course, flight, frames};
use crate::ship::pilot::{Autopilot, Phase};
use crate::ship::warp::Warp;

/// Largest angle in radians between the nose and the course at the first frame of the spool.
const TIGHT: f64 = 1.0e-3;

/// Largest angle in radians the nose may move from one frame of the drive to the next.
const NONE: f64 = 1.0e-12;

/// Flies a route to its first planet and returns every frame of it, and the angle between the
/// nose and the course at the first frame that spools.
fn voyage() -> (Vec<Seen>, f64) {
    let (mut app, targets) = flight();
    app.world_mut()
        .resource_mut::<Autopilot>()
        .engage(targets[0]);
    let mut seen = frames(&mut app, 3000, Some(Phase::Spooling));
    let off = course(&mut app, targets[0]).angle_between(frames(&mut app, 1, None)[0].nose);
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
