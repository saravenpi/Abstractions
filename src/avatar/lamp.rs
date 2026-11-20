//! The shadows of the lamp of the robot the player plays: only that lamp casts them, and only
//! when the lighting setting allows it, so a crowd of players does not cost a cube of shadow
//! maps each.

use bevy::prelude::*;

use super::own::Own;
use super::spawn::Lamp;
use crate::settings::{Persistent, Settings};

/// Returns true when the lamp shadow setting changed or a lamp was just spawned.
pub fn changed(settings: Res<Persistent<Settings>>, fresh: Query<(), Added<Lamp>>) -> bool {
    settings.is_changed() || !fresh.is_empty()
}

/// Lets the own lamp cast shadows when the setting allows it, and no other lamp.
pub fn shadows(
    settings: Res<Persistent<Settings>>,
    own: Own,
    mut lamps: Query<(&Lamp, &mut PointLight)>,
) {
    let mine = own.body().map(|(_, body)| body);
    let allowed = settings.graphics.lighting.local_shadows;
    for (lamp, mut light) in &mut lamps {
        let wanted = allowed && mine == Some(lamp.0);
        if light.shadow_maps_enabled != wanted {
            light.shadow_maps_enabled = wanted;
        }
    }
}
