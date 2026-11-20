//! The herd markers: the herds the robot has seen, at the place each was last seen and fading out,
//! rewritten four times a second for the compass and the minimap, each named for its species once the field
//! guide has it, else "Herd".

use bevy::prelude::*;

use crate::fauna::guide::Seen;
use crate::fauna::{Nearby, Sighting};
use crate::menus::hud::context::{Element, Where};
use crate::menus::{Marker, MarkerKind, MarkerPlace, Markers};
use crate::space::{Anchor, Clock, Frame};
use crate::universe::BodyId;

/// Owner of the markers of the game itself.
const GAME: u32 = 0;

/// The muted sand colour of a herd marker, quiet beside the colours of players and waypoints.
pub const HERD: Color = Color::srgba(0.82, 0.74, 0.56, 0.8);

/// Returns the marker of the herd `sighting` on `body` at universe `time`, named for its species when
/// `named`, faded by the time since it was seen.
pub fn mark(sighting: &Sighting, body: BodyId, (named, time): (bool, f64)) -> Marker {
    Marker {
        kind: MarkerKind::Herd,
        place: MarkerPlace::Body(body, sighting.at),
        color: HERD.with_alpha(HERD.alpha() * sighting.fresh(time)),
        label: if named {
            sighting.name.clone()
        } else {
            "Herd".into()
        },
        owner: GAME,
    }
}

/// What the herd markers read: the herds seen, the field guide and the universe clock.
type Reads<'w> = (
    Option<Res<'w, Nearby>>,
    Option<Res<'w, Seen>>,
    Option<Res<'w, Clock>>,
);

/// Seconds between two refills of the herd markers.
pub const EVERY: f32 = 0.25;

/// Writes a marker for every herd the robot has seen, while the robot stands on the planet they are
/// on.
pub fn herds(
    (nearby, seen, clock): Reads<'_>,
    anchor: Option<Res<Anchor>>,
    mut markers: ResMut<Markers>,
    now: Res<Where>,
) {
    markers.clear(MarkerKind::Herd, GAME);
    if !now.has(Element::Herds) {
        return;
    }
    let here = anchor.map(|a| a.frame);
    let Some((nearby, body)) = nearby.and_then(|n| n.body.map(|b| (n, b))) else {
        return;
    };
    if here != Some(Frame::Surface(body)) {
        return;
    }
    for sighting in &nearby.herds {
        let named = seen.as_deref().is_some_and(|s| s.has(sighting.species));
        let time = clock.as_deref().map_or(sighting.seen, |c| c.seconds);
        markers.0.push(mark(sighting, body, (named, time)));
    }
}
