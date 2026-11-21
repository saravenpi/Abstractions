//! The surface texture of a far body: the six baked faces of its ball as one texture of six
//! layers with all its mipmaps, colour in sRGB and the share of sea in alpha, so a dot reads the
//! mean of its surface and a disc every coast.

use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{
    Extent3d, TextureDimension, TextureFormat, TextureViewDescriptor, TextureViewDimension,
};

use crate::world::globe::far::Map;

/// Bytes of one texel.
const TEXEL: usize = 4;

/// Returns the mipmap levels of a square texture of `side` texels, down to one texel.
pub fn mips(side: usize) -> u32 {
    side.max(1).ilog2() + 1
}

/// Returns the bytes the texture of faces of `side` texels takes with its mipmaps.
pub fn bytes(side: usize) -> usize {
    let layer: usize = (0..mips(side)).map(|k| (side >> k).max(1).pow(2)).sum();
    6 * layer * TEXEL
}

/// Returns `texels`, a square of `side` texels, at half the side: each the mean of the four it
/// covers.
pub fn halve(texels: &[Vec4], side: usize) -> Vec<Vec4> {
    let half = (side / 2).max(1);
    let at = |i: usize, j: usize| texels[j.min(side - 1) * side + i.min(side - 1)];
    let mean = |k: usize| {
        let (i, j) = (2 * (k % half), 2 * (k / half));
        (at(i, j) + at(i + 1, j) + at(i, j + 1) + at(i + 1, j + 1)) * 0.25
    };
    (0..half * half).map(mean).collect()
}

/// Adds `texels` to `bytes`: the linear colour encoded to sRGB and the share of sea as it is.
fn encode(texels: &[Vec4], bytes: &mut Vec<u8>) {
    let byte = |value: f32| (value.clamp(0.0, 1.0) * 255.0).round() as u8;
    let colour = |value: f32| byte(Srgba::gamma_function_inverse(value));
    for t in texels {
        bytes.extend([colour(t.x), colour(t.y), colour(t.z), byte(t.w)]);
    }
}

/// Returns the bytes of the six `maps` layer after layer, each followed by its mipmaps, as the
/// texture is laid out on upload.
pub fn layers(maps: &[Map]) -> Vec<u8> {
    let side = maps.first().map_or(1, |map| map.side);
    let mut bytes = Vec::with_capacity(bytes(side));
    for map in maps {
        encode(&map.texels, &mut bytes);
        let (mut level, mut size) = (halve(&map.texels, side), side / 2);
        while size >= 1 {
            encode(&level, &mut bytes);
            if size == 1 {
                break;
            }
            level = halve(&level, size);
            size /= 2;
        }
    }
    bytes
}

/// Returns the surface texture of the six `maps`, read smoothly between texels and mipmaps. It
/// lives only on the GPU once uploaded.
pub fn image(maps: &[Map]) -> Image {
    let side = maps.first().map_or(1, |map| map.side) as u32;
    let size = Extent3d {
        width: side,
        height: side,
        depth_or_array_layers: 6,
    };
    let mut image = Image::new_uninit(
        size,
        TextureDimension::D2,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.texture_descriptor.mip_level_count = mips(side as usize);
    image.data = Some(layers(maps));
    image.texture_view_descriptor = Some(TextureViewDescriptor {
        dimension: Some(TextureViewDimension::D2Array),
        ..default()
    });
    image.sampler = ImageSampler::linear();
    image
}
