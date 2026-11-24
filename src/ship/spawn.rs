//! Spawning the ship: landed near the spawn point in a new game, or where the save left it.

use avian3d::prelude::*;
use bevy::ecs::system::SystemParam;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::berth::{berth_bay, landing_spot, parked};
use super::body::Ship;
use super::gear::infer;
use super::glow::Engine;
use super::landing::GEAR_DROP;
use super::mesh::{Looks, Skin, dress, hulls, material};
use super::restore::{Restored, Sky, restore};
use super::save::saved;
use crate::home::Base;
use crate::net::{Link, Remotes};
use crate::render::Warm;
use crate::saves::ActiveWorld;
use crate::screens::Screen;
use crate::space::{Clock, Frame, Location, Placed, Travel};
use crate::universe::Universe;
use crate::world::{Resettled, SpawnPoint, Terrain};

/// What the player is told when the ship came back inside a planet, a moon or a star and was
/// put on an orbit around it.
pub const IN_ORBIT: &str = "Your ship could not come back where it was. It is in orbit instead.";

/// What the player is told when the ship was not where the player is, or could not come back
/// where it was, and was landed beside the player.
pub const BROUGHT: &str = "Your ship could not come back where it was. It is landed beside you.";

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
    /// The universe the saved ship is checked against.
    universe: Option<Res<'w, Universe>>,
    /// Universe time, which says where the bodies are now.
    clock: Option<Res<'w, Clock>>,
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

    /// Returns the ship a world starts with, and a line for the player when it is not where the
    /// save left it. The saved ship comes back exactly where it was, landed, flying or in orbit,
    /// beside the body it was beside, and a player saved in the seat starts seated. A ship that
    /// would come back inside a body with the player in it is on a safe orbit instead. A ship
    /// without the player that is lost, inside a body or not on the body the player stands on
    /// is landed beside the player, as the ship of a new game is: no start leaves the player
    /// without a ship. A record that does not hold the gear gets the gear its height implies.
    fn ship(&mut self) -> (Ship, Option<&'static str>) {
        let here = Frame::Surface(self.location.body);
        let time = self.clock.as_deref().map_or(0.0, |clock| clock.seconds);
        let land = (self.location.body, self.terrain.globe().as_ref());
        let known = self.universe.as_deref().zip(saved(self.active.as_deref()));
        let back = known.map(|(universe, state)| {
            restore(
                state,
                &Sky {
                    universe,
                    time,
                    land,
                },
            )
        });
        let (mut ship, seated, line) = match back {
            Some(Restored::Kept(ship, seat)) if seat || ship.frame == here => (ship, seat, None),
            Some(Restored::Orbit(ship)) => (ship, true, Some(IN_ORBIT)),
            Some(_) => return (self.fresh(), Some(BROUGHT)),
            None => return (self.fresh(), None),
        };
        let old = saved(self.active.as_deref()).is_some_and(|state| state.gear.is_none());
        if old && line.is_none() {
            let radar = self.radar(&ship);
            infer(&mut ship, radar);
        }
        if seated {
            self.location.travel = Travel::InShip;
        }
        (ship, line)
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
    (base, terrain): (Res<Base>, Res<Terrain>),
    (link, remotes): (Option<Res<Link>>, Option<Res<Remotes>>),
    ship: Single<(Entity, &mut Ship, &mut Placed), Without<Berthed>>,
) {
    let (entity, mut ship, mut placed) = ship.into_inner();
    commands.entity(entity).insert(Berthed);
    if ship.frame != Frame::Surface(base.body) {
        return;
    }
    let slot = link
        .as_deref()
        .and_then(|l| l.you)
        .map_or(0, |id| id.0.saturating_sub(1));
    let occupied: Vec<DVec3> = remotes
        .as_deref()
        .into_iter()
        .flat_map(|r| r.ships.values())
        .filter(|t| t.frame == abstractions_net::wire::Frame::Surface(base.body))
        .map(|t| t.latest.position)
        .collect();
    let globe = terrain.globe();
    let height = |pos: DVec3| f64::from(globe.height(pos.normalize()));
    let sea = f64::from(globe.sea_level());
    let (pos, rot) = berth_bay(&base, &height, (globe.radius(), sea), slot, &occupied);
    ship.pos = pos;
    ship.rot = rot;
    (ship.landed, ship.touching) = (true, true);
    *placed = Placed {
        pos: ship.pos,
        rot: ship.rot,
    };
}

/// Returns the collider of the ship root: one convex hull for each part of the airframe as it is
/// drawn, the fuselage, the canopy, the fin, the heat shield, the pods of the gear and of the
/// engines, the wings and the winglets, so that the player stands on the wings and bumps into them.
pub fn airframe() -> Collider {
    let part = |points: Vec<Vec3>| {
        let hull = Collider::convex_hull(&points).expect("the airframe parts are solid");
        (Vec3::ZERO, Quat::IDENTITY, hull)
    };
    Collider::compound(hulls().into_iter().map(part).collect())
}

/// Spawns the ship with its parts once the spawn point exists, and resets the flight state. Its
/// engine glow is the one material of its own: its colour follows its own engine. A ship that
/// is not where the save left it leaves a line for the player.
pub fn spawn(
    mut commands: Commands,
    mut start: Start,
    mut yard: Yard,
    existing: Query<(), With<Ship>>,
) {
    if !existing.is_empty() {
        return;
    }
    let (ship, line) = start.ship();
    if let Some(line) = line {
        commands.insert_resource(Resettled(line));
    }
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
        airframe(),
    );
    dress(&mut commands.spawn(root), &looks, glow);
    super::reset(&mut commands);
}
