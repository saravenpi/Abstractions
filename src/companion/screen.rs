//! The screen: a small picture repainted on the CPU only when the face frame changes.

use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use super::Companion;
use super::attention::eye_shift;
use super::brain::Brain;
use super::face::Frame;
use super::face::machine::Face;
use super::face::paint::{HEIGHT, WIDTH, paint, rgba};

/// The picture and the materials of the companion, and what they currently show.
#[derive(Component, Debug, Clone)]
pub struct Display {
    /// The picture on the screen.
    pub image: Handle<Image>,
    /// Unlit material of the screen, tinted by the face colour.
    pub face: Handle<StandardMaterial>,
    /// Unlit material of the antenna tip and the thruster.
    pub glow: Handle<StandardMaterial>,
    /// Emissive material of the emitter that stands for the light in the ray traced scene.
    pub emitter: Handle<StandardMaterial>,
    /// The frame painted last.
    pub shown: Option<Frame>,
    /// Linear face colour times its brightness, as last applied to the materials and the light.
    pub tint: Vec3,
    /// Light power in lumens as last applied to the ray traced emitter.
    pub power: f32,
    /// How hard the thruster works, from 0 at rest to 1 in flight, as last applied to the glow.
    pub thrust: f32,
}

/// Builds the picture for `frame`: nearest sampling, no mipmaps, so the pixels stay sharp.
pub fn picture(frame: &Frame) -> Image {
    let size = Extent3d {
        width: WIDTH as u32,
        height: HEIGHT as u32,
        depth_or_array_layers: 1,
    };
    let mut image = Image::new(
        size,
        TextureDimension::D2,
        rgba(&paint(frame)),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::nearest();
    image
}

/// Repaints the screen when the face frame changed. The eyes turn toward what the companion looks at.
pub fn repaint(
    bot: Single<(&GlobalTransform, &Brain, &Face, &mut Display), With<Companion>>,
    mut images: ResMut<Assets<Image>>,
) {
    let (at, brain, face, mut display) = bot.into_inner();
    let local = at.rotation().inverse() * (brain.look - at.translation());
    let gaze = match brain.mind.dozing {
        true => IVec2::ZERO,
        false => eye_shift(local),
    };
    let frame = face.frame(gaze);
    if display.shown == Some(frame) {
        return;
    }
    if let Some(mut image) = images.get_mut(&display.image) {
        image.data = Some(rgba(&paint(&frame)));
        display.shown = Some(frame);
    }
}
