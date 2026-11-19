//! The yard the solid tests run in: a player walking on the test world with the companion
//! following it through the real sweep, among boxes added to a headless physics world.

use std::time::Duration;

use avian3d::prelude::*;
use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use super::brain::Brain;
use super::motion::{Float, chase};
use super::solid::Solids;
use super::spot::{Probe, Stance};
use crate::space::Anchor;
use crate::universe::BodyId;
use crate::world::Terrain;

/// Frame time of the simulated world.
const DT: f32 = 1.0 / 60.0;

/// Seed of the test world.
const SEED: u64 = 3;

/// A player walking on the test world with the companion beside it, among static solids.
pub struct Yard {
    /// The physics world, which holds the solids, the terrain and the anchor.
    app: App,
    /// The companion's root.
    bot: Entity,
    /// Position of the player's eyes.
    eye: Vec3,
    /// Where the player looks.
    forward: Vec3,
}

impl Yard {
    /// Returns the world with the companion settled at its station beside a player standing
    /// on the ground at the origin and looking along `forward`.
    pub fn new(forward: Vec3) -> Self {
        let mut app = App::new();
        let frame = TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(DT));
        app.add_plugins((MinimalPlugins, AssetPlugin::default(), TransformPlugin))
            .add_plugins(PhysicsPlugins::default())
            .init_asset::<Mesh>()
            .insert_resource(frame);
        app.finish();
        app.cleanup();
        let terrain = Terrain::new(SEED);
        let eye = Vec3::Y * (terrain.height(0.0, 0.0).max(0.0) + 1.6);
        let anchor = Anchor::surface(BodyId::default(), &terrain, Vec3::ZERO);
        app.insert_resource(terrain).insert_resource(anchor);
        let bot = (Transform::from_translation(eye + Vec3::X), Float::default());
        let bot = app.world_mut().spawn((bot, Brain::new(SEED))).id();
        let mut yard = Self {
            app,
            bot,
            eye,
            forward,
        };
        yard.walk(Vec3::ZERO, 3.0);
        yard
    }

    /// Returns where the companion is.
    pub fn at(&self) -> Vec3 {
        self.app
            .world()
            .get::<Transform>(self.bot)
            .unwrap()
            .translation
    }

    /// Returns where the companion's station is now, cleared of the ground.
    pub fn station(&self) -> Vec3 {
        let world = self.app.world();
        let (goal, clearance) = world.get::<Brain>(self.bot).unwrap().goal();
        let probe = Probe {
            anchor: world.resource::<Anchor>(),
            terrain: world.resource::<Terrain>(),
        };
        probe.clear(goal, clearance)
    }

    /// Adds a static box of `size` at `at` and lets the physics world take it in.
    pub fn solid(&mut self, at: Vec3, size: Vec3, body: RigidBody) {
        let shape = Collider::cuboid(size.x, size.y, size.z);
        let place = Transform::from_translation(at);
        self.app.world_mut().spawn((body, shape, place));
        (0..3).for_each(|_| self.app.update());
    }

    /// Moves the player at `velocity` for `seconds`, the companion following with the sweep,
    /// and returns where the companion was after each frame.
    pub fn walk(&mut self, velocity: Vec3, seconds: f32) -> Vec<Vec3> {
        let bot = self.bot;
        let mut path = Vec::new();
        for _ in 0..(seconds / DT) as usize {
            self.eye += velocity * DT;
            let stance = Stance {
                eye: self.eye,
                forward: self.forward,
                up: Vec3::Y,
            };
            let world = self.app.world_mut();
            let anchor = *world.resource::<Anchor>();
            let mut brain = world.get_mut::<Brain>(bot).unwrap();
            brain.observe(&anchor, stance, DT);
            brain.look = stance.eye + stance.forward * 10.0;
            let step =
                move |terrain: Res<Terrain>,
                      anchor: Res<Anchor>,
                      mut solids: Solids,
                      mut bots: Query<(&mut Transform, &mut Float, &Brain)>| {
                    let probe = Probe {
                        anchor: &anchor,
                        terrain: &terrain,
                    };
                    let (mut at, mut float, brain) = bots.get_mut(bot).unwrap();
                    chase(&probe, &mut solids, brain, (&mut at, &mut float), DT);
                };
            world.run_system_once(step).unwrap();
            path.push(self.at());
        }
        path
    }
}

/// Returns how far `at` is outside the box of `size` centred on `centre`, negative inside.
pub fn outside(at: Vec3, centre: Vec3, size: Vec3) -> f32 {
    let off = (at - centre).abs() - size * 0.5;
    off.max(Vec3::ZERO).length() + off.max_element().min(0.0)
}
