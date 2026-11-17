//! Spawning the ship: landed near the spawn point in a new game, or where the save left it.

use avian3d::prelude::*;
use bevy::ecs::system::SystemParam;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::berth::{landing_spot, parked};
use super::body::Ship;
use super::glow::{Engine, Plume};
use super::landing::GEAR_DROP;
use super::mesh::{NOZZLE_Z, Skin, hull_meshes, material, plume_mesh};
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
        let spot = landing_spot(&height, near);
        parked(self.location.body, &height, spot, globe.radius())
    }

    /// Returns the ship a world starts with: the saved one exactly where it was, landed, flying
    /// or in space, else the ship of a new game. A player saved in the seat starts seated.
    fn ship(&mut self) -> Ship {
        let state = saved(self.active.as_deref());
        let kept = state.and_then(|state| Some((Ship::from_state(&state)?, state.seated)));
        let Some((ship, seated)) = kept else {
            return self.fresh();
        };
        if seated {
            self.location.travel = Travel::InShip;
        }
        ship
    }
}

/// The asset stores the ship parts go into.
#[derive(SystemParam)]
pub struct Yard<'w> {
    /// Mesh assets.
    meshes: ResMut<'w, Assets<Mesh>>,
    /// Material assets.
    materials: ResMut<'w, Assets<StandardMaterial>>,
}

/// Adds the hull and the exhaust to the ship entity, each part with its skin.
fn dress(entity: &mut EntityCommands, yard: &mut Yard, skins: [Handle<StandardMaterial>; 5]) {
    let glow = skins[Skin::Glow as usize].clone();
    for (mesh, skin) in hull_meshes().into_iter().zip(skins) {
        entity.with_child((Mesh3d(yard.meshes.add(mesh)), MeshMaterial3d(skin)));
    }
    entity.with_child((
        Plume,
        Mesh3d(yard.meshes.add(plume_mesh())),
        MeshMaterial3d(glow),
        Transform::from_xyz(0.0, 0.0, NOZZLE_Z).with_scale(Vec3::new(1.0, 1.0, 0.02)),
    ));
}

/// Hands every part of the ship to the pipeline warm-up, so its pipelines are ready whenever
/// it is spawned, before or after the warm-up ran.
pub fn warm(mut warm: ResMut<Warm>, mut yard: Yard) {
    let skins = Skin::ALL.map(|skin| yard.materials.add(material(skin)));
    let glow = skins[Skin::Glow as usize].clone();
    for (mesh, skin) in hull_meshes().into_iter().zip(skins) {
        warm.add(yard.meshes.add(mesh), skin);
    }
    warm.add(yard.meshes.add(plume_mesh()), glow);
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

/// Spawns the ship with its parts once the spawn point exists, and resets the flight state.
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
    let skins = Skin::ALL.map(|skin| yard.materials.add(material(skin)));
    let glow = skins[Skin::Glow as usize].clone();
    let root = (
        Name::new("Ship"),
        Placed {
            pos: ship.pos,
            rot: ship.rot,
        },
        ship,
        Engine { glow },
        Transform::default(),
        Visibility::default(),
        DespawnOnEnter(Screen::Title),
        RigidBody::Kinematic,
        Collider::cuboid(HULL_BOX.x, HULL_BOX.y, HULL_BOX.z),
    );
    dress(&mut commands.spawn(root), &mut yard, skins);
    super::reset(&mut commands);
}
