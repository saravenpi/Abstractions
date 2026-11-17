//! The works: finding the base and building its plan off the main thread, once per world.

use std::sync::Arc;

use bevy::ecs::system::SystemParam;
use bevy::material::OpaqueRendererMethod;
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future::poll_once};

use super::base::Base;
use super::draft::Block;
use super::glow::{Glow, Mood};
use super::plan::{Plan, plan};
use super::soup::Soup;
use crate::render::Warm;
use crate::universe::Universe;
use crate::world::{Globe, Planet};

/// The parts of the base, ready to spawn as often as the player comes home.
pub struct Kit {
    /// Mesh of everything lit.
    pub lit: Handle<Mesh>,
    /// Material of the lit mesh.
    pub paint: Handle<StandardMaterial>,
    /// Mesh and material of each glowing part, in the order of [`Glow::ALL`].
    pub glows: [(Handle<Mesh>, Handle<StandardMaterial>); 6],
    /// Boxes nothing walks through.
    pub blocks: Vec<Block>,
    /// Where each light that is cast sits.
    pub lamps: Vec<(Glow, Vec3)>,
}

/// What is being worked out for the base of this world.
#[derive(Resource, Default)]
pub struct Works {
    /// The search for the site, while it runs.
    survey: Option<Task<Base>>,
    /// The drawing of the plan, while it runs.
    drawing: Option<Task<Plan>>,
    /// The finished parts.
    pub kit: Option<Kit>,
}

/// The asset stores the parts go into.
#[derive(SystemParam)]
pub struct Stock<'w> {
    /// Mesh assets.
    meshes: ResMut<'w, Assets<Mesh>>,
    /// Material assets.
    materials: ResMut<'w, Assets<StandardMaterial>>,
}

/// Returns the material of the lit mesh: lightly brushed metal coloured by the vertex colours.
fn metal() -> StandardMaterial {
    StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.5,
        metallic: 0.25,
        reflectance: 0.5,
        opaque_render_method: OpaqueRendererMethod::Forward,
        ..default()
    }
}

/// Returns the unlit material of `glow`, which shows its base colour as it is and so can bloom.
fn unlit(glow: Glow) -> StandardMaterial {
    StandardMaterial {
        base_color: glow.color(Mood::default()),
        unlit: true,
        opaque_render_method: OpaqueRendererMethod::Forward,
        ..default()
    }
}

impl Stock<'_> {
    /// Stores the meshes and the materials of `plan` and returns the kit that names them.
    fn kit(&mut self, plan: Plan) -> Kit {
        let mut glows = plan.glows.into_iter().zip(Glow::ALL);
        Kit {
            lit: self.meshes.add(plan.lit),
            paint: self.materials.add(metal()),
            glows: std::array::from_fn(|_| {
                let (mesh, glow) = glows.next().expect("one mesh per glow");
                (self.meshes.add(mesh), self.materials.add(unlit(glow)))
            }),
            blocks: plan.blocks,
            lamps: plan.lamps,
        }
    }
}

/// Hands the two materials of the base and its vertex layout to the pipeline warm-up, so they
/// are ready whenever the base is spawned, before or after the warm-up ran.
pub fn warm(mut warm: ResMut<Warm>, mut stock: Stock) {
    let mut soup = Soup::default();
    soup.slab(Vec3::ZERO, Vec3::ONE, [1.0; 3]);
    let mesh = stock.meshes.add(soup.into_mesh());
    warm.add(mesh.clone(), stock.materials.add(metal()));
    warm.add(mesh, stock.materials.add(unlit(Glow::ALL[0])));
}

/// Finds the base of the world in a task and inserts it once found. Runs while the world has
/// no base: the world may also insert one itself when it begins.
pub fn locate(mut commands: Commands, universe: Res<Universe>, mut works: ResMut<Works>) {
    if let Some(task) = works.survey.as_mut() {
        if let Some(base) = block_on(poll_once(task)) {
            commands.insert_resource(base);
            works.survey = None;
        }
        return;
    }
    let home = universe.home();
    let spec = universe.planet(home);
    let search = async move { Base::on(&Arc::new(Globe::of(&spec)), home) };
    works.survey = Some(AsyncComputeTaskPool::get().spawn(search));
}

/// Draws the plan of the base in a task while its planet is the streamed one, and stores the
/// finished parts. Does nothing once the kit exists.
pub fn prepare(
    base: Res<Base>,
    planet: Option<Res<Planet>>,
    mut works: ResMut<Works>,
    mut stock: Stock,
) {
    if works.kit.is_some() {
        return;
    }
    if let Some(task) = works.drawing.as_mut() {
        if let Some(plan) = block_on(poll_once(task)) {
            works.kit = Some(stock.kit(plan));
            works.drawing = None;
        }
        return;
    }
    if let Some(planet) = planet.filter(|planet| planet.body == base.body) {
        let (globe, base) = (planet.globe.clone(), *base);
        let drawing = async move { plan(&base.lot(&globe)) };
        works.drawing = Some(AsyncComputeTaskPool::get().spawn(drawing));
    }
}
