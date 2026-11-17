//! The sea floor depth map around the focus: toroidal addressing, refresh sets and texel values.
//!
//! Texel `(ix, iz)` holds the bed at chart position `(ix * TEXEL, iz * TEXEL)` of the seat of the
//! sea, a metric chart fixed to the planet, as the height the finest terrain mesh draws near the
//! shore, so the filtered map crosses zero where the visible ground meets the sea.

use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::world::{SEA_LEVEL, Terrain};

/// Side of the square map in texels.
pub const SIZE: i32 = 128;

/// Chart meters between two texels.
pub const TEXEL: f32 = 2.0;

/// Deepest water the map stores, in meters.
pub const DEEP: f32 = 16.0;

/// Highest ground above the sea the map stores, in meters, so the waterline filters to the right place.
pub const LAND: f32 = 4.0;

/// Bytes of a texel that is not sampled yet: deep open water.
pub const BLANK: [u8; 4] = [255, 255, 0, 255];

/// Builds the map image filled with deep water: wrapping, linear filtering, no color space.
pub fn blank() -> Image {
    let side = SIZE as u32;
    let mut image = Image::new_fill(
        Extent3d {
            width: side,
            height: side,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &BLANK,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        ..ImageSamplerDescriptor::linear()
    });
    image
}

/// Returns the texel the map centers on for a focus at chart position `(p.x, p.z)`.
pub fn centre_texel(p: Vec3) -> IVec2 {
    IVec2::new((p.x / TEXEL).round() as i32, (p.z / TEXEL).round() as i32)
}

/// Returns the byte offset of a chart texel in the image data, wrapping on both axes.
pub fn offset(texel: IVec2) -> usize {
    let wrapped = texel.rem_euclid(IVec2::splat(SIZE));
    (wrapped.y * SIZE + wrapped.x) as usize * BLANK.len()
}

/// Returns true when the map centered on `centre` covers `texel`.
pub fn inside(centre: IVec2, texel: IVec2) -> bool {
    let d = texel - centre + IVec2::splat(SIZE / 2);
    d.min_element() >= 0 && d.max_element() < SIZE
}

/// Returns the texels the map gains when its center moves from `old` to `new`, nearest first.
///
/// Without an old center, or after a jump past the map, that is every texel.
pub fn entered(old: Option<IVec2>, new: IVec2) -> Vec<IVec2> {
    let span = -SIZE / 2..SIZE / 2;
    let mut texels: Vec<IVec2> = span
        .clone()
        .flat_map(|z| span.clone().map(move |x| new + IVec2::new(x, z)))
        .filter(|&texel| !old.is_some_and(|old| inside(old, texel)))
        .collect();
    texels.sort_by_key(|texel| (*texel - new).length_squared());
    texels
}

/// Packs a signed depth in meters and the pond flag: 16 bit depth in red and green, pond in blue.
pub fn encode(depth: f32, pond: bool) -> [u8; 4] {
    let unit = (depth.clamp(-LAND, DEEP) + LAND) / (DEEP + LAND);
    let [high, low] = ((unit * 65535.0).round() as u16).to_be_bytes();
    [high, low, if pond { 255 } else { 0 }, 255]
}

/// Returns the bytes of a texel read through the chart of `terrain`: how far the bed lies under
/// the sea and whether a pond holds it.
pub fn sample(terrain: &Terrain, texel: IVec2) -> [u8; 4] {
    let p = texel.as_vec2() * TEXEL;
    encode(
        SEA_LEVEL - terrain.height(p.x, p.y),
        terrain.pond_at(p.x, p.y),
    )
}

#[cfg(test)]
mod tests;
