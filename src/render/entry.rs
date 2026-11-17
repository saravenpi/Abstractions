//! Flight effects around the view: the glow of re-entry, warp streaks (the camera rig opens the view),
//! and cloud wisps streaking past inside the cloud layer.

use bevy::asset::embedded_asset;
use bevy::camera::visibility::NoFrustumCulling;
use bevy::ecs::system::SystemParam;
use bevy::light::NotShadowCaster;
use bevy::pbr::{Material, MaterialPlugin};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;

use super::GameCamera;
use super::celestial::smoothstep;
use super::clouds::CloudCrossing;
use super::glow::{entry_glow, stream_heading};
use super::palette::Palette;
use super::sky::linear;
use super::vantage::Vantage;
use crate::screens::Screen;

/// Radius in metres of the shell the effects are drawn on, around the camera and the ship.
const SHELL_RADIUS: f32 = 60.0;

/// Speed in metres per second at which cloud wisps start to streak and at which they are full.
const WISP_SPEED: (f32, f32) = (8.0, 80.0);

/// Brightness of the re-entry glow at the rim of the view. It stays a rim: a dive under power
/// is fast and deep in the air too, and a brighter glow washed the whole view out to white.
const GLOW_GAIN: f32 = 0.7;

/// What the flight effects are drawn from. The ship writes it; everything is zero on foot.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct FlightFx {
    /// Dynamic pressure of the air on the hull in pascals: half the density times the speed squared.
    pub dynamic_pressure: f32,
    /// How far the warp drive is engaged, from 0 to 1.
    pub warp: f32,
    /// Speed in metres per second; 0 lets the sky measure the camera speed itself.
    pub speed: f32,
    /// Unit vector along the nose of the ship in render axes, which the warp streams from. Zero
    /// lets the effects stream from where the camera looks.
    pub nose: Vec3,
}

/// Uniform block read by `entry.wesl`. Every field is a `vec4` so the layout never needs padding.
#[derive(Clone, Copy, Debug, Default, ShaderType)]
pub struct FxUniform {
    /// Unit vector of the direction of travel, in render axes.
    pub heading: Vec4,
    /// Linear glow colour in xyz and its strength in w.
    pub glow: Vec4,
    /// Linear warp streak colour in xyz and its strength in w.
    pub streak: Vec4,
    /// Linear wisp colour in xyz and its strength in w.
    pub wisp: Vec4,
    /// Time in seconds, streak rate, wisp rate and how far the streaks stretch from 0 to 1.
    pub params: Vec4,
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

/// Everything the effect values are computed from.
#[derive(SystemParam)]
struct FxInputs<'w, 's> {
    flight: Res<'w, FlightFx>,
    vantage: Res<'w, Vantage>,
    crossing: Res<'w, CloudCrossing>,
    palette: Res<'w, Palette>,
    time: Res<'w, Time>,
    cameras: Query<'w, 's, &'static GlobalTransform, (With<GameCamera>, Without<FxShell>)>,
}

impl FxInputs<'_, '_> {
    /// Returns the shader values: the glow grows with dynamic pressure while sinking, the streaks with the warp
    /// and the wisps with the depth in the clouds and the speed, all streaming from [`stream_heading`].
    fn uniform(&self) -> FxUniform {
        let flight = *self.flight;
        let velocity = self.vantage.velocity;
        let speed = if flight.speed > 0.0 {
            flight.speed
        } else {
            velocity.length()
        };
        let seen = self.cameras.iter().next();
        let ahead = flight
            .nose
            .try_normalize()
            .unwrap_or_else(|| seen.map_or(Vec3::NEG_Z, |camera| *camera.forward()));
        let sink = -velocity.dot(self.vantage.up);
        let glow = entry_glow(flight.dynamic_pressure, sink);
        let hot = Vec3::new(1.0, 0.32, 0.07).lerp(Vec3::new(1.0, 0.6, 0.3), glow) * GLOW_GAIN;
        let warp = flight.warp.clamp(0.0, 1.0);
        let wisps = self.crossing.depth * smoothstep(WISP_SPEED.0, WISP_SPEED.1, speed);
        FxUniform {
            heading: stream_heading(velocity, ahead, warp).extend(0.0),
            glow: hot.extend(glow),
            streak: Vec3::new(0.8, 0.9, 1.5).extend(smoothstep(0.0, 0.15, warp)),
            wisp: (linear(self.palette.cloud).truncate() * 0.6).extend(wisps),
            params: Vec4::new(
                self.time.elapsed_secs(),
                0.15 + 1.1 * warp,
                (speed / 40.0).clamp(0.5, 6.0),
                warp,
            ),
        }
    }
}

/// Keeps the shell around the camera and shows it only while an effect is on.
fn follow(
    inputs: FxInputs,
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
    let fx = inputs.uniform();
    let shown = fx.glow.w + fx.streak.w + fx.wisp.w > 0.0;
    let pose =
        Transform::from_translation(inputs.vantage.camera).with_scale(Vec3::splat(SHELL_RADIUS));
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

/// Embeds the shader, registers the material and runs the effects once the clouds are known.
pub(super) fn plugin(app: &mut App) {
    embedded_asset!(app, "entry.wesl");
    app.add_plugins(MaterialPlugin::<FxMaterial>::default())
        .init_resource::<FlightFx>()
        .add_systems(OnEnter(Screen::Loading), spawn)
        .add_systems(PostUpdate, follow.after(super::clouds::drift));
}
