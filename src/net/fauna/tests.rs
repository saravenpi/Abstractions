//! Tests of client fauna synchronization: authority tie-break, lease expiry, body switch and
//! disconnect resets, and multi-chunk sending gated on same-planet peers.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use abstractions_net::host::DATAGRAM_BUDGET;
use abstractions_net::ids::BodyId;
use abstractions_net::io::ClientEvent;
use abstractions_net::log::PlayerId;
use abstractions_net::wire::{
    self, Body, FAUNA_BATCH_MAX, FaunaPose, Frame, Head, Kind, Record, RobotBody, ServerMsg,
    decode_fauna, encode_fauna, push, records,
};
use bevy::asset::AssetPlugin;
use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;

use super::FaunaNet;
use crate::avatar::Looks;
use crate::fauna::motion::Motion;
use crate::fauna::senses::{Stance, Wits};
use crate::fauna::{Beast, Fade, Lead, Mood, RemoteFauna, Spot, Temper};
use crate::menus::Menu;
use crate::net::join::open;
use crate::net::link::{Link, Role, Script, Source, Wire};
use crate::net::{NetDir, driver, plugin};
use crate::render::DayClock;
use crate::saves::{SaveDir, plugin as saves_plugin};
use crate::screens::Screen;
use crate::sim::plugin as sim_plugin;
use crate::space::{Anchor, Clock, Frame as SpaceFrame, Location, Travel};
use crate::universe::Universe;
use crate::view::Heading;
use crate::world::dither::Way;

/// Seed and local player identity used across fauna sync harness tests.
const SEED: u64 = 9;
/// Local player identifier granted by the scripted server.
const YOU: PlayerId = PlayerId(3);

/// Returns the home planet id and a point on its radius.
fn ground() -> (BodyId, DVec3) {
    let u = Universe::new(SEED);
    let home = u.home();
    let r = u.planet(home).radius;
    (home, DVec3::new(0.2, 0.9, 0.3).normalize() * r)
}

/// Builds a headless app with networking registered and positioned on the home planet.
fn fauna_app(tag: &str) -> App {
    let dir = std::env::temp_dir().join(format!("abstractions-fauna-{tag}-{}", std::process::id()));
    let (home, spot) = ground();
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
        .insert_resource(Universe::new(SEED))
        .insert_resource(Anchor {
            frame: SpaceFrame::Surface(home),
            origin: spot,
            rotation: DQuat::from_rotation_arc(spot.normalize(), DVec3::Y),
            flat: false,
        })
        .insert_resource(Location {
            body: home,
            travel: Travel::OnFoot,
        })
        .add_plugins((plugin, driver::plugin, saves_plugin, sim_plugin));
    let world = app.world_mut();
    let looks = world.resource_scope(|world, mut m: Mut<Assets<Mesh>>| {
        Looks::new(
            &mut m,
            &mut world.resource_mut::<Assets<StandardMaterial>>(),
        )
    });
    world.insert_resource(looks);
    app.update();
    app
}

/// Installs a scripted [`Link`] welcomed as [`YOU`] and returns its shared queue.
fn linked(app: &mut App) -> Script {
    let script: Script = Arc::new(Mutex::new(Wire::default()));
    let mut link = Link::over(
        Source::Script(script.clone()),
        Role::Join {
            address: "friends.example:24650".to_string(),
            expected: None,
        },
    );
    link.you = Some(YOU);
    open(&mut app.world_mut().commands(), link);
    app.world_mut().flush();
    script
}

/// Builds a datagram batch carrying one `Kind::Robot` record so a peer appears on `body`.
fn peer_robot(player: PlayerId, frame: wire::Frame, pos: DVec3) -> Vec<u8> {
    let mut body = Vec::new();
    RobotBody {
        position: pos,
        mode: 1,
        ..RobotBody::default()
    }
    .encode(&frame, &mut body);
    let head = Head {
        player,
        kind: Kind::Robot,
        frame,
        time: 1.0,
        flags: 0,
    };
    let mut out = Vec::new();
    push(&mut out, &Record { head, body: &body });
    out
}

/// Builds a datagram batch carrying one `Kind::Fauna` pose for `(herd, slot)` from `sender`.
fn fauna_datagram(
    sender: PlayerId,
    body: BodyId,
    time: f64,
    (herd, slot): (u64, u8),
    pos: DVec3,
) -> Vec<u8> {
    let pose = FaunaPose {
        herd,
        slot,
        species: 2,
        pos,
        vel: Vec3::new(1.5, 0.0, -0.5),
        heading: Vec3::X,
        stance: 3,
        excite: 0.7,
    };
    let mut encoded = Vec::new();
    encode_fauna(pos, &[pose], &mut encoded);
    let head = Head {
        player: sender,
        kind: Kind::Fauna,
        frame: Frame::Surface(body),
        time,
        flags: 0,
    };
    let mut datagram = Vec::new();
    push(
        &mut datagram,
        &Record {
            head,
            body: &encoded,
        },
    );
    datagram
}

/// Spawns `count` living drawn beasts on `planet` in `app`.
fn spawn_beasts(app: &mut App, planet: BodyId, pos: DVec3, count: usize) {
    let radius = app.world().resource::<Universe>().planet(planet).radius;
    let spot = Spot::at(pos.normalize(), radius);
    let sp = {
        let u = app.world().resource::<Universe>();
        crate::life::of(&u.planet(planet), &u.system(planet.star).star).unwrap()
    };
    let species = &sp.species[0];
    for slot in 0..count {
        let motion = Motion::new(
            &species.body,
            Vec3::new(slot as f32, 0.0, 0.0),
            Vec3::NEG_Z,
            slot as u64,
        );
        app.world_mut().spawn(Beast {
            lead: Lead::Herd {
                id: 900,
                slot: slot as u8,
            },
            index: 0,
            spot,
            motion,
            parts: Vec::new(),
            mood: Mood::default(),
            temper: Temper::of(species),
            wits: Wits::of(species),
            fade: Fade::new(Way::In, 0.0),
        });
    }
}

/// Authority tie-break keeps lower player ids when contested, yet adopts an unowned creature sent
/// by a higher player id without fighting.
#[test]
fn authority_tie_break_prefers_lowest_id_and_adopts_unowned_owner() {
    let mut app = fauna_app("tie");
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
        0.11,
    )));
    let script = linked(&mut app);
    let (body, spot) = ground();
    let other_pos = spot + DVec3::new(8.0, 0.0, 0.0);
    spawn_beasts(&mut app, body, spot, 1);

    script
        .lock()
        .unwrap()
        .events
        .push_back(ClientEvent::Datagram(peer_robot(
            PlayerId(5),
            Frame::Surface(body),
            spot,
        )));
    app.update();
    assert!(app.world().resource::<FaunaNet>().owned.contains(&(900, 0)));

    {
        let mut w = script.lock().unwrap();
        w.events.push_back(ClientEvent::Datagram(fauna_datagram(
            PlayerId(5),
            body,
            1.0,
            (900, 0),
            other_pos,
        )));
        w.events.push_back(ClientEvent::Datagram(fauna_datagram(
            PlayerId(5),
            body,
            1.0,
            (20, 0),
            other_pos,
        )));
    }
    app.update();
    assert!(
        app.world()
            .resource::<RemoteFauna>()
            .get((900, 0))
            .is_none()
    );
    assert!(app.world().resource::<RemoteFauna>().get((20, 0)).is_some());
    assert!(app.world().resource::<FaunaNet>().owned.contains(&(900, 0)));

    {
        let mut w = script.lock().unwrap();
        w.events.push_back(ClientEvent::Datagram(fauna_datagram(
            PlayerId(1),
            body,
            1.1,
            (900, 0),
            spot + DVec3::Z * 4.0,
        )));
        w.events.push_back(ClientEvent::Datagram(fauna_datagram(
            PlayerId(4),
            body,
            1.1,
            (20, 0),
            spot + DVec3::Z * 6.0,
        )));
        w.events.push_back(ClientEvent::Datagram(fauna_datagram(
            PlayerId(6),
            body,
            1.2,
            (20, 0),
            spot + DVec3::Z * 99.0,
        )));
    }
    app.update();
    let net = app.world().resource::<FaunaNet>();
    assert!(!net.owned.contains(&(900, 0)));
    assert_eq!(
        (net.owners[&(900, 0)].owner, net.owners[&(20, 0)].owner),
        (PlayerId(1), PlayerId(4))
    );
    let remote = app.world().resource::<RemoteFauna>();
    assert_eq!(remote.get((900, 0)).unwrap().stance, Stance::Threaten);
    assert!(
        remote
            .get((20, 0))
            .unwrap()
            .pos
            .distance(spot + DVec3::Z * 6.0)
            < 1e-2
    );
}

/// A departing player's left message and an expired lease both release remote ownership cleanly.
#[test]
fn leave_and_lease_expiry_release_ownership() {
    let mut app = fauna_app("lease");
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
        0.10,
    )));
    let script = linked(&mut app);
    let (body, spot) = ground();
    {
        let mut w = script.lock().unwrap();
        w.events.push_back(ClientEvent::Datagram(fauna_datagram(
            PlayerId(1),
            body,
            1.0,
            (30, 0),
            spot,
        )));
        w.events.push_back(ClientEvent::Datagram(fauna_datagram(
            PlayerId(2),
            body,
            1.0,
            (31, 0),
            spot,
        )));
    }
    app.update();
    assert_eq!(app.world().resource::<RemoteFauna>().targets.len(), 2);

    script
        .lock()
        .unwrap()
        .events
        .push_back(ClientEvent::Message(ServerMsg::Left(PlayerId(1))));
    app.update();
    assert!(app.world().resource::<RemoteFauna>().get((30, 0)).is_none());
    assert!(app.world().resource::<RemoteFauna>().get((31, 0)).is_some());

    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
        0.80,
    )));
    app.update();
    assert!(app.world().resource::<RemoteFauna>().get((31, 0)).is_none());
}

/// Switching bodies or disconnecting clears all remote creature targets.
#[test]
fn body_switch_and_disconnect_clear_remote_fauna() {
    let mut app = fauna_app("clear");
    let script = linked(&mut app);
    let (body, spot) = ground();
    let other_body = BodyId {
        index: body.index ^ 1,
        ..body
    };

    script
        .lock()
        .unwrap()
        .events
        .push_back(ClientEvent::Datagram(fauna_datagram(
            PlayerId(2),
            body,
            1.0,
            (40, 0),
            spot,
        )));
    app.update();
    assert!(app.world().resource::<RemoteFauna>().get((40, 0)).is_some());

    app.world_mut().resource_mut::<Location>().body = other_body;
    app.update();
    assert!(app.world().resource::<RemoteFauna>().targets.is_empty());

    script
        .lock()
        .unwrap()
        .events
        .push_back(ClientEvent::Datagram(fauna_datagram(
            PlayerId(2),
            other_body,
            2.0,
            (41, 0),
            spot,
        )));
    app.update();
    assert!(app.world().resource::<RemoteFauna>().get((41, 0)).is_some());

    app.world_mut().remove_resource::<Link>();
    app.update();
    assert!(app.world().resource::<RemoteFauna>().targets.is_empty());
}

/// Outgoing fauna datagrams stay silent while solo on a planet and split into budget-sized chunks
/// covering all 40 drawn creatures once another player shares that planet.
#[test]
fn sends_chunked_batches_only_when_peer_shares_body() {
    let mut app = fauna_app("send");
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
        0.11,
    )));
    let script = linked(&mut app);
    let (body, spot) = ground();
    spawn_beasts(&mut app, body, spot, 40);

    app.update();
    app.update();
    let solo: usize = script
        .lock()
        .unwrap()
        .datagrams
        .iter()
        .map(|b| records(b).filter(|r| r.head.kind == Kind::Fauna).count())
        .sum();
    assert_eq!(solo, 0);

    script.lock().unwrap().datagrams.clear();
    script
        .lock()
        .unwrap()
        .events
        .push_back(ClientEvent::Datagram(peer_robot(
            PlayerId(8),
            Frame::Surface(body),
            spot,
        )));
    app.update();
    app.update();

    let sent_batches = script.lock().unwrap().datagrams.clone();
    let mut keys = Vec::new();
    let mut fauna_records = 0;
    for datagram in &sent_batches {
        assert!(datagram.len() <= DATAGRAM_BUDGET);
        for rec in records(datagram) {
            if rec.head.kind == Kind::Fauna {
                assert_eq!(
                    (rec.head.player, rec.head.frame),
                    (YOU, Frame::Surface(body))
                );
                let decoded = decode_fauna(rec.body).unwrap();
                assert!(decoded.poses.len() <= FAUNA_BATCH_MAX);
                keys.extend(decoded.poses.into_iter().map(|p| (p.herd, p.slot)));
                fauna_records += 1;
            }
        }
    }
    keys.sort_unstable();
    keys.dedup();
    assert!(fauna_records >= 2 && keys.len() == 40);
}
