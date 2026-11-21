//! Tests of the feedback of a blow: a hit of the local robot freezes it and the animal, kicks the
//! camera, marks the crosshair and throws flecks from the contact; a miss only whooshes. Each struck
//! layer throws its own flecks, and a cut through flesh leaves them to the blood.

use bevy::ecs::message::{Message, Messages};
use bevy::prelude::*;
use bevy_persistent::StorageFormat;

use super::super::super::Avatar;
use super::super::act::{Act, Strike};
use super::super::hit::{Hit, Tip};
use super::super::outcome::Outcome;
use super::super::sense::landed;
use super::super::stage::blade;
use super::super::stop::{self, Hitstop};
use super::super::swish::{self, Missed};
use super::mark::{self, Mark};
use super::spark::{self, sprays};
use super::tune::SPARK;
use crate::audio::{Body, Impact, Outer, Whoosh};
use crate::effects::Fling;
use crate::player::Player;
use crate::settings::{Persistent, Settings};
use crate::sim::PlayerId;

/// Returns default settings kept in a scratch file of this test run.
fn settings(tag: &str) -> Persistent<Settings> {
    let dir = std::env::temp_dir().join(format!("abstractions-feel-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    Persistent::<Settings>::builder()
        .name("settings")
        .format(StorageFormat::Toml)
        .path(dir.join("settings.toml"))
        .default(Settings::default())
        .build()
        .expect("settings in a scratch directory")
}

/// Returns an app that runs the feedback of blows, with the local robot holding a blade in the middle
/// of a swing, and its body.
fn app(tag: &str) -> (App, Entity) {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<Hit>()
        .add_message::<Missed>()
        .add_message::<Impact>()
        .add_message::<Fling>()
        .add_message::<Whoosh>()
        .init_resource::<Hitstop>()
        .init_resource::<Mark>()
        .insert_resource(settings(tag))
        .add_systems(
            Update,
            (stop::start, mark::note, spark::scatter, swish::swish),
        );
    let player = app.world_mut().spawn((Player, PlayerId::LOCAL)).id();
    let strike = Strike {
        act: Act::Swing(0.05),
        ..default()
    };
    let body = app
        .world_mut()
        .spawn((
            Avatar,
            blade(),
            strike,
            GlobalTransform::default(),
            ChildOf(player),
        ))
        .id();
    (app, body)
}

/// Returns how many messages of `T` are waiting, and takes them.
fn count<T: Message>(app: &mut App) -> usize {
    app.world_mut()
        .resource_mut::<Messages<T>>()
        .drain()
        .count()
}

/// Returns a clean blow of the blade of `body` on the animal `victim`, and the sound of it on hide.
fn blow(body: Entity, victim: Entity) -> (Hit, Impact) {
    let held = blade();
    let tip = Vec3::new(0.0, 0.0, -held.build.reach());
    let (energy, _) = landed(&held, [Vec3::ZERO, tip], tip);
    let hit = Hit {
        attacker: body,
        item: 0,
        victim,
        part: victim,
        at: tip,
        along: Vec3::X,
        blow: energy,
        intent: held.style.intent(),
        tip: Tip {
            stiffness: held.build.head.material.stiffness,
            mass: held.build.head.mass,
        },
        outcome: Outcome::Clean,
    };
    (hit, sound(tip, energy.energy, hit.tip))
}

/// Returns the sound of a blow of `joules` at `at` with the head `tip` on hide.
fn sound(at: Vec3, joules: f32, tip: Tip) -> Impact {
    Impact {
        at,
        joules,
        cut: false,
        outer: Outer::Hide,
        struck: Body {
            stiffness: 0.1,
            mass: 4.0,
        },
        tool: Body {
            stiffness: tip.stiffness,
            mass: tip.mass,
        },
        deflected: false,
    }
}

/// A hit of the local robot starts a hitstop with the animal frozen and the camera kicked, marks the
/// crosshair clean with the animal fought, and throws flecks off the hide where it landed.
#[test]
fn a_hit_is_felt_seen_and_marked() {
    let (mut app, body) = app("hit");
    let victim = app.world_mut().spawn_empty().id();
    let (hit, sound) = blow(body, victim);
    app.world_mut().write_message(hit);
    app.world_mut().write_message(sound);
    app.update();
    let stop = *app.world().resource::<Hitstop>();
    assert!(stop.left > 0.0 && stop.victim == Some(victim), "{stop:?}");
    assert!(stop.camera > 0.0 && stop.gain > 0.0);
    let mark = *app.world().resource::<Mark>();
    assert_eq!(mark.outcome, Some(Outcome::Clean));
    assert_eq!(mark.victim, Some(victim));
    assert!(mark.shown().is_some());
    assert!(count::<Fling>(&mut app) > 0);
    assert_eq!(count::<Whoosh>(&mut app), 1);
}

/// A swing that meets nothing whooshes as it starts and says it missed as it ends, and nothing else:
/// no hitstop, no mark, no flecks.
#[test]
fn a_miss_only_whooshes() {
    let (mut app, body) = app("miss");
    app.update();
    assert_eq!(count::<Whoosh>(&mut app), 1);
    app.world_mut()
        .get_mut::<Strike>(body)
        .expect("a strike")
        .act = Act::Recover(0.0);
    app.update();
    assert_eq!(count::<Missed>(&mut app), 1);
    assert_eq!(count::<Whoosh>(&mut app), 0);
    assert_eq!(*app.world().resource::<Hitstop>(), Hitstop::default());
    assert_eq!(app.world().resource::<Mark>().outcome, None);
    assert_eq!(count::<Fling>(&mut app), 0);
}

/// Hide sheds tufts and no sparks, shell under a hard head sparks, bone chips, flint throws dust and
/// sparks where soft soil throws dust alone, and flesh leaves its flecks to the blood.
#[test]
fn each_layer_throws_its_own_flecks() {
    let (_, sound) = blow(Entity::PLACEHOLDER, Entity::PLACEHOLDER);
    let kinds = |outer: Outer, stiffness: f32| {
        let struck = Body {
            stiffness,
            ..sound.struck
        };
        sprays(&Impact {
            outer,
            struck,
            ..sound
        })
    };
    let hide = kinds(Outer::Hide, 0.1);
    assert!(hide[0].is_some() && hide[1].is_none());
    let shell = kinds(Outer::Shell, 30.0);
    assert!(shell[1].is_some_and(|spark| spark.colour == SPARK));
    let bone = kinds(Outer::Bone, 17.7);
    assert!(bone.iter().all(Option::is_some));
    assert_eq!(kinds(Outer::Flesh, 0.01), [None, None]);
    let flint = kinds(Outer::Ground, 80.0);
    assert!(flint[0].is_some() && flint[1].is_some_and(|spark| spark.colour == SPARK));
    assert!(kinds(Outer::Ground, 0.05)[1].is_none());
}
