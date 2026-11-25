//! Software orthographic isometric 3D z-buffered thumbnail renderer and UI image cache.

use std::marker::PhantomData;

use bevy::asset::RenderAssetUsages;
use bevy::asset::uuid::Uuid;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::ui::ComputedNode;

use super::glyph::snap;
use super::models::ItemPreview;
use crate::craft::gun::mesh::Tri;
use crate::ui::reach;

/// Distinct top-16-bit prefix separating 3D item thumbnails from 2D tool silhouettes.
const KIN3D: u128 = 0x91E4_0000_0000_0000 << 64;

/// UI node displaying a cached isometric 3D thumbnail of `item`.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Preview3d {
    /// Item descriptor to rasterize.
    pub item: ItemPreview,
    /// Target square size in logical UI pixels.
    pub size: f32,
    /// Adds a 1–2 px dark silhouette outline/shadow when drawn over live 3D views.
    pub shadow: bool,
}

/// Isometric view projection + z-buffer rasterization into RGBA8 bytes.
fn raster_tris(tris: &[Tri], pixels: u32) -> Vec<u8> {
    let w = pixels.max(8) as usize;
    let mut out = vec![0u8; w * w * 4];
    if tris.is_empty() {
        return out;
    }
    let rot = Quat::from_euler(EulerRot::YXZ, -0.68, 0.42, 0.0);
    let light = Vec3::new(0.45, 0.72, 0.52).normalize();
    let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
    for tri in tris {
        for v in tri.v {
            let p = rot * v;
            lo = lo.min(p);
            hi = hi.max(p);
        }
    }
    let span = (hi.x - lo.x).max(hi.y - lo.y).max(0.01);
    let mid = (lo + hi) * 0.5;
    let scale = (w as f32 * 0.78) / span;
    let half = w as f32 * 0.5;
    let mut zbuf = vec![f32::NEG_INFINITY; w * w];
    for tri in tris {
        let p: [Vec3; 3] = tri.v.map(|v| rot * v);
        let n = (p[1] - p[0]).cross(p[2] - p[0]).normalize_or(Vec3::Z);
        let diff = n.dot(light).abs();
        let gain = 0.34 + 0.66 * diff;
        let rgb = tri.color.map(|c| {
            let lit = (c * gain).clamp(0.0, 1.0).powf(1.0 / 2.2);
            (lit * 255.0).round() as u8
        });
        let s: [Vec3; 3] = p.map(|q| {
            Vec3::new(
                (q.x - mid.x) * scale + half,
                half - (q.y - mid.y) * scale,
                q.z,
            )
        });
        let denom = (s[1].y - s[2].y) * (s[0].x - s[2].x) + (s[2].x - s[1].x) * (s[0].y - s[2].y);
        if denom.abs() < 1e-4 {
            continue;
        }
        let x0 = s
            .iter()
            .map(|q| q.x)
            .fold(f32::MAX, f32::min)
            .floor()
            .max(0.0) as usize;
        let x1 = (s.iter().map(|q| q.x).fold(f32::MIN, f32::max).ceil() as usize).min(w - 1);
        let y0 = s
            .iter()
            .map(|q| q.y)
            .fold(f32::MAX, f32::min)
            .floor()
            .max(0.0) as usize;
        let y1 = (s.iter().map(|q| q.y).fold(f32::MIN, f32::max).ceil() as usize).min(w - 1);
        for y in y0..=y1 {
            for x in x0..=x1 {
                let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
                let w0 =
                    ((s[1].y - s[2].y) * (px - s[2].x) + (s[2].x - s[1].x) * (py - s[2].y)) / denom;
                let w1 =
                    ((s[2].y - s[0].y) * (px - s[2].x) + (s[0].x - s[2].x) * (py - s[2].y)) / denom;
                let w2 = 1.0 - w0 - w1;
                if w0 < -0.001 || w1 < -0.001 || w2 < -0.001 {
                    continue;
                }
                let z = w0 * s[0].z + w1 * s[1].z + w2 * s[2].z;
                let idx = y * w + x;
                if z > zbuf[idx] {
                    zbuf[idx] = z;
                    let off = idx * 4;
                    out[off..off + 4].copy_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
                }
            }
        }
    }
    out
}

/// Adds a dark crisp 1-px silhouette border + optional offset shadow onto transparent pixels.
fn add_outline(buf: &mut [u8], size: usize, reach_px: usize) {
    let orig = buf.to_vec();
    let opaque = |x: isize, y: isize| -> bool {
        if x < 0 || y < 0 || x >= size as isize || y >= size as isize {
            return false;
        }
        orig[(y as usize * size + x as usize) * 4 + 3] == 255
    };
    for y in 0..size {
        for x in 0..size {
            let idx = (y * size + x) * 4;
            if orig[idx + 3] != 0 {
                continue;
            }
            let (ix, iy) = (x as isize, y as isize);
            let near = opaque(ix - 1, iy)
                || opaque(ix + 1, iy)
                || opaque(ix, iy - 1)
                || opaque(ix, iy + 1);
            let drop = reach_px > 0 && opaque(ix - reach_px as isize, iy - reach_px as isize);
            if near || drop {
                buf[idx..idx + 4].copy_from_slice(&[10, 12, 16, if near { 180 } else { 128 }]);
            }
        }
    }
}

/// Renders an isometric sRGB RGBA8 byte buffer (`pixels x pixels`) for `item`.
pub fn render_preview(item: &ItemPreview, pixels: u32, shadow_reach: u32) -> Vec<u8> {
    let tris = item.tris();
    let mut data = raster_tris(&tris, pixels);
    add_outline(&mut data, pixels.max(8) as usize, shadow_reach as usize);
    data
}

/// Returns the deterministic cached Bevy image handle for `item` at `(pixels, shadow_reach)`.
pub fn handle_3d(item: &ItemPreview, pixels: u32, shadow_reach: u32) -> Handle<Image> {
    let key = KIN3D
        | (u128::from(item.code() & 0x0000_FFFF_FFFF_FFFF) << 40)
        | (u128::from(shadow_reach & 0xFF) << 32)
        | u128::from(pixels);
    Handle::Uuid(Uuid::from_u128(key), PhantomData)
}

/// Builds a nearest-sampled Bevy `Image` for `item`.
fn preview_image(item: &ItemPreview, pixels: u32, shadow_reach: u32) -> Image {
    let size = Extent3d {
        width: pixels.max(8),
        height: pixels.max(8),
        depth_or_array_layers: 1,
    };
    let mut image = Image::new(
        size,
        TextureDimension::D2,
        render_preview(item, pixels, shadow_reach),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::nearest();
    image
}

/// Spawns a UI image node displaying the 3D isometric thumbnail of `item`.
pub fn preview_badge(item: ItemPreview, size: f32, shadow: bool) -> impl Bundle {
    let room = Node {
        width: px(size),
        height: px(size),
        flex_shrink: 0.0,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    };
    let unseen = ImageNode {
        color: Color::NONE,
        ..default()
    };
    (
        room,
        children![(
            Preview3d { item, size, shadow },
            unseen,
            Node {
                width: px(size),
                height: px(size),
                flex_shrink: 0.0,
                ..default()
            }
        )],
    )
}

/// Query for [`Preview3d`] nodes needing thumbnail rasterization.
type PreviewQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Preview3d,
        &'static ComputedNode,
        &'static mut ImageNode,
        &'static mut Node,
    ),
    Or<(Added<Preview3d>, Changed<ComputedNode>)>,
>;

/// Rasterizes and assigns cached 3D thumbnails whenever a [`Preview3d`] node appears or rescales.
pub(super) fn paint_previews(mut nodes: PreviewQuery, mut images: ResMut<Assets<Image>>) {
    for (preview, computed, mut shown, mut node) in &mut nodes {
        let scale = 1.0 / computed.inverse_scale_factor.max(f32::EPSILON);
        let (pixels, logical) = snap(preview.size, scale);
        let reach_px = [0, reach(scale) as u32][usize::from(preview.shadow)];
        let wanted = handle_3d(&preview.item, pixels, reach_px);
        if !images.contains(wanted.id()) {
            let _ = images.insert(wanted.id(), preview_image(&preview.item, pixels, reach_px));
        }
        if shown.image != wanted {
            (shown.image, shown.color) = (wanted, Color::WHITE);
        }
        if node.width != px(logical) {
            (node.width, node.height) = (px(logical), px(logical));
        }
    }
}
