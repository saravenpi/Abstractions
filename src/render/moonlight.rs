//! Moonlight: the night light comes from the real moons, from where they stand and as full as
//! they are. A planet without moons has none, and its nights are lit by the stars alone.

use bevy::prelude::*;

use super::moons::{Glow, moonlight};
use super::palette::Palette;
use super::vantage::Scene;

/// Ambient brightness the stars add on a night no moon lights, on top of
/// [`Palette::NIGHT_AMBIENT`]: together 100, three quarters of what the old painted moon and
/// the night ambient gave a floor facing it (about 130). With 25 the ground showed almost black
/// in the frames, a quarter as bright on screen as under the painted moon; with 60 it shows a
/// little over half as bright, so the ground and the way ahead still read on foot, and a lamp,
/// the light of the companion or a glowing sea stand out more than they did.
pub const STARLIGHT: f32 = 60.0;

/// The real body that lights the night of the camera, or nothing on a planet without moons.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct Moonlight(pub Option<Glow>);

/// Returns `palette` lit at night by `light`: the moon stands where the real one is and gives the
/// share of a full moon it is worth, and the stars fill in what it does not give. Without a
/// real moon there is no moon light at all, only the stars. The day is left as it is.
pub fn moonlit(mut palette: Palette, light: &Moonlight) -> Palette {
    let night = palette.moon_lux / Palette::MOON_LUX;
    let share = light.0.map_or(0.0, |glow| glow.share);
    if let Some(glow) = light.0 {
        palette.moon_dir = glow.dir;
    }
    palette.moon_lux *= share;
    palette.ambient_brightness += STARLIGHT * night * (1.0 - share.min(1.0));
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
