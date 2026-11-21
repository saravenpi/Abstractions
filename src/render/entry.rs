//! What the flight effects read, and the light of a warp around the view: the light ahead and
//! the flash of its drop-out (the stars themselves streak, the camera rig opens the view).
//!
//! Nothing of flight through air is drawn around the camera: the glow of an entry is the sheath
//! of `heat` at the craft, a cloud is the whiteout of the fog from the cloud field at the camera,
//! and speed shows in the ground, the clouds, the rain and the dust passing at their places, in
//! the shake and in the field of view.

use bevy::asset::embedded_asset;
use bevy::camera::visibility::NoFrustumCulling;
use bevy::light::NotShadowCaster;
use bevy::pbr::{Material, MaterialPlugin};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;

use super::GameCamera;
use super::glow::stream_heading;
use super::vantage::Vantage;
use super::warp::Lens;
use crate::screens::Screen;

/// Radius in metres of the shell the effects are drawn on, around the camera and the ship.
const SHELL_RADIUS: f32 = 60.0;

/// Linear colour of the light ahead in a warp and of the flash of its drop-out: blue-white.
const CORE_COLOR: Vec3 = Vec3::new(0.8, 0.9, 1.3);

/// What the flight effects are drawn from. The ship writes it; everything is zero on foot.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct FlightFx {
    /// Dynamic pressure of the air on the hull in pascals: half the density times the speed squared.
    pub dynamic_pressure: f32,
    /// How far the warp drive is on: 0 when off, below 1 while it spools, 1 once it is engaged.
    /// The warp sequence of the sky follows it.
    pub warp: f32,
    /// Speed of an engaged warp drive as a share of its top speed, from 0 to 1.
    pub pace: f32,
    /// Speed in metres per second; 0 lets the sky measure the camera speed itself.
    pub speed: f32,
    /// Unit vector along the nose of the ship in render axes, which the warp streams from. Zero
    /// lets the effects stream from where the camera looks.
    pub nose: Vec3,
}

/// Uniform block read by `entry.wesl`. Every field is a `vec4` so the layout never needs padding.
#[derive(Clone, Copy, Debug, Default, PartialEq, ShaderType)]
pub struct FxUniform {
    /// Unit vector of the direction of travel, in render axes.
    pub heading: Vec4,
    /// Linear colour of the light ahead in a warp in xyz and its strength in w.
    pub core: Vec4,
    /// Strength of the flash of a warp drop-out in x.
    pub flash: Vec4,
}

impl FxUniform {
    /// Returns the light of a warp for `lens`, streaming along `heading`.
    pub fn of(lens: &Lens, heading: Vec3) -> Self {
        Self {
            heading: heading.extend(0.0),
            core: CORE_COLOR.extend(lens.core),
            flash: Vec4::new(lens.flash, 0.0, 0.0, 0.0),
        }
    }

    /// Returns true while there is a warp light to draw: never in flight through air.
    pub fn lit(&self) -> bool {
        self.core.w + self.flash.x > 0.0
    }
}

/// Material of the effect shell.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
pub struct FxMaterial {
    /// Values for the shader.
    #[uniform(0)]
    pub fx: FxUniform,
}

impl Material for FxMaterial {
    /// Points at the shader embedded in the binary.
    fn fragment_shader() -> ShaderRef {
        "embedded://abstractions/render/entry.wesl".into()
    }

    /// The effects add light to the view.
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Add
    }

    /// The shell stays out of the depth prepass.
    fn enable_prepass() -> bool {
        false
    }
}

/// Marks the effect shell.
#[derive(Component, Debug, Clone, Copy)]
pub struct FxShell;

/// Spawns the hidden shell when the world starts loading.
fn spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<FxMaterial>>,
) {
    commands.spawn((
        FxShell,
        Mesh3d(meshes.add(super::shell(true))),
        MeshMaterial3d(materials.add(FxMaterial::default())),
        Transform::default(),
        Visibility::Hidden,
        NotShadowCaster,
        NoFrustumCulling,
        DespawnOnEnter(Screen::Title),
    ));
}

/// Keeps the shell around the camera and shows it only while a warp lights it, streaming from
/// the nose of the ship or from where the camera looks.
fn follow(
    (flight, vantage, lens): (Res<FlightFx>, Res<Vantage>, Res<Lens>),
    cameras: Query<&GlobalTransform, (With<GameCamera>, Without<FxShell>)>,
    mut shells: Query<
        (
            &mut Transform,
            &mut GlobalTransform,
            &mut Visibility,
            &MeshMaterial3d<FxMaterial>,
        ),
        With<FxShell>,
    >,
    mut materials: ResMut<Assets<FxMaterial>>,
) {
    let looking = cameras
        .iter()
        .next()
        .map_or(Vec3::NEG_Z, |camera| *camera.forward());
    let nose = flight.nose.try_normalize().unwrap_or(looking);
    let ahead = if lens.level > 0.0 { lens.heading } else { nose };
    let fx = FxUniform::of(&lens, stream_heading(vantage.velocity, ahead, lens.level));
    let shown = fx.lit();
    let pose = Transform::from_translation(vantage.camera).with_scale(Vec3::splat(SHELL_RADIUS));
    for (mut transform, mut global, mut visibility, handle) in &mut shells {
        super::pin(pose, &mut transform, &mut global);
        visibility.set_if_neq(match shown {
            true => Visibility::Visible,
            false => Visibility::Hidden,
        });
        if shown && let Some(mut material) = materials.get_mut(&handle.0) {
            material.fx = fx;
        }
    }
}

/// Embeds the shader, registers the material and lights the shell once the warp is known.
pub(super) fn plugin(app: &mut App) {
    embedded_asset!(app, "entry.wesl");
    app.add_plugins((MaterialPlugin::<FxMaterial>::default(), super::heat::plugin))
        .init_resource::<FlightFx>()
        .add_systems(OnEnter(Screen::Loading), spawn)
        .add_systems(PostUpdate, follow.after(super::warp::run));
}
