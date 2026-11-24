//! The low-resolution image the game camera renders into. Every screen caps vertical game pixels
//! at [`REFERENCE_ROWS`] while matching the window aspect ratio without cropping edges, so
//! fullscreen and large windows never look less detailed than smaller windows.

use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use bevy::window::PrimaryWindow;

use super::tune::REFERENCE_ROWS;

/// The render target image of the game camera, shown full window by the UI camera with nearest filtering.
#[derive(Resource, Debug, Clone)]
pub struct LowRes(pub Handle<Image>);

impl FromWorld for LowRes {
    /// Creates a small target image with a nearest sampler; the resize system fits it to the window.
    fn from_world(world: &mut World) -> Self {
        let mut image = Image::new_fill(
            Extent3d {
                width: 960,
                height: 540,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            &[0, 0, 0, 255],
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        );
        image.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING
            | TextureUsages::COPY_DST
            | TextureUsages::RENDER_ATTACHMENT;
        image.sampler = ImageSampler::nearest();
        Self(world.resource_mut::<Assets<Image>>().add(image))
    }
}

/// Smallest target side: screen-space effects build mip chains that need a few levels.
const MIN_SIDE: u32 = 32;

/// How the game image fits a window of physical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fit {
    /// Size of the game image, in game pixels.
    pub size: UVec2,
}

impl Fit {
    /// Returns the fit for a window of `physical` pixels: height clamped between [`MIN_SIDE`] and
    /// [`REFERENCE_ROWS`], width matching the window aspect ratio and at least [`MIN_SIDE`].
    pub fn of(physical: UVec2) -> Self {
        let physical = physical.max(UVec2::ONE);
        let height = physical.y.clamp(MIN_SIDE, REFERENCE_ROWS);
        let width = ((u64::from(physical.x) * u64::from(height) + u64::from(physical.y) / 2)
            / u64::from(physical.y))
        .max(u64::from(MIN_SIDE)) as u32;
        Self {
            size: UVec2::new(width, height),
        }
    }
}

/// Whether the primary window changed this frame, such as its size or its scale factor on
/// another monitor.
pub fn window_changed(windows: Query<(), (With<PrimaryWindow>, Changed<Window>)>) -> bool {
    !windows.is_empty()
}

/// Resizes the target image when the window asks for another size.
fn resize_target(
    low: Res<LowRes>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut images: ResMut<Assets<Image>>,
) {
    let want = Fit::of(window.physical_size()).size;
    if images.get(&low.0).is_none_or(|image| image.size() == want) {
        return;
    }
    if let Some(mut image) = images.get_mut(&low.0) {
        image.resize(Extent3d {
            width: want.x,
            height: want.y,
            depth_or_array_layers: 1,
        });
    }
}

/// Creates the target image and keeps it sized.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<LowRes>()
        .add_systems(Update, resize_target.run_if(window_changed));
}
