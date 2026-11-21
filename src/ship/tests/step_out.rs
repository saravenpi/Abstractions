//! Stepping out of the landed ship: the press acts on its own frame at any frame rate, leaving
//! is never offered while the ground beside the hatch has no collider, and the feet go down on
//! the collider.

use avian3d::prelude::*;
use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;

use super::leaving::boards;
use super::rig::{FRAME, Game};
use crate::player::{Interact, Player, Verb};
use crate::settings::{Persistent, Settings};
use crate::ship::{Hatch, Ship};
use crate::space::{Anchor, Location, Travel, up};

/// Returns the interact key.
fn key(game: &Game) -> KeyCode {
    let settings = game.app.world().resource::<Persistent<Settings>>();
    settings.controls.keys.ship.board
}

/// Returns true while the prompt names leaving the ship.
fn offered(game: &mut Game) -> bool {
    game.app.world().resource::<Interact>().is(Verb::Leave)
}

/// Returns the metres from the feet of the body down to the collider under them, the ship and
/// the body left out, or nothing when no collider lies within 10 m.
fn gap(game: &mut Game) -> Option<f32> {
    let (feet, anchor) = (game.walker().feet, game.get::<Anchor>());
    let vertical = up(&anchor, feet);
    let probe = move |query: SpatialQuery,
                      movers: Query<Entity, Or<(With<Ship>, With<Player>)>>| {
        let filter = SpatialQueryFilter::from_excluded_entities(movers.iter());
        let down = Dir3::new(-vertical).expect("a vertical");
        let hit = query.cast_ray(feet + vertical * 2.0, down, 12.0, true, &filter);
        hit.map(|hit| hit.distance - 2.0)
    };
    game.app
        .world_mut()
        .run_system_once(probe)
        .expect("the probe runs")
}

/// Presses the interact key for one frame and lets go of it after.
fn tap(game: &mut Game) {
    let key = key(game);
    game.press(key);
    game.step(1);
    game.release(key);
}

/// Stands the player beside the ship, then for 20, 30 and 60 frames a second and a press 0, 1
/// or 2 frames after leaving is offered: the press leaves on its own frame, and the body stands
/// on the collider beside the hatch.
#[test]
fn leaving_acts_on_the_frame_of_the_press_at_any_frame_rate() {
    let mut game = Game::full("step-out-fps");
    game.start("StepOutFps", None);
    game.until("the player to stand", 300, |game| game.walker().grounded);
    for fps in [20.0, 30.0, 60.0] {
        for late in 0..3 {
            boards(&mut game);
            game.pace(1.0 / fps);
            game.wait("leaving to be offered", offered);
            game.step(late);
            assert!(offered(&mut game), "{fps} fps");
            tap(&mut game);
            assert_eq!(
                game.get::<Location>().travel,
                Travel::OnFoot,
                "{fps} fps, {late} frames late"
            );
            game.step(1);
            assert!(!game.walker().seated, "{fps} fps");
            let gap = gap(&mut game).expect("a collider under the feet");
            assert!(
                (-0.05..0.3).contains(&gap),
                "{fps} fps: feet {gap} m over the ground"
            );
            game.pace(FRAME);
            game.until("the player to stand", 300, |game| game.walker().grounded);
        }
    }
}

/// Returns every collider of a static body.
fn statics(colliders: Query<(Entity, &ColliderOf)>, bodies: Query<&RigidBody>) -> Vec<Entity> {
    let fixed = |of: &ColliderOf| bodies.get(of.body).is_ok_and(RigidBody::is_static);
    let found = colliders.iter().filter(|(_, of)| fixed(of));
    found.map(|(entity, _)| entity).collect()
}

/// Switches off every collider of a static body, and returns them.
fn loosen(game: &mut Game) -> Vec<Entity> {
    let found = game
        .app
        .world_mut()
        .run_system_once(statics)
        .expect("the search runs");
    for entity in &found {
        game.app
            .world_mut()
            .entity_mut(*entity)
            .insert(ColliderDisabled);
    }
    found
}

/// While the ground beside the hatch has no collider, leaving is not offered, the hatch says
/// the pilot waits, and the key keeps the pilot seated. Once the collider is back, leaving is
/// offered and the press steps out onto it.
#[test]
fn the_pilot_stays_seated_until_the_ground_has_a_collider() {
    let mut game = Game::full("step-out-wait");
    game.start("StepOutWait", None);
    game.until("the player to stand", 300, |game| game.walker().grounded);
    boards(&mut game);
    game.wait("leaving to be offered", offered);
    let loose = loosen(&mut game);
    game.step(2);
    for _ in 0..30 {
        assert!(!offered(&mut game) && game.get::<Hatch>().waiting);
        tap(&mut game);
        assert_eq!(game.get::<Location>().travel, Travel::InShip);
        assert!(game.walker().seated);
    }
    for entity in &loose {
        game.app
            .world_mut()
            .entity_mut(*entity)
            .remove::<ColliderDisabled>();
    }
    game.wait("leaving to be offered", offered);
    assert!(!game.get::<Hatch>().waiting);
    tap(&mut game);
    assert_eq!(game.get::<Location>().travel, Travel::OnFoot);
    game.step(1);
    let gap = gap(&mut game).expect("a collider under the feet");
    assert!((-0.05..0.3).contains(&gap), "feet {gap} m over the ground");
}
