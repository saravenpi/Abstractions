//! The season of the planet under the camera, carried to the ground shader in a texture of its own.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::render::{SeasonLight, SeasonUniform};
use crate::world::props::PropAssets;

/// Builds the texture that carries the season to the ground shader, all zero: no season to show.
pub fn season_map() -> Image {
    let size = Extent3d {
        width: SeasonUniform::FIELDS as u32,
        height: 1,
        depth_or_array_layers: 1,
    };
    Image::new(
        size,
        TextureDimension::D2,
        vec![0; SeasonUniform::BYTES],
        TextureFormat::Rgba32Float,
        RenderAssetUsages::default(),
    )
}

/// Writes the season of the planet under the camera into the texture of the ground material, in place,
/// whenever it changed.
pub fn follow(
    season: Res<SeasonLight>,
    assets: Option<Res<PropAssets>>,
    mut images: ResMut<Assets<Image>>,
) {
    let Some(assets) = assets else {
        return;
    };
    let bytes = season.0.texels();
    let same = |image: &Image| image.data.as_deref() == Some(&bytes[..]);
    if images.get(&assets.season).is_none_or(same) {
        return;
    }
    let Some(mut image) = images.get_mut(&assets.season) else {
        return;
    };
    if let Some(data) = image.data.as_mut() {
        data.copy_from_slice(&bytes);
    }
}
