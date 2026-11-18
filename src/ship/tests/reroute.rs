//! Routes that must not disturb the drive, and the rule a route waits by: the destination
//! already flown to changes nothing, and a new one waits for the drop-out only when the drive ran.

use super::retarget::{DT, STILL, Seen, flight, frames};
use crate::computer::Target;
use crate::ship::pilot::{Autopilot, Phase, SETTLE, Settle};
use crate::ship::warp::Warp;
use crate::universe::Universe;

/// Engaging the destination already flown to changes nothing: the drive stays engaged and the
/// route goes on.
#[test]
fn the_same_destination_again_changes_nothing() {
    let (mut app, targets) = flight();
    app.world_mut()
        .resource_mut::<Autopilot>()
        .engage(targets[0]);
    frames(&mut app, 3000, Some(Phase::Cruising));
    frames(&mut app, 10, None);
    let route = app.world().resource::<Autopilot>().clone();
    app.world_mut()
        .resource_mut::<Autopilot>()
        .engage(targets[0]);
    assert_eq!(*app.world().resource::<Autopilot>(), route);
    let on = frames(&mut app, 10, None);
    let cruising =
        |seen: &Seen| seen.phase == Phase::Cruising && matches!(seen.warp, Warp::Active { .. });
    assert!(on.iter().all(cruising), "{on:?}");
}

/// A destination picked while the drive spools stops the spool, and the ship holds its heading
/// before it turns.
#[test]
fn a_new_destination_during_the_spool_stops_it_first() {
    let (mut app, targets) = flight();
    app.world_mut()
        .resource_mut::<Autopilot>()
        .engage(targets[0]);
    frames(&mut app, 3000, Some(Phase::Spooling));
    let spooling = frames(&mut app, 5, None);
    let nose = spooling.last().unwrap().nose;
    app.world_mut()
        .resource_mut::<Autopilot>()
        .engage(targets[1]);
    let held = frames(&mut app, 600, Some(Phase::Aligning));
    let still = |seen: &Seen| {
        (seen.phase, seen.warp) == (Phase::Dropping, Warp::Off)
            && seen.nose.angle_between(nose) < STILL
    };
    assert!(!held.is_empty() && held.iter().all(still), "{held:?}");
}

/// A route that finds the drive off may turn at once. One that finds it running stops it once,
/// holds for the time the drop-out takes on screen, and is clear after that.
#[test]
fn a_route_waits_for_the_drop_out_only_when_the_drive_ran() {
    let target = Target::Body(Universe::new(7).home());
    let mut calm = Autopilot::default();
    calm.engage(target);
    assert_eq!(calm.settle(false, 10.0), Settle::Clear);
    let mut hot = Autopilot::default();
    hot.engage(target);
    assert_eq!(hot.settle(true, 10.0), Settle::Stop);
    assert_eq!(hot.settle(false, 10.0 + DT), Settle::Hold);
    assert_eq!(hot.settle(false, 10.0 + SETTLE - DT), Settle::Hold);
    assert_eq!(hot.settle(false, 10.0 + SETTLE), Settle::Clear);
    assert_eq!(hot.settle(true, 20.0), Settle::Clear);
}
