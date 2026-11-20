//! The underwater pass in the render world: one fullscreen triangle, before tonemapping, that
//! fogs every pixel whose view starts under the water surface and draws the waterline across
//! the lens. It reads the depth prepass and only runs while part of the lens can be under water.

use bevy::asset::{embedded_asset, load_embedded_asset};
use bevy::core_pipeline::prepass::ViewPrepassTextures;
use bevy::core_pipeline::tonemapping::tonemapping;
use bevy::core_pipeline::{Core3dSystems, FullscreenShader, schedule::Core3d};
use bevy::prelude::*;
use bevy::render::extract_component::{
    ComponentUniforms, DynamicUniformIndex, ExtractComponentPlugin, UniformComponentPlugin,
};
use bevy::render::render_resource::binding_types::{texture_2d, texture_depth_2d, uniform_buffer};
use bevy::render::render_resource::{
    BindGroupEntries, BindGroupLayoutDescriptor, BindGroupLayoutEntries, CachedRenderPipelineId,
    ColorTargetState, ColorWrites, FragmentState, Operations, PipelineCache,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipelineDescriptor, ShaderStages,
    SpecializedRenderPipeline, SpecializedRenderPipelines, TextureFormat, TextureSampleType,
};
use bevy::render::renderer::{RenderContext, ViewQuery};
use bevy::render::view::{ExtractedView, ViewTarget, ViewUniform, ViewUniformOffset, ViewUniforms};
use bevy::render::{Render, RenderApp, RenderStartup, RenderSystems};
use bevy::shader::Shader;

use super::gauge::Murk;

/// The pipeline of the pass: its bind group layout and its shaders.
#[derive(Resource)]
struct MurkPipeline {
    /// Screen, depth, the values of the pass and the view.
    layout: BindGroupLayoutDescriptor,
    /// The fragment shader, `murk.wesl`.
    shader: Handle<Shader>,
    /// The vertex shader of the fullscreen triangle.
    fullscreen: FullscreenShader,
}

impl SpecializedRenderPipeline for MurkPipeline {
    type Key = TextureFormat;

    /// Builds the pipeline that writes a view target of `format`.
    fn specialize(&self, format: TextureFormat) -> RenderPipelineDescriptor {
        RenderPipelineDescriptor {
            label: Some("murk_pipeline".into()),
            layout: vec![self.layout.clone()],
            vertex: self.fullscreen.to_vertex_state(),
            fragment: Some(FragmentState {
                shader: self.shader.clone(),
                targets: vec![Some(ColorTargetState {
                    format,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
                ..default()
            }),
            ..default()
        }
    }
}

/// The pipeline a view draws the pass with.
#[derive(Component)]
struct MurkPipelineId(CachedRenderPipelineId);

/// Builds the layout and loads the shader once the render world starts.
fn init(mut commands: Commands, assets: Res<AssetServer>, fullscreen: Res<FullscreenShader>) {
    let entries = BindGroupLayoutEntries::sequential(
        ShaderStages::FRAGMENT,
        (
            texture_2d(TextureSampleType::Float { filterable: false }),
            texture_depth_2d(),
            uniform_buffer::<Murk>(true),
            uniform_buffer::<ViewUniform>(true),
        ),
    );
    commands.insert_resource(MurkPipeline {
        layout: BindGroupLayoutDescriptor::new("murk_layout", &entries),
        shader: load_embedded_asset!(assets.as_ref(), "murk.wesl"),
        fullscreen: fullscreen.clone(),
    });
}

/// Picks the pipeline of every view with the pass for the format of its target.
fn prepare(
    mut commands: Commands,
    cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedRenderPipelines<MurkPipeline>>,
    pipeline: Res<MurkPipeline>,
    views: Query<(Entity, &ExtractedView), With<Murk>>,
) {
    for (entity, view) in &views {
        let id = pipelines.specialize(&cache, &pipeline, view.target_format);
        commands.entity(entity).insert(MurkPipelineId(id));
    }
}

/// What the pass reads of a view.
type Seen = (
    &'static ViewTarget,
    &'static MurkPipelineId,
    &'static ViewPrepassTextures,
    &'static Murk,
    &'static DynamicUniformIndex<Murk>,
    &'static ViewUniformOffset,
);

/// Draws the pass over the view while part of its lens can be under water.
fn draw(
    view: ViewQuery<Seen>,
    pipeline: Res<MurkPipeline>,
    cache: Res<PipelineCache>,
    uniforms: (Res<ComponentUniforms<Murk>>, Res<ViewUniforms>),
    mut ctx: RenderContext,
) {
    let (target, id, prepass, murk, index, offset) = view.into_inner();
    let found = (
        cache.get_render_pipeline(id.0).filter(|_| murk.under > 0.0),
        prepass.depth_only_view(),
        uniforms.0.uniforms().binding(),
        uniforms.1.uniforms.binding(),
    );
    let (Some(render), Some(depth), Some(values), Some(views)) = found else {
        return;
    };
    let post = target.post_process_write();
    let layout = cache.get_bind_group_layout(&pipeline.layout);
    let entries = BindGroupEntries::sequential((post.source, depth, values, views));
    let group = ctx
        .render_device()
        .create_bind_group(Some("murk_bind_group"), &layout, &entries);
    let mut pass = ctx.begin_tracked_render_pass(RenderPassDescriptor {
        label: Some("murk"),
        color_attachments: &[Some(RenderPassColorAttachment {
            view: post.destination,
            depth_slice: None,
            resolve_target: None,
            ops: Operations::default(),
        })],
        ..default()
    });
    pass.set_render_pipeline(render);
    pass.set_bind_group(0, &group, &[index.index(), offset.offset]);
    pass.draw(0..3, 0..1);
}

/// Embeds the shader, sends the values of the pass to the render world and runs it before
/// tonemapping, so the fog mixes in linear colour.
pub(in crate::render) fn plugin(app: &mut App) {
    embedded_asset!(app, "murk.wesl");
    app.add_plugins((
        ExtractComponentPlugin::<Murk>::default(),
        UniformComponentPlugin::<Murk>::default(),
    ));
    let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
        return;
    };
    render_app
        .init_resource::<SpecializedRenderPipelines<MurkPipeline>>()
        .add_systems(RenderStartup, init)
        .add_systems(Render, prepare.in_set(RenderSystems::Prepare))
        .add_systems(
            Core3d,
            draw.in_set(Core3dSystems::PostProcess).before(tonemapping),
        );
}
