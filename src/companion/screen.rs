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

/// Where the eyes of a companion with no brain of its own look, each axis -1, 0 or 1, as its
/// player's game says they do.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq)]
pub struct Look(pub IVec2);

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

/// What a screen is painted from: the brain or the look that turns the eyes, the face, the
/// parts that hold the head and what the screen shows.
type Painted = (
    Option<&'static Brain>,
    Option<&'static Look>,
    &'static Face,
    &'static Parts,
    &'static mut Display,
);

/// Repaints the screen of every companion with a face when its frame changed. The eyes turn
/// toward what the companion looks at, from where its own head points, or where its [`Look`]
/// says.
pub fn repaint(
    mut bots: Query<Painted, With<Companion>>,
    heads: Query<&GlobalTransform>,
    mut images: ResMut<Assets<Image>>,
) {
    for (brain, look, face, parts, mut display) in &mut bots {
        let Ok(head) = heads.get(parts.head) else {
            continue;
        };
        let frame = face.frame(look.map_or_else(|| gaze(brain, head), |look| look.0));
        if display.shown == Some(frame) {
            continue;
        }
        if let Some(mut image) = images.get_mut(&display.image) {
            image.data = Some(rgba(&paint(&frame)));
            display.shown = Some(frame);
        }
    }
}
