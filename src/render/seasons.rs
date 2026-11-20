//! The state of the season of every planet, handed to the shaders that draw ground: the ground of the world
//! reads the block of the planet under the camera from a texture, and each ball of a planet seen from afar
//! carries the block of its own planet. Both turn a fragment by its own latitude and height with
//! `season.wesl`, so the colours of the ground are the same at every height of the camera.

use bevy::math::DQuat;
use bevy::prelude::*;
use bevy::render::render_resource::ShaderType;

use super::bodies::{Body, BodyMaterial};
use super::daylight::PlaceLit;
use super::season::{lean, window};
use super::vantage::{Scene, Vantage};
use crate::climate::{self, Setting};
use crate::space::Clock;
use crate::universe::{NO_AIR, PlanetSpec};
use crate::world::Link;

/// Uniform block of the season of one planet, read through `season.wesl`. Every field is a `vec4` so the
/// layout never needs padding. All zero, the planet has no season to show.
#[derive(Clone, Copy, Debug, Default, PartialEq, ShaderType)]
pub struct SeasonUniform {
    /// Spin axis in render axes in xyz, and 1 in w when the planet has a season to show.
    pub axis: Vec4,
    /// Mean temperature in kelvin, the pole to equator shape, the swing of the season per unit of the sine
    /// of the latitude, and the kelvin lost per metre of height.
    pub heat: Vec4,
    /// The lean of the sun path per unit of the sine of the latitude, autumn of the north, autumn of the
    /// south, and 1 where there is air for snow.
    pub leaf: Vec4,
    /// Direction of the warm side of a locked moon in render axes in xyz, and its extra kelvin in w.
    pub near: Vec4,
}

impl SeasonUniform {
    /// Fields of the block, each one texel where a shader reads it from a texture.
    pub const FIELDS: usize = 4;

    /// Bytes of the block as texels of four 32 bit floats.
    pub const BYTES: usize = 16 * Self::FIELDS;

    /// Returns the block of the season of `spec` at `time` seconds of universe time, where `to_render` turns
    /// the axes of the planet into render axes.
    pub fn of(spec: &PlanetSpec, time: f64, to_render: DQuat) -> Self {
        let setting = Setting::of(spec);
        let season = climate::season(spec, time);
        let heat = climate::terms(&setting, season);
        let render = |v: Vec3| (to_render * v.as_dvec3()).as_vec3();
        let air = f32::from(u8::from(setting.pressure >= NO_AIR as f32));
        Self {
            axis: render(Vec3::Y).extend(1.0),
            heat: Vec4::new(heat.mean, heat.shape, heat.swing, heat.lapse),
            leaf: Vec4::new(
                lean(setting.tilt),
                window(season),
                window((season + 0.5).rem_euclid(1.0)),
                air,
            ),
            near: render(heat.near).extend(heat.side),
        }
    }

    /// Returns the fields in the order of the block, which `season_from` in `season.wesl` reads texel by texel.
    pub fn fields(&self) -> [Vec4; Self::FIELDS] {
        [self.axis, self.heat, self.leaf, self.near]
    }

    /// Returns the bytes of the block as texels of four 32 bit floats.
    pub fn texels(&self) -> [u8; Self::BYTES] {
        let mut bytes = [0; Self::BYTES];
        let numbers = self.fields().into_iter().flat_map(|f| f.to_array());
        for (slot, number) in bytes.as_chunks_mut::<4>().0.iter_mut().zip(numbers) {
            *slot = number.to_le_bytes();
        }
        bytes
    }
}

#[cfg(test)]
impl SeasonUniform {
    /// Returns the look of the ground at the unit direction `dir` from the planet centre, `height` metres above
    /// its sea level, as `look_at` in `season.wesl` works it out.
    pub fn look(&self, dir: Vec3, height: f32) -> super::season::SeasonLook {
        use super::season::{SeasonLook, snow};
        if self.axis.w <= 0.0 {
            return SeasonLook::NONE;
        }
        let sine = dir.dot(self.axis.truncate()).clamp(-1.0, 1.0);
        let [mean, shape, swing, lapse] = self.heat.to_array();
        let side = self.near.w * dir.dot(self.near.truncate());
        let kelvin = mean + shape * (1.0 / 3.0 - sine * sine) + swing * sine + side
            - lapse * height.max(0.0);
        let window = if sine < 0.0 { self.leaf.z } else { self.leaf.y };
        SeasonLook {
            autumn: (self.leaf.x * sine.abs()).min(1.0) * window,
            snow: snow(kelvin) * self.leaf.w,
        }
    }
}

/// The season of the planet under the camera, all zero outside a world.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct SeasonLight(pub SeasonUniform);

/// The system that works out [`SeasonLight`]. Whatever hands it to a shader runs after this set.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SeasonLit;

/// Works out the season of the planet or moon under the camera, or none outside a world.
fn below(scene: Scene, mut light: ResMut<SeasonLight>) {
    let Scene {
        vantage,
        local,
        clock,
    } = scene;
    let spec = local
        .0
        .as_ref()
        .and_then(|near| near.body(u8::try_from(vantage.body).ok()?, vantage.moon));
    let observer = &vantage.observer;
    let block = spec.map_or(SeasonUniform::default(), |spec| {
        SeasonUniform::of(spec, clock.seconds, observer.to_render * observer.spin)
    });
    light.set_if_neq(SeasonLight(block));
}

/// Gives every planet and moon seen from afar the block of its own season, once per change.
fn planets(
    scene: Scene,
    bodies: Query<(&Body, &MeshMaterial3d<BodyMaterial>)>,
    mut materials: ResMut<Assets<BodyMaterial>>,
) {
    let to_render = scene.vantage.observer.to_render;
    let time = scene.clock.seconds;
    for (body, handle) in &bodies {
        let Body::Planet(id) = *body else {
            continue;
        };
        let Some((planet, _)) = scene.body(id) else {
            continue;
        };
        let block = SeasonUniform::of(planet, time, to_render * Link::orbiting(planet, time).turn);
        if materials
            .get(&handle.0)
            .is_some_and(|material| material.season != block)
            && let Some(mut material) = materials.get_mut(&handle.0)
        {
            material.season = block;
        }
    }
}

/// Embeds the season shader module and works out the seasons every frame the camera is tracked.
pub(super) fn plugin(app: &mut App) {
    if app
        .world()
        .contains_resource::<Assets<bevy::shader::Shader>>()
    {
        bevy::shader::load_shader_library!(app, "season.wesl");
    }
    app.init_resource::<SeasonLight>().add_systems(
        PostUpdate,
        (
            below.in_set(SeasonLit).after(PlaceLit),
            planets
                .after(super::sun::track)
                .run_if(resource_exists::<Assets<BodyMaterial>>),
        )
            .run_if(resource_exists::<Clock>.and_then(resource_exists::<Vantage>)),
    );
}
