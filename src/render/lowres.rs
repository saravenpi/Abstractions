//! The low-resolution image the game camera renders into. Every screen shows about
//! [`REFERENCE_ROWS`] rows of game pixels, and every game pixel is a square of the same whole
//! number of window pixels.
//!
//! The image is shown at a whole scale with nearest sampling, as a pixel-perfect camera does from
//! its reference resolution (https://docs.unity3d.com/Manual/urp/2d-pixelperfect-ref.html): a
//! stretch by a fraction makes some rows and columns wider than others
//! (https://propixelizer.github.io/docs/tips_and_tricks/artefacts). When the window is not a
//! multiple of the scale, the image holds one more game pixel and the window crops the remainder
//! evenly at its edges, rather than the bilinear remainder of
//! https://github.com/rsn8887/Sharp-Bilinear-Shaders, so nothing is ever blurred or barred.

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
                width: 320,
                height: 180,
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
    /// Window pixels per game pixel, along each side.
    pub scale: u32,
    /// Size of the game image, in game pixels.
    pub size: UVec2,
    /// Window pixels of the scaled image cut away at the left and the top. The right and the
    /// bottom lose the rest of the overflow, at most one more.
    pub crop: UVec2,
}

impl Fit {
    /// Returns the fit for a window of `physical` pixels: the whole scale nearest to the height
    /// over [`REFERENCE_ROWS`], and an image that covers the window at that scale.
    pub fn of(physical: UVec2) -> Self {
        let physical = physical.max(UVec2::ONE);
        let scale = ((physical.y + REFERENCE_ROWS / 2) / REFERENCE_ROWS).max(1);
        let step = UVec2::splat(scale);
        let size = ((physical + step - UVec2::ONE) / step).max(UVec2::splat(MIN_SIDE));
        let crop = (size * scale).saturating_sub(physical) / 2;
        Self { scale, size, crop }
    }

    /// Returns the part of the image a window of `physical` pixels shows, in game pixels: exactly
    /// `scale` window pixels to a game pixel.
    pub fn shown(&self, physical: UVec2) -> Rect {
        let scale = self.scale as f32;
        let min = self.crop.as_vec2() / scale;
        Rect::from_corners(min, min + physical.as_vec2() / scale)
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
