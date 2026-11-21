//! Tests of the icons of tools and materials: every kind of tool draws its own crisp picture and
//! every material has an icon, and pictures land on whole pixels at every interface size.

use bevy::math::Vec2;

use super::bench::INTENTS;
use super::glyph::{CARD, HUD, ROW, snap};
use super::kind::material_icon;
use super::raster::{BYTES, raster};
use crate::craft::build::{Figure, Intent, Look, Tint};
use crate::craft::matter::{Class, Mineral, Stuff};
use crate::craft::stage::tissue;
use crate::settings::InterfaceSize;
use crate::ui::ui_scale;

/// Returns the look of a tool of `figure`, its head of stone on a shaft of wood, at its longest.
fn drawn(figure: Figure) -> Look {
    Look {
        intent: Intent::Cut,
        figure: Some(figure),
        length: 3,
        head: Tint::Stone,
        shaft: Tint::Wood,
    }
}

/// Returns how many pixels of `picture` have the opacity `alpha`.
fn count(picture: &[u8], alpha: u8) -> usize {
    picture
        .chunks_exact(BYTES)
        .filter(|pixel| pixel[3] == alpha)
        .count()
}

/// Checks the pictures of every figure drawn `size` pixels wide: a head and a shaft each, every
/// pixel clear or in the grey of a material with no blend, and from 16 pixels up no two alike.
fn check_figures(size: u32) {
    let pictures: Vec<Vec<u8>> = Figure::ALL.map(|f| raster(&drawn(f), size)).to_vec();
    for (figure, picture) in Figure::ALL.iter().zip(&pictures) {
        assert!(
            count(picture, Tint::Stone.alpha()) > 0,
            "{figure:?} at {size}: no head"
        );
        assert!(
            count(picture, Tint::Wood.alpha()) > 0,
            "{figure:?} at {size}: no shaft"
        );
        let inked = count(picture, 0) + count(picture, 255) + count(picture, Tint::Wood.alpha());
        assert_eq!(
            inked * BYTES,
            picture.len(),
            "{figure:?} at {size}: blended pixels"
        );
    }
    for (at, one) in pictures.iter().enumerate().filter(|_| size >= 16) {
        assert!(
            pictures[at + 1..].iter().all(|other| other != one),
            "{size}: twins"
        );
    }
}

/// Every kind of tool draws its own crisp picture at every size a place shows it at.
#[test]
fn every_tool_kind_has_an_icon() {
    for size in [12, 16, 18, 24, 32, 48, 96] {
        check_figures(size);
    }
}

/// A tool with no parts draws nothing and falls back to the icon of its intent; every material has
/// an icon from the icon font.
#[test]
fn every_bare_tool_and_material_has_an_icon() {
    for intent in INTENTS {
        assert!(Look::bare(intent).figure.is_none());
        assert!(
            raster(&Look::bare(intent), 24)
                .iter()
                .all(|byte| *byte == 0)
        );
    }
    let materials = Mineral::ALL
        .map(Stuff::Mineral)
        .into_iter()
        .chain(Class::ALL.map(|class| tissue(class, 0, 1.0).stuff));
    for stuff in materials {
        assert!(!material_icon(&stuff).glyph().is_empty(), "{stuff:?}");
    }
}

/// Every scale the interface takes at the supported windows and sizes, times every screen density.
fn scales() -> Vec<(f32, f32)> {
    let windows = [
        Vec2::new(960.0, 540.0),
        Vec2::new(1280.0, 720.0),
        Vec2::new(1920.0, 1080.0),
        Vec2::new(2560.0, 1440.0),
    ];
    let interface = windows
        .into_iter()
        .flat_map(|window| InterfaceSize::ALL.map(|size| ui_scale(window, size)));
    interface
        .flat_map(|scale| [1.0, 1.25, 1.5, 2.0].map(|density| (scale, density)))
        .collect()
}

/// At every window, interface size and screen density, a picture covers a whole number of
/// physical pixels, its node exactly that many, and at a density of one it keeps its own size.
#[test]
fn pictures_are_pixel_aligned_at_every_interface_size() {
    for ((scale, density), size) in scales()
        .into_iter()
        .flat_map(|s| [HUD, ROW, CARD].map(|z| (s, z)))
    {
        let (pixels, logical) = snap(size, scale * density);
        assert!(pixels >= 1);
        assert!(
            (logical * scale * density - pixels as f32).abs() < 1e-3,
            "{size} {scale}"
        );
        assert!(density != 1.0 || logical == size, "{size} at {scale}");
    }
}
