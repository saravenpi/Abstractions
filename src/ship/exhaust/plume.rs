//! The plume: three nested shells of one mesh, a white-hot core, a coloured mantle and a faint
//! outer glow, drawn additively by `exhaust.wesl`. The mesh is one unit long along +Z with a
//! unit radius at the nozzle; the length, the width and the brightness are set from the real
//! thrust and the air. A ship has three materials, for its main engines, its lift nozzles and
//! its attitude jets, which all the plumes of a kind share.

use bevy::asset::embedded_asset;
use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::pbr::{Material, MaterialPipeline, MaterialPipelineKey, MaterialPlugin};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, ShaderType, SpecializedMeshPipelineError,
};
use bevy::shader::ShaderRef;

use super::jets::Plume;
use super::state::Exhaust;

/// Uniform block read by `exhaust.wesl`. Every field is a `vec4`, so the layout never needs
/// padding. The fields are the same, in the same order, as in the shader.
#[derive(Clone, Copy, Debug, Default, PartialEq, ShaderType)]
pub struct JetUniform {
    /// Brightness in x, the spread of the plume against thick air in y, the warp drive in z.
    pub drive: Vec4,
    /// Linear colour of the core in xyz, the brightness in w.
    pub hot: Vec4,
    /// Linear colour of the mantle in xyz, the plume lengths per second the noise scrolls in w.
    pub cool: Vec4,
    /// The length in metres in x, the shock diamonds in y.
    pub form: Vec4,
}

/// Material of a plume.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
pub struct ExhaustMaterial {
    /// Values for the shader.
    #[uniform(0)]
    pub jet: JetUniform,
}

impl Material for ExhaustMaterial {
    /// Points at the shader embedded in the binary.
    fn fragment_shader() -> ShaderRef {
        "embedded://abstractions/ship/exhaust/exhaust.wesl".into()
    }

    /// The exhaust adds light to the view.
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Add
    }

    /// The exhaust stays out of the depth prepass.
    fn enable_prepass() -> bool {
        false
    }

    /// The exhaust casts no shadow.
    fn enable_shadows() -> bool {
        false
    }

    /// Draws both faces, so the plume shows from inside its own shell too.
    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}

/// The plumes that have no material yet.
type Bare = Without<MeshMaterial3d<ExhaustMaterial>>;

/// Run condition: a plume has no material yet. It asks what is there, not what was just added:
/// a condition is evaluated on every frame, also while its set does not run, so a ship dressed
/// before the game plays would be seen as new once, too early, and its plumes never be drawn.
pub fn bare(plumes: Query<(), (With<Plume>, Bare)>) -> bool {
    !plumes.is_empty()
}

/// Gives every plume without a material the material of its kind, and its ship its three
/// materials the first time.
pub fn equip(
    mut commands: Commands,
    mut materials: ResMut<Assets<ExhaustMaterial>>,
    mut ships: Query<&mut Exhaust>,
    plumes: Query<(Entity, &ChildOf, &Plume), Bare>,
) {
    for (plume, parent, Plume(jet)) in &plumes {
        let Ok(mut exhaust) = ships.get_mut(parent.parent()) else {
            continue;
        };
        if exhaust.jets[0] == Handle::default() {
            exhaust.jets = [0, 1, 2].map(|_| materials.add(ExhaustMaterial::default()));
        }
        let material = exhaust.jets[jet.kind as usize].clone();
        commands.entity(plume).insert(MeshMaterial3d(material));
    }
}

/// Embeds the shader and registers the material.
pub(super) fn plugin(app: &mut App) {
    embedded_asset!(app, "exhaust.wesl");
    app.add_plugins(MaterialPlugin::<ExhaustMaterial>::default());
}
