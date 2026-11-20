//! The cube texture of the galaxy light: the empty texture a world starts with and the bytes of
//! a finished bake written over it.

use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{
    Extent3d, TextureDimension, TextureFormat, TextureViewDescriptor, TextureViewDimension,
};

use super::march::{BYTES, FACES, SIDE};

/// Returns the empty cube texture of the map: six faces of [`SIDE`] texels, read smoothly.
pub fn blank() -> Image {
    let side = SIDE as u32;
    let mut image = Image::new(
        Extent3d {
            width: side,
            height: side,
            depth_or_array_layers: FACES as u32,
        },
        TextureDimension::D2,
        vec![0; BYTES],
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::default(),
    );
    image.texture_view_descriptor = Some(TextureViewDescriptor {
        dimension: Some(TextureViewDimension::Cube),
        ..default()
    });
    image.sampler = ImageSampler::linear();
    image
}

/// Writes `bytes` over the texels of the texture `image`, which keeps its size.
pub fn write(images: &mut Assets<Image>, image: &Handle<Image>, bytes: &[u8]) {
    let Some(mut image) = images.get_mut(image) else {
        return;
    };
    if let Some(data) = image.data.as_mut().filter(|data| data.len() == bytes.len()) {
        data.copy_from_slice(bytes);
    }
}
