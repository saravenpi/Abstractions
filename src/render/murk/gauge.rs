//! Where the water surface stands against the lens of the game camera, measured once a frame for
//! the underwater pass and the camera rig.

use bevy::prelude::*;
use bevy::render::RenderApp;
use bevy::render::extract_component::ExtractComponent;
use bevy::render::render_resource::ShaderType;

use super::super::GameCamera;
use super::super::fog::UNDERWATER_VISIBILITY;
use super::murk;
use crate::space::{Anchor, Frame};
use crate::world::{Planet, SEA_LEVEL};

/// Height in metres above the waterline below which part of the near plane of the camera can
/// be under the water: its half diagonal at the widest field of view, and a margin.
pub const LENS: f32 = 0.1;

/// Distance in metres from where the waterline was measured within which [`Waterline::above`]
/// still holds: farther, the anchor was rebased or the camera cut away.
const REACH: f32 = 20.0;

/// Radius in metres of the sea of a flat world, which has no planet around it.
const FLAT_RADIUS: f32 = 1.0e7;

/// Height in metres of the water surface above sea level under the game camera: the waves
/// raise and lower it. The sea writes it.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct Swell(pub f32);

/// The water surface under the game camera as the last frame measured it.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct Waterline {
    /// Render position of the camera where it was measured.
    pub at: Vec3,
    /// Unit vector away from the planet centre there, in render axes.
    pub up: Vec3,
    /// Height of the camera above the water surface there, in metres.
    pub height: f32,
    /// False on a planet without a sea and away from the surface of a planet.
    pub sea: bool,
}

impl Waterline {
    /// Returns the height in metres of render position `pos` above the water surface, or
    /// nothing without a sea or far from where the surface was measured.
    pub fn above(&self, pos: Vec3) -> Option<f32> {
        let near = self.sea && pos.distance(self.at) < REACH;
        near.then(|| self.height + self.up.dot(pos - self.at))
    }
}

/// What the underwater pass of the game camera reads: the water surface around the lens and
/// the liquid below it. Every pixel whose view starts under the surface is fogged in the colour
/// of the liquid over the part of its ray that runs under water.
#[derive(Component, ExtractComponent, Debug, Default, Clone, Copy, PartialEq, ShaderType)]
#[extract_app(RenderApp)]
pub struct Murk {
    /// Unit vector away from the planet centre at the camera, in render axes.
    pub up: Vec3,
    /// Height of the camera above the water surface, in metres.
    pub height: f32,
    /// Linear colour of the liquid.
    pub colour: Vec3,
    /// Fog density of the liquid, per metre.
    pub density: f32,
    /// Distance in metres from the planet centre to the water surface under the camera.
    pub radius: f32,
    /// 1 when part of the lens can be under the surface and the pass runs, else 0.
    pub under: f32,
}

impl Murk {
    /// Returns the pass of a camera `height` metres above the water surface of a sea of
    /// `radius` metres in the sRGB colour `liquid`, with `up` away from the planet centre.
    /// Without a liquid nothing is under water.
    pub fn of(liquid: Option<Vec3>, up: Vec3, height: f32, radius: f32) -> Self {
        let Some(srgb) = liquid else {
            return Self::default();
        };
        let linear = Color::srgb(srgb.x, srgb.y, srgb.z).to_linear();
        Self {
            up,
            height,
            colour: Vec3::new(linear.red, linear.green, linear.blue),
            density: 3.0 / UNDERWATER_VISIBILITY,
            radius,
            under: f32::from(u8::from(height < LENS)),
        }
    }

    /// Returns true when the camera itself is under the water surface.
    pub fn submerged(&self) -> bool {
        self.under > 0.0 && self.height < 0.0
    }
}

/// Returns the underwater pass of a camera at render position `camera` on the surface of
/// `planet` under `anchor`, with the waves `swell` metres above sea level under it.
fn read(anchor: &Anchor, planet: &Planet, camera: Vec3, swell: f32) -> Murk {
    let liquid = murk(planet.record());
    if anchor.flat {
        return Murk::of(liquid, Vec3::Y, camera.y - SEA_LEVEL - swell, FLAT_RADIUS);
    }
    let frame = anchor.to_frame(camera);
    let up = (anchor.rotation * frame.normalize()).as_vec3();
    let height = (frame.length() - planet.globe.radius()) as f32 - swell;
    Murk::of(liquid, up, height, planet.globe.radius() as f32 + swell)
}

/// Measures the water surface against the game camera after the transforms are propagated, for
/// the underwater pass of this frame and the camera rig of the next. Away from the surface of
/// the streamed planet nothing is under water.
fn gauge(
    swell: Res<Swell>,
    world: (Option<Res<Anchor>>, Option<Res<Planet>>),
    mut cameras: Query<(&GlobalTransform, &mut Murk), With<GameCamera>>,
    mut line: ResMut<Waterline>,
) {
    let found = world.0.as_deref().zip(world.1.as_deref());
    let here =
        found.filter(|(anchor, planet)| anchor.flat || anchor.frame == Frame::Surface(planet.body));
    for (at, mut murk) in &mut cameras {
        let camera = at.translation();
        let next = here.map_or_else(Murk::default, |(anchor, planet)| {
            read(anchor, planet, camera, swell.0)
        });
        murk.set_if_neq(next);
        line.set_if_neq(Waterline {
            at: camera,
            up: next.up,
            height: next.height,
            sea: next.density > 0.0,
        });
    }
}

/// Registers the measure of the water surface, after the transforms are propagated.
pub(in crate::render) fn plugin(app: &mut App) {
    app.init_resource::<Swell>()
        .init_resource::<Waterline>()
        .add_systems(
            PostUpdate,
            gauge
                .after(bevy::transform::TransformSystems::Propagate)
                .run_if(any_with_component::<GameCamera>),
        );
}
