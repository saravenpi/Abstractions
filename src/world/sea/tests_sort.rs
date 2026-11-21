//! Tests of how the sea is ordered among see-through things: it draws first and writes its
//! depth, so a trail is shown above the water and hidden below it at any length and angle.

use bevy::core_pipeline::core_3d::TransparentSortingInfo3d;
use bevy::math::Affine3A;
use bevy::pbr::Material;
use bevy::prelude::*;
use bevy::render::render_phase::ViewRangefinder3d;
use bevy::render::render_resource::{
    CompareFunction, DepthStencilState, RenderPipelineDescriptor, TextureFormat,
};

use super::material::WaterMaterial;
use super::order::surface;

/// Returns the sort distance of a mesh centred at `centre` with `bias`, seen from `eye` looking
/// at `target`.
fn sorted(eye: Vec3, target: Vec3, centre: Vec3, bias: f32) -> f32 {
    let view = Transform::from_translation(eye).looking_at(target, Vec3::Y);
    let finder = ViewRangefinder3d::from_world_from_view(&Affine3A::from_mat4(view.to_matrix()));
    TransparentSortingInfo3d::Sorted {
        mesh_center: centre,
        depth_bias: bias,
    }
    .sort_distance(&finder)
}

/// Seen from above, level with the water or from below, the sea sorts before any other mesh
/// without a bias, wherever their middles lie: near, far, ahead, behind or far down a trail.
/// The sea sorts by the origin of its seat, which can lie kilometres away, behind the camera.
#[test]
fn the_sea_draws_before_every_trail() {
    let sea = WaterMaterial {
        water: default(),
        floor: Handle::default(),
        place: default(),
        mirror: Handle::default(),
    };
    let bias = sea.depth_bias();
    let eyes = [
        (Vec3::new(0.0, 3.0, 0.0), Vec3::new(0.0, 0.0, -40.0)),
        (Vec3::new(0.0, 80.0, 20.0), Vec3::new(0.0, 0.0, -60.0)),
        (Vec3::new(0.0, 0.3, 0.0), Vec3::new(0.0, 0.3, -100.0)),
        (Vec3::new(0.0, -4.0, 0.0), Vec3::new(0.0, 2.0, -30.0)),
    ];
    let centres = [
        Vec3::ZERO,
        Vec3::new(0.0, 2.0, -1.0),
        Vec3::new(0.0, -6.0, -300.0),
        Vec3::new(5.0, 1.0, 400.0),
        Vec3::new(0.0, 0.0, -1.0e7),
    ];
    for (eye, target) in eyes {
        let behind = eye + (eye - target) * 30.0;
        for sea_at in [Vec3::ZERO, Vec3::new(0.0, -1.0e6, 0.0), eye, behind] {
            let first = sorted(eye, target, sea_at, bias);
            for centre in centres {
                assert!(first < sorted(eye, target, centre, 0.0), "{eye} {centre}");
            }
        }
    }
}

/// The sea pipeline draws both faces and writes the depth it tests.
#[test]
fn the_sea_writes_its_depth() {
    let mut descriptor = RenderPipelineDescriptor {
        depth_stencil: Some(DepthStencilState {
            format: TextureFormat::Depth32Float,
            depth_write_enabled: Some(false),
            depth_compare: Some(CompareFunction::GreaterEqual),
            stencil: default(),
            bias: default(),
        }),
        ..default()
    };
    surface(&mut descriptor);
    let depth = descriptor.depth_stencil.expect("a depth state");
    assert_eq!(descriptor.primitive.cull_mode, None);
    assert_eq!(depth.depth_write_enabled, Some(true));
    assert_eq!(depth.depth_compare, Some(CompareFunction::GreaterEqual));
}
