//! The light of the planet under the camera, worked out once per frame for every shader that
//! lights a place of it: the ground, the sea and the ball the planet turns into from afar.

use bevy::prelude::*;

use super::ambient::with_ground;
use super::{PlaceUniform, Sky};
use crate::render::cloudshade::{CloudShade, fixed};
use crate::render::moonlight::{Moonlight, STARLIGHT};
use crate::render::sun::SunDir;
use crate::render::vantage::{LocalSystem, Vantage};

/// Linear grey of the ground of a place whose body is not known, a mean land albedo.
const GROUND: f32 = 0.2;

/// The uniform block of the light of the planet under the camera, all zero outside a world.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct PlaceLight(pub PlaceUniform);

/// The systems that work out [`PlaceLight`]. Whatever hands it to a shader runs after this set.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlaceLit;

/// Returns the ambient brightness the stars add to a complete night under `moon`: all of
/// [`STARLIGHT`] without a moon, less as the moon takes over.
pub fn starlight(moon: &Moonlight) -> f32 {
    STARLIGHT * (1.0 - moon.0.map_or(0.0, |glow| glow.share.min(1.0)))
}

/// Returns the sky of the body under the camera of `vantage` in the system `local`, on a night
/// lit by `moon`, or nothing outside a world.
pub fn sky_below(vantage: &Vantage, local: &LocalSystem, moon: &Moonlight) -> Option<Sky> {
    let star = local.0.as_ref()?.system.star.color;
    let air = local.air_below(vantage);
    let size = (vantage.radius, vantage.atmosphere);
    Some(Sky::of(star, &air, size, starlight(moon)))
}

/// Returns the uniform block for a camera at `vantage` that sees the star as `sun` does: the
/// planet centre in render space, the star, the air, the light the rule gives the camera, and the
/// height of the camera, worked out in 64 bits, that the shader measures every other height from.
pub fn block(sky: &Sky, vantage: &Vantage, sun: &SunDir) -> PlaceUniform {
    let observer = &vantage.observer;
    let below = observer.center - observer.eye;
    let down = (observer.to_render * below).as_vec3();
    let place = sky.uniform(
        vantage.camera + down,
        sun.dir,
        (sun.height, vantage.altitude),
    );
    let high = below.length() - f64::from(sky.radius);
    place.seen_from(vantage.camera, vantage.up, high)
}

/// Works out the light of the planet under the camera for this frame, once the camera, the star
/// and the moon are tracked.
pub fn place_light(
    place: (Res<Vantage>, Res<LocalSystem>, Res<SunDir>),
    (moon, shade): (Res<Moonlight>, Res<CloudShade>),
    mut light: ResMut<PlaceLight>,
) {
    let (vantage, local, sun) = place;
    let cloud = shade.eye(fixed(&vantage) * vantage.up);
    let sky = sky_below(&vantage, &local, &moon);
    let ground = local
        .below(&vantage)
        .map_or(Vec3::splat(GROUND), |body| body.record.class.look().land);
    let block = sky.map_or(PlaceUniform::default(), |sky| {
        let lit = block(&sky, &vantage, &sun);
        let dimmed = lit.eye_sun.truncate() * cloud;
        with_ground(
            PlaceUniform {
                eye_sun: dimmed.extend(lit.eye_sun.w),
                ..lit
            },
            ground,
        )
    });
    light.set_if_neq(PlaceLight(block));
}
