//! Moonlight: under a planet with moons the night light comes from the real moons, from where
//! they stand and as full as they are; a planet without moons keeps the moon opposite the star.

use bevy::prelude::*;

use super::moons::{Glow, moonlight};
use super::palette::Palette;
use super::vantage::Scene;

/// The real body that lights the night of the camera, or nothing on a planet without moons.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct Moonlight(pub Option<Glow>);

/// Returns `palette` lit at night by `light`: its moon stands where the real one is and gives
/// the share of the moon light it is worth. Without a real moon the palette is left as it is.
pub fn moonlit(mut palette: Palette, light: &Moonlight) -> Palette {
    if let Some(glow) = light.0 {
        palette.moon_dir = glow.dir;
        palette.moon_lux *= glow.share;
    }
    palette
}

/// Finds the real body that lights the night of the camera this frame.
pub(super) fn shine(scene: Scene, mut light: ResMut<Moonlight>) {
    let near = scene.local.0.as_ref();
    let glow = near.and_then(|near| moonlight(&scene.vantage, near, scene.clock.seconds));
    light.set_if_neq(Moonlight(glow));
}

/// Registers the moonlight and finds it once the camera is tracked.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Moonlight>()
        .add_systems(PostUpdate, shine.after(super::sun::track));
}
