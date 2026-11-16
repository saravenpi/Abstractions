//! The low-resolution image the game camera renders into, sized from the window and the pixel size.

use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use bevy::window::PrimaryWindow;

use crate::settings::{Persistent, Settings};

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

/// Returns the target size in pixels for a window logical size and a block size, never tiny.
///
/// The logical size is used so a high-density display shades no more pixels than a normal one.
pub fn target_size(physical: UVec2, pixel_size: u32) -> UVec2 {
    let block = pixel_size.max(1);
    ((physical.max(UVec2::ONE) + UVec2::splat(block - 1)) / UVec2::splat(block))
        .max(UVec2::splat(MIN_SIDE))
}

/// Resizes the target image when the window or the camera pixel size asks for another size.
fn resize_target(
    low: Res<LowRes>,
    window: Single<&Window, With<PrimaryWindow>>,
    settings: Res<Persistent<Settings>>,
    mut images: ResMut<Assets<Image>>,
) {
    let logical = Vec2::new(window.width(), window.height()).ceil().as_uvec2();
    let want = target_size(logical, settings.graphics.pixel_size);
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
        .add_systems(Update, resize_target);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sizes round up, never reach zero and shrink with the block size.
    #[test]
    fn target_size_rounds_up() {
        assert_eq!(target_size(UVec2::new(2880, 1800), 3), UVec2::new(960, 600));
        assert_eq!(target_size(UVec2::new(1000, 700), 3), UVec2::new(334, 234));
        assert_eq!(target_size(UVec2::ZERO, 4), UVec2::splat(MIN_SIDE));
        assert_eq!(target_size(UVec2::new(100, 90), 0), UVec2::new(100, 90));
    }
}
