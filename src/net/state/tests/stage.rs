//! The headless app of the state tests: the session seam and driver with a scripted link, the
//! asset stores and the shared robot look, and a world standing on the home planet.

use std::sync::{Arc, Mutex};

use abstractions_net::wire::{self, Head, Kind, Record, RobotBody, push};
use bevy::asset::AssetPlugin;
use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;

use crate::avatar::Looks;
use crate::menus::Menu;
use crate::net::join::open;
use crate::net::link::{Link, Role, Script, Source, Wire};
use crate::net::{NetDir, Remote};
use crate::render::DayClock;
use crate::saves::SaveDir;
use crate::screens::Screen;
use crate::sim::PlayerId;
use crate::space::{Anchor, Clock, Frame, Location, Placed, Travel};
use crate::universe::{BodyId, Universe};
use crate::view::Heading;

/// Seed of the universe of these tests.
pub const SEED: u64 = 9;

/// The id the scripted server gave this player.
pub const YOU: PlayerId = PlayerId(3);

/// Returns the home planet of the universe of these tests and a point on its ground, where
/// the anchor sits.
pub fn ground() -> (BodyId, DVec3) {
    let universe = Universe::new(SEED);
    let home = universe.home();
    let radius = universe.planet(home).radius;
    (home, DVec3::new(0.2, 0.9, 0.3).normalize() * radius)
}

/// Puts the world on the home planet: the universe, the player on foot there and the anchor
/// on the [`ground`], render +Y up.
fn stand(app: &mut App) {
    let (home, spot) = ground();
    let anchor = Anchor {
        frame: Frame::Surface(home),
        origin: spot,
        rotation: DQuat::from_rotation_arc(spot.normalize(), DVec3::Y),
        flat: false,
    };
    app.insert_resource(Universe::new(SEED))
        .insert_resource(anchor)
        .insert_resource(Location {
            body: home,
            travel: Travel::OnFoot,
        });
}

/// Returns a headless app with the network seam and driver, the asset stores and the robot
/// look, its files in a scratch folder named after `tag`.
pub fn app(tag: &str) -> App {
    let dir = std::env::temp_dir().join(format!("abstractions-state-{tag}-{}", std::process::id()));
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin, AssetPlugin::default()))
        .init_asset::<Mesh>()
        .init_asset::<Image>()
        .init_asset::<StandardMaterial>()
        .init_state::<Screen>()
        .init_state::<Menu>()
        .insert_resource(SaveDir(dir.join("worlds")))
        .insert_resource(NetDir(dir.join("config")))
        .init_resource::<DayClock>()
        .init_resource::<Heading>()
        .init_resource::<Clock>()
        .add_plugins((
            crate::net::plugin,
            crate::net::driver::plugin,
            crate::saves::plugin,
            crate::sim::plugin,
        ));
    let world = app.world_mut();
    let looks = world.resource_scope(|world, mut meshes: Mut<Assets<Mesh>>| {
        Looks::new(
            &mut meshes,
            &mut world.resource_mut::<Assets<StandardMaterial>>(),
        )
    });
    world.insert_resource(looks);
    stand(&mut app);
    app.update();
    app
}

/// Inserts a scripted link the server welcomed as [`YOU`], and returns its script.
pub fn linked(app: &mut App) -> Script {
    let script: Script = Arc::new(Mutex::new(Wire::default()));
    let role = Role::Join {
        address: "friends.example:24650".to_string(),
        expected: None,
    };
    let world = app.world_mut();
    let mut link = Link::over(Source::Script(script.clone()), role);
    link.you = Some(YOU);
    open(&mut world.commands(), link);
    world.flush();
    script
}

/// Returns a batch holding the robot of `player` in `frame` at universe time `time`, at `pos`
/// moving at `vel`.
pub fn robot(
    player: PlayerId,
    frame: wire::Frame,
    time: f64,
    (pos, vel): (DVec3, DVec3),
) -> Vec<u8> {
    let body = RobotBody {
        position: pos,
        velocity: vel.as_vec3(),
        mode: 1,
        speed: vel.length() as f32,
        ..RobotBody::default()
    };
    let mut bytes = Vec::new();
    body.encode(&frame, &mut bytes);
    let head = Head {
        player,
        kind: Kind::Robot,
        frame,
        time,
        flags: 0,
    };
    let mut batch = Vec::new();
    push(&mut batch, &Record { head, body: &bytes });
    batch
}

/// Runs one frame at universe time `time`.
pub fn frame_at(app: &mut App, time: f64) {
    app.world_mut().resource_mut::<Clock>().seconds = time;
    app.update();
}

/// Returns the pose of every robot puppet.
pub fn puppets(app: &mut App) -> Vec<Placed> {
    let mut query = app
        .world_mut()
        .query_filtered::<&Placed, (With<Remote>, With<crate::avatar::AvatarPose>)>();
    query.iter(app.world()).copied().collect()
}
