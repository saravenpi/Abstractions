//! Spawning the ship: landed near the spawn point in a new game, or where the save left it.

use avian3d::prelude::*;
use bevy::ecs::system::SystemParam;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::berth::{landing_spot, parked};
use super::body::{SavedFrame, Ship};
use super::glow::{Engine, Plume};
use super::mesh::{NOZZLE_Z, Skin, hull_meshes, material, plume_mesh};
use super::save::saved;
use crate::saves::ActiveWorld;
use crate::screens::Screen;
use crate::space::{Location, Placed};
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
    /// Which planet that is.
    location: Res<'w, Location>,
    /// The world being played, with its saved ship.
    active: Option<Res<'w, ActiveWorld>>,
}

impl Start<'_> {
    /// Returns the ship a world starts with: the saved one when it rests on the planet the
    /// player starts on, else a ship landed on dry, flat ground near the spawn point.
    fn ship(&self) -> Ship {
        let body = self.location.body;
        let kept = saved(self.active.as_deref());
        let kept = kept.filter(|state| state.landed && state.frame == SavedFrame::Surface(body));
        if let Some(state) = kept {
            return Ship::from_state(&state);
        }
        let globe = self.terrain.globe();
        let height = |pos: DVec3| f64::from(globe.height(pos.normalize()));
        let at = self.spawn.0;
        let near = self.terrain.embed(at.x, at.y, at.z).normalize() * globe.radius();
        parked(body, &height, landing_spot(&height, near), globe.radius())
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

/// Spawns the ship with its parts once the spawn point exists, and resets the flight state.
pub fn spawn(
    mut commands: Commands,
    start: Start,
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
        Engine { glow: glow.clone() },
        Transform::default(),
        Visibility::default(),
        DespawnOnEnter(Screen::Title),
        RigidBody::Kinematic,
        Collider::cuboid(HULL_BOX.x, HULL_BOX.y, HULL_BOX.z),
    );
    let mut entity = commands.spawn(root);
    for (mesh, skin) in hull_meshes().into_iter().zip(skins) {
        entity.with_child((Mesh3d(yard.meshes.add(mesh)), MeshMaterial3d(skin)));
    }
    entity.with_child((
        Plume,
        Mesh3d(yard.meshes.add(plume_mesh())),
        MeshMaterial3d(glow),
        Transform::from_xyz(0.0, 0.0, NOZZLE_Z).with_scale(Vec3::new(1.0, 1.0, 0.02)),
    ));
    super::reset(&mut commands);
}
