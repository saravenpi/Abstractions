//! The gates of the fauna systems: on which planet animals are drawn, and the forgetting of them all
//! when the anchor leaves it or its life is gone.

use std::sync::Arc;

use bevy::prelude::*;

use super::beast::Beast;
use super::herds::Herds;
use crate::space::{Anchor, Frame};
use crate::universe::{BodyId, PlanetSpec};
use crate::world::Terrain;

/// The planet under the anchor: its identity and its record of orbit and spin.
#[derive(Resource, Default)]
pub struct Spec {
    /// The planet, once read.
    pub planet: Option<Arc<PlanetSpec>>,
}

/// Returns the planet the anchor stands on, when it is a surface frame.
pub fn footing(anchor: &Anchor) -> Option<BodyId> {
    match anchor.frame {
        Frame::Surface(body) if !anchor.flat => Some(body),
        _ => None,
    }
}

/// Returns true when `terrain` shows a planet whose life is known and alive.
fn alive(terrain: Option<Res<Terrain>>) -> bool {
    terrain.is_some_and(|t| t.globe().lives() == Some(true))
}

/// Run condition: the anchor stands on a planet whose life is known and alive.
pub fn living(anchor: Option<Res<Anchor>>, terrain: Option<Res<Terrain>>) -> bool {
    alive(terrain) && anchor.is_some_and(|a| footing(&a).is_some())
}

/// Run condition: the anchor or the terrain changed this frame.
pub fn moved(anchor: Option<Res<Anchor>>, terrain: Option<Res<Terrain>>) -> bool {
    anchor.is_some_and(|a| a.is_changed()) || terrain.is_some_and(|t| t.is_changed())
}

/// Despawns every animal at once and forgets the herds, when the anchor left the planet they were
/// drawn on or its life is gone.
pub fn forget(
    mut commands: Commands,
    (anchor, terrain): (Option<Res<Anchor>>, Option<Res<Terrain>>),
    mut spec: ResMut<Spec>,
    mut herds: ResMut<Herds>,
    beasts: Query<Entity, With<Beast>>,
) {
    let here = anchor.and_then(|a| footing(&a));
    let same = spec.planet.as_ref().is_none_or(|p| Some(p.id) == here);
    if alive(terrain) && here.is_some() && same {
        return;
    }
    for beast in &beasts {
        commands.entity(beast).despawn();
    }
    herds.clear();
    spec.planet = None;
}
