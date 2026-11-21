//! The shadows of the clouds on the ground and the grass.
//!
//! The clouds are a field of puffs on a shell round the planet. The ground shader works out the
//! shadow at every pixel, every frame: the ray from the ground point toward the star runs to the
//! shell and reads the very puff field the shell draws, at the same second, with the same turn of
//! the wind (`cloud_shade` in `daylight.wesl`). So the shadow is under its cloud, moves with it,
//! and has no grid to step or shimmer. This resource only hands the shader what it reads: the
//! planet axes, the shell, the cover and the turn, in the light block texture of the ground
//! shader after the fields of the place block, so no other binding is needed.
//!
//! The camera takes its own share through [`CloudShade::eye`], worked out the same way on the CPU:
//! the light of the place at the camera is dimmed by it, so the lights of the scene, the exposure
//! and the ground shader all agree. The sea is not shaded: its material binds the place block as
//! a uniform with no room for the shell.
//!
//! Cost: one ray to the shell and four octaves of the noise per ground and grass pixel while the
//! shadows are on, nothing while they are off.

mod cast;
mod tune;

#[cfg(test)]
mod rig;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_follow;

use bevy::prelude::*;

pub use self::cast::{Cast, sunward_hit};
use self::tune::HEAD;
use super::clouds::CloudCover;
use super::{LocalSystem, SunDir, Vantage};
use crate::settings::{Persistent, Settings};
use crate::space::Clock;
use crate::weather::{CELL, Field, Weather, drift};

/// What the ground shader reads to shade the ground under the clouds.
#[derive(Resource, Debug, Default, Clone, PartialEq)]
pub struct CloudShade {
    /// The header texels: the rotation from render axes to planet axes in three, then the radius
    /// of the shell in metres, its noise cells per radius, the broad cover and 1 when the shadows
    /// are on, then the turn of the puffs. All zero means no shadows, and the shader takes the
    /// sun as it is.
    pub head: [Vec4; HEAD],
    /// What the shadow at the camera is worked out from, while the shadows are on.
    pub cast: Option<Cast>,
    /// Counts the changes of the header, so a writer copies it only when it changed.
    pub revision: u32,
}

impl CloudShade {
    /// Texels the shadows add to the light block of the ground shader.
    pub const TEXELS: usize = HEAD;

    /// Writes the header as the texels the shader reads, four 32 bit floats each, into `out`,
    /// which is [`Self::TEXELS`] texels long.
    pub fn write(&self, out: &mut [u8]) {
        let head = self.head.iter().flat_map(|texel| texel.to_array());
        for (slot, number) in out.as_chunks_mut::<4>().0.iter_mut().zip(head) {
            *slot = number.to_le_bytes();
        }
    }

    /// Returns the share of the sun the clouds let through to the ground under the unit direction
    /// `dir` from the planet centre, in axes fixed to the planet: what the shader works out there,
    /// and what the light of the camera takes. 1 while the shadows are off.
    pub fn eye(&self, dir: Vec3) -> f32 {
        self.cast.map_or(1.0, |cast| cast.through(dir))
    }
}

/// Returns the rotation from render axes to the axes fixed to the planet under the camera,
/// the same the clouds are laid in.
pub fn fixed(vantage: &Vantage) -> Mat3 {
    let observer = &vantage.observer;
    Mat3::from_quat((observer.to_render * observer.spin).inverse().as_quat())
}

/// Returns what the shadows are worked out from, or nothing when the clouds or their shadows are
/// off, the planet has no deck of clouds, or no world is there. The cover is the one the clouds
/// over the camera are drawn with: the one the probe forces, else the weather's.
fn want(
    (vantage, local, sun): (&Vantage, &LocalSystem, &SunDir),
    (clock, cover): (&Clock, f32),
    settings: &Settings,
) -> Option<Cast> {
    let graphics = &settings.graphics;
    let on = graphics.clouds && graphics.lighting.cloud_shade;
    let body = local
        .below(vantage)
        .filter(|body| on && Field::of(body, false).deck)?;
    Some(Cast {
        radius: body.radius as f32,
        shell: vantage.radius + 0.5 * (vantage.cloud_base + vantage.cloud_top),
        sun: fixed(vantage) * sun.dir,
        seconds: clock.seconds,
        cover,
    })
}

/// Returns the header the shader reads for `cast` seen from `vantage`.
fn header(vantage: &Vantage, cast: &Cast) -> [Vec4; HEAD] {
    let to_fixed = fixed(vantage);
    let cells = cast.shell / CELL;
    [
        to_fixed.x_axis.extend(0.0),
        to_fixed.y_axis.extend(0.0),
        to_fixed.z_axis.extend(0.0),
        Vec4::new(cast.shell, cells, cast.cover, 1.0),
        drift(cast.seconds, cells),
    ]
}

/// Hands the shader the shell and the turn of the puffs for this frame, the same the clouds are
/// drawn with, or clears them when the clouds or their shadows are off.
fn refresh(
    sky: (Res<Vantage>, Res<LocalSystem>, Res<SunDir>),
    (clock, weather, forced): (Res<Clock>, Option<Res<Weather>>, Res<CloudCover>),
    settings: Res<Persistent<Settings>>,
    mut shade: ResMut<CloudShade>,
) {
    let cover = forced
        .0
        .unwrap_or_else(|| weather.as_deref().map_or(0.0, |weather| weather.cloud));
    let cast = want((&sky.0, &sky.1, &sky.2), (&clock, cover), &settings);
    let head = cast.map_or([Vec4::ZERO; HEAD], |cast| header(&sky.0, &cast));
    if shade.head == head && shade.cast == cast {
        return;
    }
    let revision = shade.revision.wrapping_add(1);
    *shade = CloudShade {
        head,
        cast,
        revision,
    };
}

/// Registers the shadows, handed over every frame the clock or the camera moves, before the
/// light of the place is worked out from them.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<CloudShade>()
        .init_resource::<CloudCover>()
        .add_systems(
            PostUpdate,
            refresh
                .after(super::sun::track)
                .before(super::PlaceLit)
                .run_if(
                    resource_changed::<Clock>
                        .or_else(resource_changed::<Vantage>)
                        .or_else(resource_changed::<Persistent<Settings>>),
                ),
        );
}
