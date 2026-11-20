//! Placing the ship: its render pose, the render anchor while flying and the planet the world streams.

use bevy::ecs::system::SystemParam;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::body::Ship;
use super::mooring::anchor_for;
use super::sim::{Cosmos, Pace};
use super::sky::Skies;
use super::tune::BODY_SWITCH;
use super::warp::Warp;
use super::wells::Well;
use crate::player::Player;
use crate::space::{Anchor, Clock, Frame, Location, Placed, Rebased, Shift, Travel};
use crate::universe::{BodyId, PlanetSpec};
use crate::world::enter_planet;

/// Returns where the ship is drawn: its position moved by the frame time not yet simulated.
fn shown(ship: &Ship, carry: f64) -> DVec3 {
    let still = ship.landed || matches!(ship.warp, Warp::Active { .. });
    ship.pos + ship.vel * if still { 0.0 } else { carry }
}

/// Filter of the free root entities of a planet, without the player body, which the view places.
type Free = (With<Rebased>, Without<ChildOf>, Without<Player>);

/// What the anchor follows.
#[derive(SystemParam)]
pub struct Berth<'w> {
    /// Where the player is.
    location: Res<'w, Location>,
    /// What surrounds the ship.
    skies: Res<'w, Skies>,
    /// Bookkeeping of the fixed step.
    pace: Res<'w, Pace>,
    /// Universe time.
    clock: Res<'w, Clock>,
}

/// Keeps the render anchor at the ship while the player is seated, and carries the free
/// entities of a planet along when its tangent frame moves.
pub fn moor(
    berth: Berth,
    mut anchor: ResMut<Anchor>,
    ship: Single<&Ship>,
    mut free: Query<&mut Transform, Free>,
) {
    if berth.location.travel != Travel::InShip || anchor.flat {
        return;
    }
    let at = (ship.frame, shown(&ship, berth.pace.carry));
    let next = anchor_for(&anchor, at, berth.skies.planet(), berth.clock.seconds);
    if next == *anchor {
        return;
    }
    if next.frame == anchor.frame && matches!(next.frame, Frame::Surface(_)) {
        let shift = Shift::between(&anchor, &next);
        for mut transform in &mut free {
            transform.translation = shift.point(transform.translation);
            transform.rotation = shift.turn(transform.rotation);
        }
    }
    *anchor = next;
}

/// Writes the pose of the ship in the anchor frame, which the frame systems copy to its transform.
pub fn pose(anchor: Res<Anchor>, pace: Res<Pace>, ship: Single<(&Ship, &mut Placed)>) {
    let (ship, mut placed) = ship.into_inner();
    let next = Placed {
        pos: shown(ship, pace.carry),
        rot: ship.rot,
    };
    if anchor.frame == ship.frame && *placed != next {
        *placed = next;
    }
}

/// Returns the planet the ship is at: the one whose surface frame it is in, or in the frame of
/// a star the nearest planet once it is close or `current` is not of that star.
fn body_at(ship: &Ship, skies: &Skies, current: BodyId) -> Option<BodyId> {
    let star = match ship.frame {
        Frame::Surface(body) => return Some(body),
        Frame::System(star) => star,
        Frame::Galaxy(_) => return None,
    };
    let system = skies.spec().filter(|system| system.star.id == star)?;
    let reach = |(planet, well): (&PlanetSpec, &Well)| (well.reach(ship.pos), planet.id);
    let scored = system.planets.iter().zip(skies.wells.get(1..)?).map(reach);
    let (reach, body) = scored.min_by(|a, b| a.0.total_cmp(&b.0))?;
    (reach < BODY_SWITCH || current.star != star).then_some(body)
}

/// Names the planet the seated player is at and has the world stream it.
pub fn track(
    mut commands: Commands,
    cosmos: Cosmos,
    skies: Res<Skies>,
    mut location: ResMut<Location>,
    ship: Single<&Ship>,
) {
    if location.travel != Travel::InShip {
        return;
    }
    if let Some(body) = body_at(&ship, &skies, location.body).filter(|body| *body != location.body)
    {
        location.body = body;
    }
    if cosmos
        .planet
        .as_ref()
        .is_some_and(|planet| planet.body != location.body)
    {
        enter_planet(&mut commands, &cosmos.universe, location.body);
    }
}
