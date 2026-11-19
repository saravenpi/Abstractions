//! Spawning the ship: landed near the spawn point in a new game, or where the save left it.

use avian3d::prelude::*;
use bevy::ecs::system::SystemParam;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::berth::{landing_spot, parked};
use super::body::Ship;
use super::gear::infer;
use super::glow::Engine;
use super::landing::GEAR_DROP;
use super::mesh::{Looks, Skin, dress, material};
use super::save::saved;
use crate::home::Base;
use crate::render::Warm;
use crate::saves::ActiveWorld;
use crate::screens::Screen;
use crate::space::{Frame, Location, Placed, Travel};
use crate::world::{SpawnPoint, Terrain};

/// Width, height and length in metres of the box that stops the player from walking through the hull.
const HULL_BOX: Vec3 = Vec3::new(1.8, 1.3, 6.4);

/// Where a new world puts the player, and the save it may come from.
#[derive(SystemParam)]
pub struct Start<'w> {
    /// Spawn point of the player in chart coordinates.
    spawn: Res<'w, SpawnPoint>,
    /// Ground of the planet the player starts on.
    terrain: Res<'w, Terrain>,
    /// Which planet that is, and whether the player starts seated.
    location: ResMut<'w, Location>,
    /// The world being played, with its saved ship.
    active: Option<Res<'w, ActiveWorld>>,
}

impl Start<'_> {
    /// Returns the ship of a new game: landed on dry, flat ground near the spawn point.
    fn fresh(&self) -> Ship {
        let globe = self.terrain.globe();
        let height = |pos: DVec3| f64::from(globe.height(pos.normalize()));
        let at = self.spawn.0;
        let near = self.terrain.embed(at.x, at.y, at.z).normalize() * globe.radius();
        let sea = f64::from(globe.sea_level());
        let spot = landing_spot(&height, near, sea);
        parked(self.location.body, (&height, sea), spot, globe.radius())
    }

    /// Returns the height in metres of the gear of `ship` above the ground or the sea of the
    /// planet the world starts on, infinite when the ship is not in the frame of that planet.
    fn radar(&self, ship: &Ship) -> f64 {
        let globe = self.terrain.globe();
        if ship.frame != Frame::Surface(self.location.body) {
            return f64::INFINITY;
        }
        let floor = globe.height(ship.pos.normalize()).max(globe.sea_level());
        ship.pos.length() - globe.radius() - f64::from(floor) - GEAR_DROP
    }

    /// Returns the ship a world starts with: the saved one exactly where it was, landed, flying
    /// or in space, else the ship of a new game. A record that does not hold the gear gets the
    /// gear its height implies. A player saved in the seat starts seated.
    fn ship(&mut self) -> Ship {
        let state = saved(self.active.as_deref());
        let kept = state.and_then(|state| Some((Ship::from_state(&state)?, state.seated)));
        let Some((mut ship, seated)) = kept else {
            return self.fresh();
        };
        if state.is_some_and(|state| state.gear.is_none()) {
            let radar = self.radar(&ship);
            infer(&mut ship, radar);
        }
        if seated {
            self.location.travel = Travel::InShip;
        }
        ship
    }
}

/// The asset stores the ship parts go into, and the shared look once it is made.
#[derive(SystemParam)]
pub struct Yard<'w> {
    /// Mesh assets.
    meshes: ResMut<'w, Assets<Mesh>>,
    /// Material assets.
    materials: ResMut<'w, Assets<StandardMaterial>>,
    /// The meshes and materials every ship shares, once the first one asked for them.
    looks: Option<Res<'w, Looks>>,
}

impl Yard<'_> {
    /// Returns the look every ship shares. It is made the first time and kept as a resource.
    pub fn looks(&mut self, commands: &mut Commands) -> Looks {
        if let Some(looks) = self.looks.as_deref() {
            return looks.clone();
        }
        let looks = Looks::new(&mut self.meshes, &mut self.materials);
        commands.insert_resource(looks.clone());
        looks
    }
}

/// Hands every part of the ship to the pipeline warm-up, so its pipelines are ready whenever
/// it is spawned, before or after the warm-up ran.
pub fn warm(mut commands: Commands, mut warm: ResMut<Warm>, mut yard: Yard) {
    for (mesh, material) in yard.looks(&mut commands).parts() {
        warm.add(mesh, material);
    }
}

/// Marks a ship that already took its place on the landing pad of the home base.
#[derive(Component, Debug, Clone, Copy)]
pub struct Berthed;

/// Parks the ship of a new game on the landing pad of the home base, its nose along the pad
/// heading, once the base is found while the world loads.
///
/// The base is found in the background. If it is not there before the game begins, the ship
/// stays where it was spawned, near the terrain spawn.
pub fn berth_at_base(
    mut commands: Commands,
    base: Res<Base>,
    ship: Single<(Entity, &mut Ship, &mut Placed), Without<Berthed>>,
) {
    let (entity, mut ship, mut placed) = ship.into_inner();
    commands.entity(entity).insert(Berthed);
    if ship.frame != Frame::Surface(base.body) {
        return;
    }
    let up = base.pad.normalize();
    ship.pos = base.pad + up * GEAR_DROP;
    ship.rot = base.pad_rot();
    (ship.landed, ship.touching) = (true, true);
    *placed = Placed {
        pos: ship.pos,
        rot: ship.rot,
    };
}

/// Spawns the ship with its parts once the spawn point exists, and resets the flight state. Its
/// engine glow is the one material of its own: its colour follows its own engine.
pub fn spawn(
    mut commands: Commands,
    mut start: Start,
    mut yard: Yard,
    existing: Query<(), With<Ship>>,
) {
    if !existing.is_empty() {
        return;
    }
    let ship = start.ship();
    let looks = yard.looks(&mut commands);
    let glow = yard.materials.add(material(Skin::Glow));
    let root = (
        Name::new("Ship"),
        Placed {
            pos: ship.pos,
            rot: ship.rot,
        },
        ship,
        Engine::new(glow.clone()),
        Transform::default(),
        Visibility::default(),
        DespawnOnEnter(Screen::Title),
        RigidBody::Kinematic,
        Collider::cuboid(HULL_BOX.x, HULL_BOX.y, HULL_BOX.z),
    );
    dress(&mut commands.spawn(root), &looks, glow);
    super::reset(&mut commands);
}
