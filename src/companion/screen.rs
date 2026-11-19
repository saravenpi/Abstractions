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
use super::puppet::Parts;

/// The picture and the materials of the companion, and what they currently show.
#[derive(Component, Debug, Clone)]
pub struct Display {
    /// The picture on the screen.
    pub image: Handle<Image>,
    /// Unlit material of the screen, tinted by the face colour.
    pub face: Handle<StandardMaterial>,
    /// Unlit material of the antenna tip and the thruster.
    pub glow: Handle<StandardMaterial>,
    /// The frame painted last.
    pub shown: Option<Frame>,
    /// Linear face colour times its brightness, as last applied to the materials and the light.
    pub tint: Vec3,
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

/// Returns the eye shift of a companion whose head is at `head` and looks at `look`: none while
/// it dozes or has no brain to look with.
fn gaze(brain: Option<&Brain>, head: &GlobalTransform) -> IVec2 {
    match brain {
        Some(brain) if !brain.mind.dozing => {
            eye_shift(head.rotation().inverse() * (brain.look - head.translation()))
        }
        _ => IVec2::ZERO,
    }
}

/// Repaints the screen of every companion with a face when its frame changed. The eyes turn
/// toward what the companion looks at, from where its own head points.
pub fn repaint(
    mut bots: Query<(Option<&Brain>, &Face, &Parts, &mut Display), With<Companion>>,
    heads: Query<&GlobalTransform>,
    mut images: ResMut<Assets<Image>>,
) {
    for (brain, face, parts, mut display) in &mut bots {
        let Ok(head) = heads.get(parts.head) else {
            continue;
        };
        let frame = face.frame(gaze(brain, head));
        if display.shown == Some(frame) {
            continue;
        }
        if let Some(mut image) = images.get_mut(&display.image) {
            image.data = Some(rgba(&paint(&frame)));
            display.shown = Some(frame);
        }
    }
}
