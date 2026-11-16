//! The sea floor map entity state: the image the shader samples and the texels waiting for a sample.

use std::collections::VecDeque;

use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use super::depth;
use crate::world::Terrain;
use crate::world::focus::Focus;

/// Texels sampled per frame, so a full refill spreads over a few frames.
const BUDGET: usize = 2048;

/// The depth map of the sea entity, following the focus.
#[derive(Component, Debug, Clone)]
pub struct SeaFloor {
    /// Image the water shader samples.
    image: Handle<Image>,
    /// Texel the map is centered on, once the focus is known.
    centre: Option<IVec2>,
    /// World texels inside the map that still hold no sample.
    pending: VecDeque<IVec2>,
}

/// Builds the map image filled with deep water: wrapping, linear filtering, no color space.
fn blank() -> Image {
    let side = depth::SIZE as u32;
    let mut image = Image::new_fill(
        Extent3d {
            width: side,
            height: side,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &depth::BLANK,
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

impl SeaFloor {
    /// Creates an empty map that reads as deep water everywhere.
    pub fn new(images: &mut Assets<Image>) -> Self {
        Self {
            image: images.add(blank()),
            centre: None,
            pending: VecDeque::new(),
        }
    }

    /// Returns the image for the water material.
    pub fn image(&self) -> Handle<Image> {
        self.image.clone()
    }

    /// Returns the world x and z of the map center, for the shader to fade the map out at its rim.
    pub fn origin(&self) -> Vec4 {
        (self.centre.unwrap_or_default().as_vec2() * depth::TEXEL)
            .extend(0.0)
            .extend(0.0)
    }

    /// Moves the map center and queues the texels that scrolled in.
    fn retarget(&mut self, centre: IVec2) {
        if self.centre == Some(centre) {
            return;
        }
        self.pending.retain(|&texel| depth::inside(centre, texel));
        self.pending.extend(depth::entered(self.centre, centre));
        self.centre = Some(centre);
    }

    /// Samples the next queued texels into the image bytes and blanks the ones still waiting.
    fn drain(&mut self, terrain: &Terrain, data: &mut [u8]) {
        let mut write = |texel: IVec2, bytes: [u8; 4]| {
            let at = depth::offset(texel);
            data[at..at + bytes.len()].copy_from_slice(&bytes);
        };
        let count = BUDGET.min(self.pending.len());
        for texel in self.pending.drain(..count) {
            write(texel, depth::sample(terrain, texel));
        }
        for &texel in &self.pending {
            write(texel, depth::BLANK);
        }
    }
}

/// Keeps the map centered on the focus and samples the terrain for the texels that scrolled in.
pub fn refresh(
    focus: Focus,
    terrain: Option<Res<Terrain>>,
    mut images: ResMut<Assets<Image>>,
    mut floors: Query<&mut SeaFloor>,
) {
    let (Some(terrain), Some(p)) = (terrain, focus.position()) else {
        return;
    };
    for mut floor in &mut floors {
        floor.retarget(depth::centre_texel(p));
        if floor.pending.is_empty() {
            continue;
        }
        let Some(mut image) = images.get_mut(&floor.image) else {
            continue;
        };
        if let Some(data) = image.data.as_mut() {
            floor.drain(&terrain, data);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fresh map fills over a few frames, then a one texel step costs one row of samples.
    #[test]
    fn the_map_fills_in_budgeted_steps_and_scrolls_cheaply() {
        let terrain = Terrain::new(1);
        let mut images = Assets::<Image>::default();
        let mut floor = SeaFloor::new(&mut images);
        let mut data = blank().data.unwrap_or_default();
        let centre = depth::centre_texel(terrain.find_spawn());
        floor.retarget(centre);
        let mut frames = 0;
        while !floor.pending.is_empty() {
            floor.drain(&terrain, &mut data);
            frames += 1;
        }
        assert_eq!(frames, (depth::SIZE * depth::SIZE) as usize / BUDGET);
        let at = depth::offset(centre);
        assert_eq!(data[at..at + 4], depth::sample(&terrain, centre));
        floor.retarget(centre + IVec2::Y);
        assert_eq!(floor.pending.len(), depth::SIZE as usize);
        assert_eq!(floor.origin().y, (centre.y + 1) as f32 * depth::TEXEL);
    }
}
