//! A headless run of the home base systems: found, built, spawned and despawned with no window.

use std::time::{Duration, Instant};

use avian3d::prelude::*;
use bevy::prelude::*;

use super::Base;
use super::glow::Glow;
use super::spawn::{Home, Lamp};
use super::tests::{homeworld, planet};
use crate::space::{Anchor, Frame};
use crate::universe::{BodyId, Universe};
use crate::world::SpawnPoint;

/// Seed of the world the app runs.
const SEED: u64 = 7;

/// Returns the number of entities that carry the component `C`.
fn count<C: Component>(world: &mut World) -> usize {
    world.query::<&C>().iter(world).count()
}

/// Runs frames until `done` holds. Fails after two minutes.
fn wait(app: &mut App, what: &str, done: impl Fn(&mut World) -> bool) {
    let start = Instant::now();
    while !done(app.world_mut()) {
        assert!(
            start.elapsed() < Duration::from_secs(120),
            "timed out waiting for {what}"
        );
        app.update();
        std::thread::sleep(Duration::from_millis(2));
    }
}

/// Returns an app with the home planet streamed, the base systems and no base yet.
fn world() -> App {
    let mut app = App::new();
    let universe = Universe::new(SEED);
    app.add_plugins(MinimalPlugins)
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<StandardMaterial>>()
        .insert_resource(planet(&universe, universe.home()))
        .insert_resource(universe);
    super::systems(&mut app);
    app
}

/// Anchors the render origin on the base centre and puts the focus `away` metres east of it.
fn focus(app: &mut App, away: f32) {
    let base = *app.world().resource::<Base>();
    app.insert_resource(Anchor {
        frame: Frame::Surface(base.body),
        origin: base.centre,
        rotation: base.rot().inverse(),
        flat: false,
    });
    app.insert_resource(SpawnPoint(Vec3::X * away));
    app.update();
    app.update();
}

/// Asserts that the spawned base is whole and sits at the render origin, level.
fn assert_whole(world: &mut World) {
    let mut roots = world.query::<(&Home, &Transform, &Collider, &Children)>();
    let (_, transform, _, children) = roots.single(world).expect("one base");
    assert!(transform.translation.length() < 1e-3);
    assert!(transform.rotation.angle_between(Quat::IDENTITY) < 1e-4);
    assert_eq!(children.len(), 1 + Glow::ALL.len() + 3);
    assert_eq!(count::<Lamp>(world), 3);
    assert_eq!(count::<PointLight>(world), 3);
    assert_eq!(count::<Glow>(world), Glow::ALL.len());
}

/// Streams another planet of the home system, then the home planet again.
fn leave_and_return(app: &mut App, home: BodyId) {
    let universe = *app.world().resource::<Universe>();
    let planets = universe.system(home.star).planets;
    let other = planets.into_iter().find(|p| p.id != home);
    app.insert_resource(planet(&universe, other.expect("a second planet").id));
    app.update();
    app.update();
    assert_eq!(
        count::<Home>(app.world_mut()),
        0,
        "the base left with its planet"
    );
    app.insert_resource(planet(&universe, home));
    app.update();
    app.update();
    assert_whole(app.world_mut());
}

/// Moves the focus far, back into the band between the two sight distances, then near.
fn go_far_and_return(app: &mut App) {
    focus(app, 5000.0);
    assert_eq!(
        count::<Home>(app.world_mut()),
        0,
        "the base left when the focus went far"
    );
    focus(app, 3300.0);
    assert_eq!(
        count::<Home>(app.world_mut()),
        0,
        "the base came back inside the hysteresis band"
    );
    focus(app, 2000.0);
    assert_eq!(count::<Home>(app.world_mut()), 1);
    let mut glows = app.world_mut().query::<(&Glow, &Transform)>();
    let beacon = glows
        .iter(app.world())
        .find(|(glow, _)| **glow == Glow::Beacon);
    assert!(beacon.expect("a beacon").1.scale.x > 10.0);
}

/// The base is found in the background, spawned near the focus on the home planet, lit, despawned when
/// another planet is streamed or the focus is far, and spawned again from its cached parts.
#[test]
fn base_follows_the_streamed_planet_and_the_focus() {
    let mut app = world();
    wait(&mut app, "the base to be found", |w| {
        w.contains_resource::<Base>()
    });
    let (globe, home) = homeworld(SEED);
    assert_eq!(*app.world().resource::<Base>(), Base::on(&globe, home));
    focus(&mut app, 3.0);
    wait(&mut app, "the base to spawn", |w| count::<Home>(w) == 1);
    (0..5).for_each(|_| app.update());
    assert_whole(app.world_mut());
    leave_and_return(&mut app, home);
    go_far_and_return(&mut app);
}
