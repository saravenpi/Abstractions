//! The tool markers: every tool of the pack that is out in the world where it came to rest, a
//! thrown spear in the ground or in an animal, named for the tool, on the compass, the minimap, the
//! map and over the world, until it is back in the pack.

use bevy::math::DVec3;
use bevy::prelude::*;

use crate::craft::{Out, Pack};
use crate::menus::{Marker, MarkerKind, MarkerPlace, Markers};
use crate::ui::theme;

/// Owner of the markers of the game itself.
const GAME: u32 = 0;

/// Returns the marker of the tool of `out`, where it last came to rest, or nothing while it has not
/// come to rest anywhere yet.
pub fn mark(out: &Out) -> Option<Marker> {
    let body = out.body?;
    Some(Marker {
        kind: MarkerKind::Tool,
        place: MarkerPlace::Body(body, DVec3::from(out.at)),
        color: theme::TEXT,
        label: format!("Your {}", out.item.name()),
        owner: GAME,
    })
}

/// Writes a marker for every tool of the pack that is out in the world.
pub fn tools(pack: Option<Res<Pack>>, mut markers: ResMut<Markers>) {
    markers.clear(MarkerKind::Tool, GAME);
    let Some(pack) = pack else {
        return;
    };
    markers.0.extend(pack.out.iter().filter_map(mark));
}
