//! Low-resolution game view, color banding, day cycle, sky, lighting, fog and camera effects.
//!
//! The sky is a real place: the sun stands where the star is, the air thins with height, and the
//! clouds, the stars and the other bodies of the system are drawn around the camera.

mod aerial;
mod air;
mod atmosphere;
mod bodies;
mod camera;
mod celestial;
mod clock;
mod clouds;
mod cloudshade;
mod daylight;
mod dome;
mod entry;
mod exposure;
mod fog;
mod galaxy_light;
mod glare;
mod glow;
mod heat;
mod lighting;
mod lowres;
mod moonlight;
mod moons;
mod murk;
mod nearby;
mod observer;
mod palette;
mod pixelate;
pub mod ranges;
mod scatter;
mod season;
mod seasons;
mod shade;
mod sky;
mod skylight;
mod starfield;
mod sun;
mod tune;
mod vantage;
mod warm;
mod warp;
mod weather;

#[cfg(test)]
mod schedule_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_air;
#[cfg(test)]
mod tests_dither;
#[cfg(test)]
mod tests_fog;
#[cfg(test)]
mod tests_fresnel;
#[cfg(test)]
mod tests_glare;
#[cfg(test)]
mod tests_lens;
#[cfg(test)]
mod tests_lowres;
#[cfg(test)]
mod tests_moonday;
#[cfg(test)]
mod tests_moons;
#[cfg(test)]
mod tests_murk;
#[cfg(test)]
mod tests_palette;
#[cfg(test)]
mod tests_rings;
#[cfg(test)]
mod tests_season;
#[cfg(test)]
mod tests_seasons;
#[cfg(test)]
mod tests_sky;
#[cfg(test)]
mod tests_skylight;
#[cfg(test)]
mod tests_weather;

pub use aerial::{Haze, sight};
pub use air::Air;
pub use camera::{configure_camera, set_fov};
pub use celestial::{daylight, smoothstep, sun_direction};
pub use clock::{DayClock, START_PHASE, catch_up};
pub use clouds::{CloudCover, CloudCrossing, CloudMaterial, CloudShell};
pub use cloudshade::CloudShade;
pub use daylight::{PlaceLight, PlaceLit, PlaceUniform, place_light};
pub use dome::SkyDome;
pub use entry::FlightFx;
pub use exposure::scene_lux;
pub use heat::{
    Craft, Gas, Glow, Heat, MIN_SPEED, SHEATH_FROM, SheathMaterial, SheathUniform, cool_speed,
    entry, skin,
};
pub use lighting::{Moon, Sun, reach};
pub use lowres::{Fit, LowRes, window_changed};
pub use murk::{Murk, Swell, Waterline, murk};
pub use observer::Observer;
pub use palette::Palette;
pub use pixelate::Pixelate;
pub use seasons::{SeasonLight, SeasonLit, SeasonUniform};
pub use starfield::Starfield;
pub use sun::SunDir;
pub use vantage::{LocalSystem, Vantage};
pub use warm::{WARM_FRAMES, Warm, Warmed, warm_material};
pub use warp::{Drive, Lens, Phase, Sequence};
pub use weather::{LocalWeather, WeatherLook};

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetBytesPerFrame;

/// Marker for the player's first-person 3D camera.
///
/// The render module adds post-processing, fog, field of view and effects to any entity with it.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct GameCamera;

/// Returns a unit sphere, turned inside out when it is only ever seen from within.
fn shell(inside: bool) -> Mesh {
    let mut mesh = Sphere::new(1.0).mesh().uv(48, 24);
    if inside && let Err(error) = mesh.invert_winding() {
        warn!("could not turn a sky shell inside out: {error}");
    }
    mesh
}

/// Writes a pose into both transforms of an entity without a parent.
///
/// The sky follows the camera after transform propagation, so the global transform is set by hand.
fn pin(pose: Transform, transform: &mut Transform, global: &mut GlobalTransform) {
    *transform = pose;
    *global = pose.into();
}

/// Registers the sky and the camera effects, which need neither a game view nor a GPU: the day
/// clock, the sun, the lights, the clouds, the stars, the bodies and the flight effects.
pub(super) fn sky(app: &mut App) {
    app.add_plugins((
        clock::plugin,
        sun::plugin,
        palette::plugin,
        lighting::plugin,
        sky::plugin,
        dome::plugin,
        camera::plugin,
        fog::plugin,
        warp::plugin,
        murk::gauge,
    ))
    .add_plugins((
        atmosphere::plugin,
        clouds::plugin,
        starfield::plugin,
        bodies::plugin,
        moonlight::plugin,
        entry::plugin,
        warm::plugin,
        seasons::plugin,
        shade::plugin,
    ));
    warm_material::<clouds::CloudMaterial>(app);
    warm_material::<sky::SkyMaterial>(app);
    warm_material::<atmosphere::HaloMaterial>(app);
    warm_material::<entry::FxMaterial>(app);
    warm_material::<starfield::StarMaterial>(app);
    warm_material::<bodies::BodyMaterial>(app);
    warm_material::<bodies::RingMaterial>(app);
}

/// Most bytes of meshes and images uploaded to the GPU in one frame. A burst of streamed chunks
/// spreads over a few frames instead of stalling one.
const UPLOAD_BYTES: usize = 16 << 20;

/// Registers the render plugins.
pub(super) fn plugin(app: &mut App) {
    app.insert_resource(RenderAssetBytesPerFrame::new(UPLOAD_BYTES))
        .add_plugins((
            pixelate::plugin,
            murk::pass,
            lowres::plugin,
            ranges::plugin,
            sky,
        ));
}
