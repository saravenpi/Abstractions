//! Where the sea stands among see-through things.
//!
//! The renderer sorts see-through meshes by one point each, and the sea sorts by the middle of
//! its seat, which can lie kilometres from the camera, even behind it. So the sea is drawn first
//! of all of them and writes its depth. A trail, a puff or a spark drawn after it is then tested
//! against the surface pixel by pixel: shown above the water, hidden below it, however long it is
//! and wherever the middle of its mesh lies.

use bevy::render::render_resource::RenderPipelineDescriptor;

/// Sort bias of the sea among see-through things: lower than any distance in view, so the sea
/// draws before every one of them.
pub const FIRST: f32 = f32::MIN;

/// Makes `descriptor` draw both faces, so the surface shows from below, and write its depth, so
/// see-through things drawn after the sea are hidden where they lie under the water.
pub fn surface(descriptor: &mut RenderPipelineDescriptor) {
    descriptor.primitive.cull_mode = None;
    if let Some(depth) = descriptor.depth_stencil.as_mut() {
        depth.depth_write_enabled = Some(true);
    }
}
