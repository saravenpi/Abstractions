//! The engage key in the ship: with a course set, it hands the course to the autopilot from
//! the cockpit, without opening the map.

use bevy::prelude::*;

use crate::computer::{Destination, Engage, RouteStatus};
use crate::settings::{Persistent, Settings};
use crate::space::{Location, Travel};

/// Hands the course to the autopilot on the engage key while seated with a course set and the
/// autopilot able to take it.
pub(super) fn engage(
    (keys, settings): (Res<ButtonInput<KeyCode>>, Res<Persistent<Settings>>),
    (location, destination, status): (Option<Res<Location>>, Res<Destination>, Res<RouteStatus>),
    mut commands: Commands,
) {
    let seated = location.is_some_and(|location| location.travel == Travel::InShip);
    let Some(target) = destination
        .0
        .filter(|_| seated && *status == RouteStatus::Idle)
    else {
        return;
    };
    if keys.just_pressed(settings.controls.keys.engage) {
        commands.trigger(Engage(target));
    }
}
