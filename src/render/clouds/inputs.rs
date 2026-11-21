//! What the cloud layer reads from its inputs each frame: the planet it draws, the cloud over the
//! camera, and the shader values of a slice.

use bevy::prelude::*;

use super::map::toward_second;
use super::material::CloudUniform;
use super::tune::{BASE_OCTAVES, UPPER_OCTAVES};
use super::{CELL, CloudInputs};
use crate::render::aerial::Haze;
use crate::render::cloudshade::fixed;
use crate::render::sky::linear;
use crate::render::vantage::Vantage;
use crate::universe::BodyId;
use crate::weather::{Field, density_at, drift as drifted, slice};

impl CloudInputs<'_> {
    /// Returns the planet or moon under the camera when its sky has a deck of clouds and
    /// clouds are switched on.
    pub(super) fn cloudy(&self) -> Option<BodyId> {
        let vantage = &*self.vantage;
        let near = self.local.0.as_ref()?;
        let body = near.body(u8::try_from(vantage.body).ok()?, vantage.moon)?;
        let on = self.settings.graphics.clouds && Field::of(body, false).deck;
        on.then_some(body.id)
    }

    /// Returns true while the camera is nearer the body than its hand-over, past which the far
    /// body draws the clouds.
    pub(super) fn within(&self, vantage: &Vantage) -> bool {
        let limit = self.local.0.as_ref().map(|near| near.handover(vantage));
        limit.is_none_or(|limit| vantage.observer.altitude(0.0) < limit)
    }

    /// Returns how much cloud the field holds where the camera is, on a shell of `radius`
    /// metres: the puff the shader draws there, under the broad cover the weather reads.
    pub(super) fn density(&self, radius: f32) -> f32 {
        let cover = self
            .forced
            .0
            .unwrap_or_else(|| self.weather.as_deref().map_or(0.0, |weather| weather.cloud));
        let dir = (fixed(&self.vantage) * self.vantage.up).as_dvec3();
        density_at(dir, radius / CELL, self.clock.seconds, cover)
    }

    /// Returns the shader values for slice `k` of radius `radius` metres, whose field is laid on
    /// the shell of `mid` metres at the middle of the band, for a camera `depth` deep in cloud.
    pub(super) fn uniform(&self, mid: f32, (k, radius): (u32, f32), depth: f32) -> CloudUniform {
        let vantage = &*self.vantage;
        let (fixed, seconds) = (fixed(vantage), self.clock.seconds);
        let air = self.local.air_below(vantage);
        CloudUniform {
            light: self.palette.sun_dir.extend(vantage.air),
            color: linear(self.palette.cloud)
                .truncate()
                .extend(toward_second(seconds)),
            haze: linear(self.palette.horizon).truncate().extend(depth),
            params: Vec4::new(
                mid / CELL,
                0.0,
                f32::from(u8::from(self.forced.0.is_some())),
                self.forced.0.unwrap_or(0.0),
            ),
            space: air.clouds_from_space().extend(0.0),
            ambient: linear(self.palette.ambient).truncate().extend(0.0),
            fade: Haze::seen(vantage, &air, self.sky.as_deref(), radius).extend(0.0),
            drift: drifted(seconds, mid / CELL),
            fixed_x: fixed.x_axis.extend(0.0),
            fixed_y: fixed.y_axis.extend(0.0),
            fixed_z: fixed.z_axis.extend(0.0),
            eye: (fixed * vantage.up * (mid / CELL)).extend(CELL),
            slice: Vec2::new(slice(k), [BASE_OCTAVES, UPPER_OCTAVES][usize::from(k > 0)])
                .extend(0.0)
                .extend(0.0),
        }
    }
}
