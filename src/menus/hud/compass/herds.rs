//! The herd markers: the herds drawn near the robot, each at the middle of its animals, rewritten every
//! frame they move for the compass, the minimap and the map, each named for its species once the field
//! guide has it, else "Herd".

use bevy::prelude::*;

use crate::fauna::guide::Seen;
use crate::fauna::{Nearby, Sighting};
use crate::menus::hud::context::{Element, Where};
use crate::menus::{Marker, MarkerKind, MarkerPlace, Markers};
use crate::space::{Anchor, Frame};
use crate::universe::BodyId;

/// Owner of the markers of the game itself.
const GAME: u32 = 0;

/// The muted sand colour of a herd marker, quiet beside the colours of players and waypoints.
pub const HERD: Color = Color::srgba(0.82, 0.74, 0.56, 0.8);

/// Returns the marker of the herd `sighting` on `body`, named for its species when `named`.
pub fn mark(sighting: &Sighting, body: BodyId, named: bool) -> Marker {
    Marker {
        kind: MarkerKind::Herd,
        place: MarkerPlace::Body(body, sighting.at),
        color: HERD,
        label: if named {
            sighting.name.clone()
        } else {
            "Herd".into()
        },
        owner: GAME,
    }
}

/// What the herd markers read: the herds drawn and the field guide.
type Reads<'w> = (Option<Res<'w, Nearby>>, Option<Res<'w, Seen>>);

/// Seconds between two refills of the tool markers.
pub const EVERY: f32 = 0.25;

/// Writes a marker for every herd drawn near the robot, while the robot stands on the planet they are
/// on.
pub fn herds(
    (nearby, seen): Reads<'_>,
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
        markers.0.push(mark(sighting, body, named));
    }
}
