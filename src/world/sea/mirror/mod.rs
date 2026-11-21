//! The water mirror: the robot, the companion, the ship, other players and the animals seen in
//! calm water.
//!
//! A planar reflection: a second camera, the game camera mirrored about the water plane under the
//! focus, draws only the actors into an image the size of the game view, before the game camera.
//! Its near plane is the water itself,
//! by the oblique clip plane of Bevy's `PerspectiveProjection::near_clip_plane` (Lengyel,
//! https://terathon.com/lengyel/Lengyel-Oblique.pdf), so nothing under the surface shows. The
//! image lines up with the game view pixel for pixel, as in Bevy's own `mirror` example
//! (https://github.com/bevyengine/bevy/blob/main/examples/3d/mirror.rs); the water shader reads
//! it where it shows the sky, bent by the waves, by the Fresnel share, and fading with distance.
//!
//! Screen-space reflections were not taken: Bevy's `ScreenSpaceReflections`
//! (https://docs.rs/bevy/latest/bevy/pbr/struct.ScreenSpaceReflections.html) needs the deferred
//! renderer, which the game does not use, it would not reach the see-through water, and it
//! misses whatever is off screen, as the underside of the ship always is.
//!
//! The actors sit on the game layer and on [`REFLECTED`]; the mirror camera draws [`REFLECTED`]
//! only, lit by twins of the sun and the moon without shadows, so neither the ground nor a shadow
//! map is drawn twice. Off, or with no water, no actor near or the camera looking up, the camera
//! is inactive and Bevy does not even extract it.

mod aim;
mod gate;
mod plane;
mod tune;

#[cfg(test)]
mod tests_aim;
#[cfg(test)]
mod tests_gate;
#[cfg(test)]
mod tests_plane;

use bevy::app::{HierarchyPropagatePlugin, Propagate, PropagateSet};
use bevy::asset::AssetEventSystems;
use bevy::camera::visibility::{RenderLayers, VisibilitySystems};
use bevy::camera::{CameraUpdateSystems, Exposure, Hdr, RenderTarget};
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureFormat};
use bevy::render::view::{Msaa, Tonemapping};
use bevy::transform::TransformSystems;

use crate::render::{GameCamera, LowRes, REFLECTED, reflected, seen_only, window_changed};
use crate::screens::Screen;

/// Marks the root of an actor the water mirrors. Its whole tree is put on the mirror layer too.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Reflected;

/// Marks the camera that draws the mirrored actors, with the image it draws into.
#[derive(Component, Debug, Clone)]
pub struct MirrorCamera(pub Handle<Image>);

/// Returns an image of `size` game pixels the mirror camera draws into: light beyond white kept,
/// see-through where no actor is, and read pixel for pixel.
fn target(size: UVec2) -> Image {
    let mut image = Image::new_target_texture(size.x, size.y, TextureFormat::Rgba16Float, None);
    image.sampler = ImageSampler::nearest();
    image
}

/// Spawns the mirror camera, inactive, and returns the image it draws into, for the water.
pub fn spawn(commands: &mut Commands, images: &mut Assets<Image>) -> Handle<Image> {
    let image = images.add(target(UVec2::new(320, 180)));
    let camera = Camera {
        order: -1,
        is_active: false,
        invert_culling: true,
        clear_color: ClearColorConfig::Custom(Color::NONE),
        ..default()
    };
    commands.spawn((
        (Name::new("WaterMirror"), MirrorCamera(image.clone())),
        (
            Camera3d::default(),
            camera,
            RenderTarget::from(image.clone()),
        ),
        (Hdr, Tonemapping::None, Msaa::Off, Exposure::default()),
        (RenderLayers::layer(REFLECTED), Transform::default()),
        DespawnOnEnter(Screen::Title),
    ));
    image
}

/// Resizes the mirror image to the game view, so the two line up pixel for pixel.
fn fit(low: Res<LowRes>, mirrors: Query<&MirrorCamera>, mut images: ResMut<Assets<Image>>) {
    let Some(want) = images.get(&low.0).map(Image::size) else {
        return;
    };
    let size = Extent3d {
        width: want.x,
        height: want.y,
        depth_or_array_layers: 1,
    };
    for mirror in &mirrors {
        if images
            .get(&mirror.0)
            .is_some_and(|image| image.size() != want)
            && let Some(mut image) = images.get_mut(&mirror.0)
        {
            image.resize(size);
        }
    }
}

/// Puts a new mirrored actor and everything it carries on the game and mirror layers.
fn tag(add: On<Add<Reflected>>, mut commands: Commands) {
    commands.entity(add.entity).insert(Propagate(reflected()));
}

/// Pins the game camera to the game layer, whatever actor it rides with.
fn pin(add: On<Add<GameCamera>>, mut commands: Commands) {
    commands.entity(add.entity).insert(Propagate(seen_only()));
}

/// Registers the mirror: the layers spread down every actor tree before Bevy works out what each
/// camera sees, and the mirror camera is posed after the transforms and before Bevy reads its
/// pose, its lens and the water material.
pub(super) fn plugin(app: &mut App) {
    let resized = window_changed.or_else(any_match_filter::<Added<MirrorCamera>>);
    app.add_plugins(HierarchyPropagatePlugin::<RenderLayers>::new(PostUpdate))
        .configure_sets(
            PostUpdate,
            PropagateSet::<RenderLayers>::default().before(VisibilitySystems::CheckVisibility),
        )
        .add_observer(tag)
        .add_observer(pin)
        .add_systems(
            PostUpdate,
            (
                fit.run_if(resource_exists::<LowRes>.and_then(resized)),
                aim::aim.run_if(any_with_component::<MirrorCamera>),
            )
                .chain()
                .after(TransformSystems::Propagate)
                .before(CameraUpdateSystems)
                .before(VisibilitySystems::UpdateFrusta)
                .before(AssetEventSystems),
        );
}
