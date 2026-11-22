//! Surface texture tests: the six faces of a ball become six layers with every mipmap, a body of
//! a few pixels reads the mean colour of its surface from the last of them and no relief, and the
//! texture memory of each level is what the report states.

use std::sync::Arc;

use bevy::prelude::*;
use bevy::render::render_resource::{TextureFormat, TextureViewDimension};

use super::surface::{Kind, bytes, halve, image, layers, mips};
use crate::universe::Universe;
use crate::world::Globe;
use crate::world::globe::Face;
use crate::world::globe::far::{Map, SIDES, ball, face};

/// Returns the linear colour and sea share the bytes of one texel stand for.
fn decode(texel: &[u8]) -> Vec4 {
    let colour = |byte: u8| Srgba::gamma_function(f32::from(byte) / 255.0);
    Vec4::new(
        colour(texel[0]),
        colour(texel[1]),
        colour(texel[2]),
        f32::from(texel[3]) / 255.0,
    )
}

/// Returns the six faces of the home planet of seed 7 at the coarsest level.
fn home() -> (Arc<Globe>, Vec<Map>) {
    let universe = Universe::new(7);
    let globe = Arc::new(Globe::of(&universe.planet(universe.home())));
    let maps = Face::ALL
        .map(|at| face(&globe, at, SIDES[0]))
        .into_iter()
        .collect();
    (globe, maps)
}

/// A texture of faces of 64 texels holds seven levels per layer, down to one texel, and takes a
/// third more than its first level; the finest level takes about 32 MiB.
#[test]
fn the_texture_holds_every_mipmap() {
    assert_eq!((mips(64), mips(1024), mips(1)), (7, 11, 1));
    assert_eq!(bytes(64), 6 * 4 * (4096 + 1024 + 256 + 64 + 16 + 4 + 1));
    assert!(bytes(SIDES[2]) < 33 << 20 && bytes(SIDES[2]) > 31 << 20);
    let flat = vec![Vec4::splat(0.5); 16];
    assert_eq!(halve(&flat, 4), vec![Vec4::splat(0.5); 4]);
    let ramp: Vec<Vec4> = (0..4).map(|k| Vec4::splat(k as f32)).collect();
    assert_eq!(halve(&ramp, 2), vec![Vec4::splat(1.5)]);
}

/// The six faces of a ball become a texture of six layers read as an array, with all its
/// mipmaps, and as many bytes as the texture holds.
#[test]
fn a_ball_becomes_a_layered_texture() {
    let (_, maps) = home();
    let texture = image(&maps, Kind::Colour);
    let size = texture.texture_descriptor.size;
    assert_eq!(
        (size.width, size.height, size.depth_or_array_layers),
        (64, 64, 6)
    );
    assert_eq!(texture.texture_descriptor.mip_level_count, 7);
    let view = texture
        .texture_view_descriptor
        .as_ref()
        .and_then(|view| view.dimension);
    assert_eq!(view, Some(TextureViewDimension::D2Array));
    assert_eq!(texture.data.as_ref().map(Vec::len), Some(bytes(64)));
}

/// A body of a few pixels reads the last mipmap of its six layers, which keeps the mean colour
/// and the sea of its whole surface within a few hundredths.
#[test]
fn a_dot_keeps_the_mean_colour() {
    let (globe, maps) = home();
    let dot = ball(&globe, &maps);
    let data = layers(&maps, Kind::Colour);
    let layer = data.len() / 6;
    let last = (0..6).map(|k| decode(&data[(k + 1) * layer - 4..(k + 1) * layer]));
    let seen = last.sum::<Vec4>() / 6.0;
    let off = (seen.truncate() - dot.mean).abs().max_element();
    assert!(off < 0.03, "{seen} against {}", dot.mean);
    assert!(
        (seen.w - dot.sea).abs() < 0.05,
        "{} against {}",
        seen.w,
        dot.sea
    );
    assert!(dot.mean.max_element() > 0.01, "{}", dot.mean);
}

/// The relief of a ball is a texture of six layers of plain numbers, as large as the colour, and
/// a dot reads almost no lean from its last mipmap: the slopes of a whole face cancel out.
#[test]
fn a_dot_shows_no_relief() {
    let (_, maps) = home();
    let relief = image(&maps, Kind::Relief);
    assert_eq!(relief.texture_descriptor.format, TextureFormat::Rgba8Unorm);
    assert_eq!(relief.data.as_ref().map(Vec::len), Some(bytes(64)));
    let data = layers(&maps, Kind::Relief);
    let layer = data.len() / 6;
    for k in 0..6 {
        let last = &data[(k + 1) * layer - 4..(k + 1) * layer];
        let lean = last[..3]
            .iter()
            .map(|b| (f32::from(*b) / 255.0 * 2.0 - 1.0).abs());
        assert!(lean.fold(0.0, f32::max) < 0.05, "face {k}: {last:?}");
        assert_eq!(last[3], 255);
    }
}
