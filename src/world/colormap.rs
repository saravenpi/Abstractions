//! The color lookup texture: each flat color is one texel, addressed by a constant UV per triangle.

use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

/// Levels kept per sRGB channel.
const LEVELS: u32 = 32;

/// Width of the texture: one tile of `LEVELS` texels per blue level.
const WIDTH: u32 = LEVELS * LEVELS;

/// Returns the level of an sRGB channel value.
fn level(channel: f32) -> u32 {
    (channel.clamp(0.0, 1.0) * (LEVELS - 1) as f32).round() as u32
}

/// Returns the 8-bit sRGB value stored for a channel level.
fn stored(level: u32) -> u8 {
    (level as f32 * 255.0 / (LEVELS - 1) as f32).round() as u8
}

/// Returns the UV of the texel center that holds a linear RGBA color.
pub fn uv(linear: [f32; 4]) -> [f32; 2] {
    let srgb = Srgba::from(LinearRgba::new(linear[0], linear[1], linear[2], linear[3]));
    let (r, g, b) = (level(srgb.red), level(srgb.green), level(srgb.blue));
    [
        ((b * LEVELS + r) as f32 + 0.5) / WIDTH as f32,
        (g as f32 + 0.5) / LEVELS as f32,
    ]
}

/// Builds the lookup texture with nearest sampling, no mipmaps and clamped edges.
pub fn image() -> Image {
    let mut data = Vec::with_capacity((WIDTH * LEVELS * 4) as usize);
    for g in 0..LEVELS {
        for b in 0..LEVELS {
            for r in 0..LEVELS {
                data.extend([stored(r), stored(g), stored(b), 255]);
            }
        }
    }
    let size = Extent3d {
        width: WIDTH,
        height: LEVELS,
        depth_or_array_layers: 1,
    };
    let mut image = Image::new(
        size,
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::nearest();
    image
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::shapes::srgb;

    /// Looking a color up through its UV returns it within one quantisation step.
    #[test]
    fn lookup_stays_within_one_step() {
        let image = image();
        let data = image.data.as_ref().unwrap();
        let colors = [
            [0.0, 0.0, 0.0],
            [1.0, 1.0, 1.0],
            [0.30, 0.58, 0.22],
            [0.9, 0.85, 0.75],
            [0.04, 0.27, 0.35],
            [0.97, 0.55, 0.7],
        ];
        for [r, g, b] in colors {
            let [u, v] = uv(srgb(r, g, b));
            let (x, y) = ((u * WIDTH as f32) as usize, (v * LEVELS as f32) as usize);
            let at = (y * WIDTH as usize + x) * 4;
            for (i, want) in [r, g, b].into_iter().enumerate() {
                let got = data[at + i] as f32 / 255.0;
                assert!(
                    (got - want).abs() <= 1.0 / (LEVELS - 1) as f32,
                    "{want} became {got}"
                );
            }
        }
    }

    /// UVs sit on texel centers, so nearest sampling never reaches a neighbour.
    #[test]
    fn uvs_hit_texel_centers() {
        let [u, v] = uv(srgb(0.3, 0.6, 0.2));
        assert_eq!((u * WIDTH as f32).fract(), 0.5);
        assert_eq!((v * LEVELS as f32).fract(), 0.5);
    }
}
