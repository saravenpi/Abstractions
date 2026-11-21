//! The sea floor map entity state: the image the shader samples and the texels waiting for a sample.

use std::collections::VecDeque;

use bevy::prelude::*;

use super::bed::Bed;
use super::depth::{self, blank};

/// Texels sampled per frame, so a full refill spreads over half a second. A texel costs up to
/// four ground samples near the shore of a globe.
const BUDGET: usize = 512;

/// The depth map of the sea entity, following the focus.
#[derive(Component, Debug, Clone)]
pub struct SeaFloor {
    /// Image the water shader samples.
    image: Handle<Image>,
    /// Texel the map is centered on, once the focus is known.
    centre: Option<IVec2>,
    /// Chart texels inside the map that still hold no sample.
    pending: VecDeque<IVec2>,
    /// Texels of the image that hold water, a texel waiting for its sample included: whether
    /// there is water near the focus for the mirror to show.
    pub(super) wet: u32,
}

impl SeaFloor {
    /// Creates an empty map that reads as deep water everywhere.
    pub fn new(images: &mut Assets<Image>) -> Self {
        Self {
            image: images.add(blank()),
            centre: None,
            pending: VecDeque::new(),
            wet: (depth::SIZE * depth::SIZE) as u32,
        }
    }

    /// Returns the image for the water material.
    pub fn image(&self) -> Handle<Image> {
        self.image.clone()
    }

    /// Returns the chart x and z of the map center, for the shader to fade the map out at its rim.
    pub fn origin(&self) -> Vec4 {
        (self.centre.unwrap_or_default().as_vec2() * depth::TEXEL)
            .extend(0.0)
            .extend(0.0)
    }

    /// Forgets every sample, for a map whose chart was replaced: the next move refills it all.
    pub fn restart(&mut self) {
        self.centre = None;
        self.pending.clear();
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

    /// Samples the next queued texels into the image bytes and blanks the ones still waiting,
    /// keeping the count of the wet texels.
    fn drain(&mut self, bed: &impl Bed, data: &mut [u8]) {
        let wet = &mut self.wet;
        let mut write = |texel: IVec2, bytes: [u8; 4]| {
            let at = depth::offset(texel);
            let was = u32::from(depth::wet(&data[at..at + bytes.len()]));
            data[at..at + bytes.len()].copy_from_slice(&bytes);
            *wet = (*wet + u32::from(depth::wet(&bytes))).saturating_sub(was);
        };
        let count = BUDGET.min(self.pending.len());
        for texel in self.pending.drain(..count) {
            write(texel, bed.texel(texel));
        }
        for &texel in &self.pending {
            write(texel, depth::BLANK);
        }
    }

    /// Centers the map on `centre` and samples `bed` for a budget of the texels that scrolled
    /// in. The image is only touched while texels wait.
    pub fn follow(&mut self, centre: IVec2, bed: &impl Bed, images: &mut Assets<Image>) {
        self.retarget(centre);
        if self.pending.is_empty() {
            return;
        }
        let Some(mut image) = images.get_mut(&self.image) else {
            return;
        };
        if let Some(data) = image.data.as_mut() {
            self.drain(bed, data);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::Terrain;

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
        let counted = data
            .chunks(depth::BLANK.len())
            .filter(|t| depth::wet(t))
            .count();
        assert_eq!(floor.wet as usize, counted);
        let at = depth::offset(centre);
        assert_eq!(data[at..at + 4], depth::sample(&terrain, centre));
        floor.retarget(centre + IVec2::Y);
        assert_eq!(floor.pending.len(), depth::SIZE as usize);
        assert_eq!(floor.origin().y, (centre.y + 1) as f32 * depth::TEXEL);
    }
}
