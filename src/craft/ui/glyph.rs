//! The icon of a tool, one picture wherever the tool shows: the workshop cards, the pack rows, the
//! hotbar of the pack screen and the hotbar of the game. Each place shows it at its own size.
//!
//! A picture is drawn once for each look and each size in physical pixels, the first time a node
//! shows it, and kept as an image under a handle made from the two, so every later node of the
//! same tool at the same size reuses it. Its node is set to the exact logical size of those pixels,
//! so one pixel of the picture is one pixel of the window at every interface size, sampled with no
//! blur. A tool with no parts to draw shows the icon of its intent from the icon font, and a
//! material the icon of its kind.

use std::marker::PhantomData;

use bevy::asset::RenderAssetUsages;
use bevy::asset::uuid::Uuid;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::ui::ComputedNode;

use super::intent;
use super::raster::{raster, shade};
use crate::craft::build::Look;
use crate::ui::{Icon, Tone, icon_sized, meter, reach};

/// Logical size of the icon of a tool in a slot of the hotbar of the game, the icon slot size.
pub const HUD: f32 = 24.0;

/// Logical size of the icon of a tool or a material in a row of the pack, and in a slot of the
/// hotbar of the pack screen: the icon slot of every row.
pub const ROW: f32 = 24.0;

/// Logical size of the icon of a tool on a workshop card.
pub const CARD: f32 = 48.0;

/// The high bits of the handle of every tool picture, so none meets a handle of another kind.
const KIN: u128 = 0x7a3c_51e0_9b2d_4f86 << 64;

/// A node that shows the picture of `look`, `size` logical pixels wide and high.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Picture {
    /// What the picture is drawn from.
    pub look: Look,
    /// Its logical size.
    pub size: f32,
    /// Whether it carries the HUD shadow, as the texts and icons over the live world do.
    pub shadow: bool,
}

/// Returns the pixels a picture `size` logical pixels wide takes at `scale` physical pixels per
/// logical one, whole and at least one, and the logical size that covers exactly that many.
pub fn snap(size: f32, scale: f32) -> (u32, f32) {
    let pixels = (size * scale).round().max(1.0);
    (pixels as u32, pixels / scale.max(f32::EPSILON))
}

/// Returns the handle the picture of `look` drawn `pixels` wide, with a shadow `reach` pixels
/// away or none at 0, is kept under.
pub fn handle(look: &Look, pixels: u32, reach: u32) -> Handle<Image> {
    let key = KIN | u128::from(look.code()) << 32 | u128::from(reach) << 24 | u128::from(pixels);
    Handle::Uuid(Uuid::from_u128(key), PhantomData)
}

/// Returns the image of `look` drawn `pixels` wide with its shadow `reach` pixels away, sampled
/// to the nearest pixel.
fn image(look: &Look, pixels: u32, reach: u32) -> Image {
    let size = Extent3d {
        width: pixels,
        height: pixels,
        depth_or_array_layers: 1,
    };
    let mut data = raster(look, pixels);
    shade(&mut data, pixels as usize, reach as usize);
    let format = TextureFormat::Rgba8UnormSrgb;
    let mut image = Image::new(
        size,
        TextureDimension::D2,
        data,
        format,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::nearest();
    image
}

/// Returns the icon of a tool of `look`, or of a material named by `glyph`, in a room `size`
/// logical pixels square: the drawn picture of a tool with parts, the icon of the intent of a tool
/// without, the glyph of a material, or nothing. Over the live world the picture is drawn with
/// the HUD `shadow`; a glyph there gets it as every HUD text does.
pub fn badge(look: Option<Look>, glyph: Option<Icon>, size: f32, shadow: bool) -> impl Bundle {
    let room = Node {
        width: px(size),
        height: px(size),
        flex_shrink: 0.0,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    };
    let spawn = SpawnWith(move |parent: &mut ChildSpawner| {
        if let Some(look) = look.filter(|look| look.figure.is_some()) {
            let unseen = ImageNode {
                color: Color::NONE,
                ..default()
            };
            let picture = Picture { look, size, shadow };
            parent.spawn((picture, unseen, room_of(size)));
        } else if let Some(symbol) = look.map(|look| intent::icon(look.intent)).or(glyph) {
            parent.spawn(icon_sized(symbol, Tone::Plain, size));
        }
    });
    (room, Children::spawn(spawn))
}

/// Returns a node `size` logical pixels square.
fn room_of(size: f32) -> Node {
    Node {
        width: px(size),
        height: px(size),
        flex_shrink: 0.0,
        ..default()
    }
}

/// Returns the icon of a tool of `shape` as a slot of the hotbar of the game shows it, over the
/// live world, with the HUD shadow.
pub fn silhouette(shape: Option<Look>) -> impl Bundle {
    badge(shape, None, HUD, true)
}

/// Returns a wear bar `width` pixels wide, filled with what is left of a tool `worn` 0 to 1.
pub fn wear_bar(width: f32, worn: f32) -> impl Bundle {
    meter(width, 1.0 - worn)
}

/// The pictures to draw or to size again: new ones, and those whose layout changed.
type Fresh = Or<(Added<Picture>, Changed<ComputedNode>)>;

/// Gives every picture the image of its look at the physical size it shows at, drawing the image
/// the first time that look shows at that size, and sizes its node to cover those pixels exactly.
/// A new picture stays clear until it has its image, so it never flashes a blank square.
pub(super) fn paint(
    mut pictures: Query<(&Picture, &ComputedNode, &mut ImageNode, &mut Node), Fresh>,
    mut images: ResMut<Assets<Image>>,
) {
    for (picture, computed, mut shown, mut node) in &mut pictures {
        let scale = 1.0 / computed.inverse_scale_factor.max(f32::EPSILON);
        let (pixels, logical) = snap(picture.size, scale);
        let reach = [0, reach(scale) as u32][usize::from(picture.shadow)];
        let wanted = handle(&picture.look, pixels, reach);
        if !images.contains(wanted.id())
            && let Err(error) = images.insert(wanted.id(), image(&picture.look, pixels, reach))
        {
            warn!("could not keep the picture of a tool: {error}");
        }
        if shown.image != wanted {
            (shown.image, shown.color) = (wanted, Color::WHITE);
        }
        if node.width != px(logical) {
            (node.width, node.height) = (px(logical), px(logical));
        }
    }
}
