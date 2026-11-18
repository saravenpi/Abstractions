//! The backdrop of menus shown over the game: the game image, blurred and darkened.
//!
//! Bevy UI has no backdrop blur. The game is already drawn into a small image
//! ([`LowRes`]) that fills the window, so blurring it costs a few dozen samples of a small texture
//! in one material shared by every menu. Nothing is drawn while no menu is open, and no pass
//! is added over the full-resolution frame. Where the material is not available (no world on
//! screen, no renderer) the root keeps a plain dark veil of the same darkness.

use bevy::asset::embedded_asset;
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;

use super::theme;
use crate::render::{GameCamera, LowRes};

/// Blur radius as a fraction of the screen height: wide enough that no shape of the game
/// survives, so a menu sits on frosted glass and not on a darkened picture.
const RADIUS: f32 = 0.05;

/// Marks a node that shows the blurred game behind it once the material is available.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Blurred;

/// Material that draws the blurred, darkened game image over the whole node.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct BlurMaterial {
    /// Share of the light removed in x, blur radius as a fraction of the screen height in y.
    #[uniform(0)]
    pub params: Vec4,
    /// The low-resolution image of the game.
    #[texture(1)]
    #[sampler(2)]
    pub world: Handle<Image>,
}

impl UiMaterial for BlurMaterial {
    /// Points at the shader embedded in the binary.
    fn fragment_shader() -> ShaderRef {
        "embedded://abstractions/ui/backdrop.wesl".into()
    }
}

/// The one blur material every menu shares, or nothing where there is no renderer.
#[derive(Resource, Debug, Clone, Default)]
pub struct Backdrop(pub Option<Handle<BlurMaterial>>);

/// Returns the relative luminance left of a world of luminance `world` behind the backdrop.
pub fn backdrop_luminance(world: f32, tint: f32) -> f32 {
    world * (1.0 - tint)
}

/// Returns the contrast ratio between two relative luminances, from 1 to 21.
pub fn contrast(a: f32, b: f32) -> f32 {
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

/// One pixel of the backdrop in a corner. Shown behind the boot screen, it has the pipeline of
/// the material compiled before the first menu opens.
pub fn warm_up() -> impl Bundle {
    (
        Blurred,
        Node {
            position_type: PositionType::Absolute,
            right: px(0),
            bottom: px(0),
            width: px(1),
            height: px(1),
            ..default()
        },
    )
}

/// Builds the shared material from the low-resolution image of the game.
fn build(
    low: Option<Res<LowRes>>,
    materials: Option<ResMut<Assets<BlurMaterial>>>,
    mut backdrop: ResMut<Backdrop>,
) {
    if let (Some(low), Some(mut materials)) = (low, materials) {
        backdrop.0 = Some(materials.add(BlurMaterial {
            params: Vec4::new(theme::TINT, RADIUS, 0.0, 0.0),
            world: low.0.clone(),
        }));
    }
}

/// Gives a new blurred node the material while a game camera draws, and drops its plain veil.
/// Otherwise the node keeps the veil.
fn attach(
    mut commands: Commands,
    backdrop: Res<Backdrop>,
    worlds: Query<(), With<GameCamera>>,
    fresh: Query<Entity, Added<Blurred>>,
) {
    let Some(material) = backdrop.0.as_ref().filter(|_| !worlds.is_empty()) else {
        return;
    };
    for entity in &fresh {
        commands
            .entity(entity)
            .insert((MaterialNode(material.clone()), BackgroundColor(Color::NONE)));
    }
}

/// Embeds the shader and registers the material where UI is rendered, and the systems everywhere.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Backdrop>();
    if app.is_plugin_added::<bevy::ui_render::UiRenderPlugin>() {
        embedded_asset!(app, "backdrop.wesl");
        app.add_plugins(UiMaterialPlugin::<BlurMaterial>::default());
    }
    app.add_systems(Startup, build).add_systems(Update, attach);
}
