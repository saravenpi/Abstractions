//! The shadows of the clouds on the ground, the plants and the props.
//!
//! The clouds are a field of puffs on a shell round the planet. Once a second a patch of ground
//! around the camera is worked out on the CPU ([`Cast`]): at each of 32 by 32 texels, the share
//! of the sun that the puff above it along the sun ray lets through. The patch rides in the light
//! block texture of the ground shader, after the fields of the place block, so the ground, the
//! grass and the props darken under the real clouds without another binding. The patch lives
//! in axes fixed to the planet, so it follows the ground when the planet turns and when the
//! render origin moves, and the puffs drift under it with the wind.
//!
//! The camera takes its own share through [`CloudShade::eye`]: the light of the place at the camera
//! is dimmed by it, so the lights of the scene, the exposure and the ground shader all agree.
//! The sea is not shaded: its material binds the place block as a uniform with no room for
//! the patch.

mod cast;
mod tune;

#[cfg(test)]
mod tests;

use std::time::Duration;

use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;

pub use self::cast::Cast;
use self::tune::{EDGE_FADE, HEAD, PERIOD, REACH, SIDE, TEXELS};
use super::{LocalSystem, SunDir, Vantage};
use crate::settings::{Persistent, Settings};
use crate::space::Clock;
use crate::weather::{Field, Weather};

/// The patch of cloud shadow around the camera, as the ground shader reads it.
#[derive(Resource, Debug, Clone, PartialEq)]
pub struct CloudShade {
    /// The header texels: the rotation from render axes to planet axes in three, the centre of
    /// the patch with its half side in metres in the fourth, then its two axes. All zero means
    /// no patch, and the shader takes the sun as it is.
    pub head: [Vec4; HEAD],
    /// The shares of the sun, row by row.
    pub grid: Vec<f32>,
    /// Sea level radius in metres of the planet the patch is on.
    pub radius: f32,
    /// Counts the works of the patch, so a writer copies it only when it changed.
    pub revision: u32,
}

impl Default for CloudShade {
    fn default() -> Self {
        Self {
            head: [Vec4::ZERO; HEAD],
            grid: vec![1.0; SIDE * SIDE],
            radius: 0.0,
            revision: 0,
        }
    }
}

impl CloudShade {
    /// Texels the patch adds to the light block of the ground shader.
    pub const TEXELS: usize = TEXELS;

    /// Writes the header and the grid as the texels the shader reads, four 32 bit floats each,
    /// into `out`, which is [`Self::TEXELS`] texels long.
    pub fn write(&self, out: &mut [u8]) {
        let head = self.head.iter().flat_map(|texel| texel.to_array());
        let grid = self.grid.iter().flat_map(|share| [*share, 0.0, 0.0, 1.0]);
        for (slot, number) in out.as_chunks_mut::<4>().0.iter_mut().zip(head.chain(grid)) {
            *slot = number.to_le_bytes();
        }
    }

    /// Returns the share of the sun the clouds let through at the unit direction `dir` from the
    /// planet centre, in axes fixed to the planet: what the shader reads, and what the light of
    /// the camera takes. 1 outside the patch and without one.
    pub fn eye(&self, dir: Vec3) -> f32 {
        let reach = self.head[3].w;
        let (centre, east, north) = (self.head[3].truncate(), self.head[4], self.head[5]);
        if reach <= 0.0 {
            return 1.0;
        }
        let along = dir.dot(centre).max(0.05);
        let across = Vec2::new(dir.dot(east.truncate()), dir.dot(north.truncate()));
        let uv = across * self.radius / along / reach * 0.5 + Vec2::splat(0.5);
        let edge = (uv * 2.0 - Vec2::ONE).abs().max_element();
        if edge >= 1.0 {
            return 1.0;
        }
        let g = uv * SIDE as f32 - Vec2::splat(0.5);
        let (i, f) = (g.floor(), g - g.floor());
        let cell = |dx: f32, dy: f32| {
            let at = |v: f32| (v as i32).clamp(0, SIDE as i32 - 1) as usize;
            self.grid[at(i.y + dy) * SIDE + at(i.x + dx)]
        };
        let low = cell(0.0, 0.0) * (1.0 - f.x) + cell(1.0, 0.0) * f.x;
        let high = cell(0.0, 1.0) * (1.0 - f.x) + cell(1.0, 1.0) * f.x;
        let share = low * (1.0 - f.y) + high * f.y;
        let fade = 1.0 - crate::render::smoothstep(EDGE_FADE, 1.0, edge);
        1.0 + (share - 1.0) * fade
    }
}

/// Returns the rotation from render axes to the axes fixed to the planet under the camera,
/// the same the clouds are laid in.
pub fn fixed(vantage: &Vantage) -> Mat3 {
    let observer = &vantage.observer;
    Mat3::from_quat((observer.to_render * observer.spin).inverse().as_quat())
}

/// Returns what the patch is worked out from, or nothing when the clouds are off, the planet
/// has no deck of clouds, or no world is there.
fn want(
    (vantage, local, sun): (&Vantage, &LocalSystem, &SunDir),
    (clock, weather): (&Clock, Option<&Weather>),
    settings: &Settings,
) -> Option<Cast> {
    let graphics = &settings.graphics;
    let on = graphics.clouds && graphics.lighting.cloud_shade;
    let body = local
        .below(vantage)
        .filter(|body| on && Field::of(body, false).deck)?;
    let to_fixed = fixed(vantage);
    let centre = (to_fixed * vantage.up).normalize_or(Vec3::Y);
    let east = Vec3::Y.cross(centre).try_normalize().unwrap_or(Vec3::X);
    Some(Cast {
        centre,
        east,
        north: centre.cross(east),
        radius: body.radius as f32,
        shell: body.radius as f32 + 0.5 * (vantage.cloud_base + vantage.cloud_top),
        sun: to_fixed * sun.dir,
        seconds: clock.seconds,
        cover: weather.map_or(0.0, |weather| weather.cloud),
    })
}

/// Works out the patch once a period, or clears it when the clouds or their shadows are off.
fn refresh(
    sky: (Res<Vantage>, Res<LocalSystem>, Res<SunDir>),
    (clock, weather): (Res<Clock>, Option<Res<Weather>>),
    settings: Res<Persistent<Settings>>,
    mut shade: ResMut<CloudShade>,
) {
    let cast = want(
        (&sky.0, &sky.1, &sky.2),
        (&clock, weather.as_deref()),
        &settings,
    );
    let Some(cast) = cast else {
        if shade.head[3].w != 0.0 {
            *shade = CloudShade {
                revision: shade.revision + 1,
                ..default()
            };
        }
        return;
    };
    let to_fixed = fixed(&sky.0);
    let head = [
        to_fixed.x_axis.extend(0.0),
        to_fixed.y_axis.extend(0.0),
        to_fixed.z_axis.extend(0.0),
        cast.centre.extend(REACH),
        cast.east.extend(0.0),
        cast.north.extend(0.0),
    ];
    let revision = shade.revision + 1;
    *shade = CloudShade {
        head,
        grid: cast.grid(),
        radius: cast.radius,
        revision,
    };
}

/// Registers the patch and its work, which runs once a period.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<CloudShade>().add_systems(
        PostUpdate,
        refresh
            .after(super::sun::track)
            .run_if(on_timer(Duration::from_secs_f32(PERIOD))),
    );
}
