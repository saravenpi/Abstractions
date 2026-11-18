//! The wetness of the ground under the cover grid: what the cloud field takes from the climate.

use bevy::math::DVec3;

use super::grid::Patch;
use crate::climate;
use crate::world::Globe;

/// Returns the wetness, from 0 to 1, of the climate at the ground of `globe` in unit direction
/// `dir`: what the broad cover over that place is raised by. Every reader of the cloud field
/// takes it from here.
///
/// The ground is read with [`Globe::glance`], which gives the measures of [`Globe::air`] from
/// three ground samples and no region grid: the cover map asks for every texel of a whole
/// planet, which would push the ground under the player out of the grid's memory.
pub fn wet(globe: &Globe, dir: DVec3) -> f32 {
    let (_, ground) = globe.glance(dir);
    climate::at(globe.setting(), &ground, globe.season())
        .moisture
        .clamp(0.0, 1.0)
}

/// The wetness at the four texels of the last patch asked for, kept so that the ground is
/// read again only when the camera moves on to another texel of the grid.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Wet {
    /// Face and first texel of the patch the values belong to.
    at: Option<(usize, [i32; 2])>,
    /// Wetness at the four texels of that patch.
    values: [f32; 4],
}

impl Wet {
    /// Returns the wetness of the ground of `globe` at the four texels of `patch`.
    pub fn over(&mut self, globe: &Globe, patch: &Patch) -> [f32; 4] {
        let at = Some((patch.face, patch.corner));
        if self.at != at {
            self.at = at;
            self.values = patch.dirs().map(|dir| wet(globe, dir));
        }
        self.values
    }
}
