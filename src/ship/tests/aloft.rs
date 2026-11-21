//! The flight gear in a whole game on foot: a jump, the boots, the wingsuit, powered flight, a
//! fold in mid-air, a shallow landing into a run, a glide into the sea, and the third person
//! view on the ground. The camera, the body and the companion are watched on every frame.

mod eye;
mod ground;
mod loops;
mod stages;
mod watch;

use bevy::prelude::*;

use super::rig::Game;
use crate::gear::{Gear, Mode};
use crate::player::{Motion, Player, place};
use crate::settings::{Keymap, Persistent, Settings};
use crate::space::{Anchor, altitude, up};
use crate::view::{ViewMode, ViewPrefs};
use crate::world::Terrain;
use watch::Watch;

/// The body as the stages read it.
#[derive(Debug, Clone, Copy)]
pub struct Body {
    /// Render position of the centre.
    pub at: Vec3,
    /// Movement state of the controller.
    pub motion: Motion,
    /// Flight gear.
    pub gear: Gear,
    /// Height of the centre above the sea, in metres.
    pub height: f32,
}

impl Game {
    /// Returns the key the settings bind to what `pick` names.
    pub fn bound(&self, pick: impl Fn(&Keymap) -> KeyCode) -> KeyCode {
        pick(
            &self
                .app
                .world()
                .resource::<Persistent<Settings>>()
                .controls
                .keys,
        )
    }

    /// Returns the player body.
    pub fn body(&mut self) -> Body {
        let (anchor, terrain) = (self.get::<Anchor>(), self.get::<Terrain>());
        let world = self.app.world_mut();
        let mut bodies = world.query_filtered::<(&Transform, &Motion, &Gear), With<Player>>();
        let (transform, motion, gear) = bodies.single(world).expect("one player");
        Body {
            at: transform.translation,
            motion: *motion,
            gear: *gear,
            height: altitude(&anchor, terrain.globe(), transform.translation),
        }
    }

    /// Puts the body `lift` metres above where it stands now, at rest, facing `toward`.
    pub fn lift(&mut self, lift: f32, toward: Vec3) {
        let at = self.body().at;
        let vertical = up(&self.get::<Anchor>(), at);
        place(self.app.world_mut(), at + vertical * lift, toward);
    }

    /// Changes the body and its gear in place.
    pub fn with_body(&mut self, change: impl FnOnce(&mut Motion, &mut Gear)) {
        let world = self.app.world_mut();
        let mut bodies = world.query_filtered::<(&mut Motion, &mut Gear), With<Player>>();
        let (mut motion, mut gear) = bodies.single_mut(world).expect("one player");
        change(&mut motion, &mut gear);
    }

    /// Taps `key`: pressed for one frame, then let go.
    pub fn tap(&mut self, key: KeyCode, watch: &mut Watch) {
        self.press(key);
        watch.step(self, 1);
        self.release(key);
    }

    /// Runs frames under `watch` until the gear is in `mode`. Fails after `limit` frames.
    pub fn until_mode(&mut self, mode: Mode, limit: usize, watch: &mut Watch) {
        for _ in 0..limit {
            if self.body().gear.state.mode == mode {
                return;
            }
            watch.step(self, 1);
        }
        panic!(
            "the gear never went into {mode:?}: {:?}",
            self.body().gear.state.mode
        );
    }
}

/// On foot from a new game: a jump, then the boots, the wingsuit, powered flight, a fold and the
/// boots again high above the ground, a shallow landing into a run, a glide into the sea, and a
/// diagonal walk in third person. The view is switched and boarding is tried along the way.
#[test]
fn the_gear_flies_and_hands_back_to_walking_and_swimming() {
    let mut game = Game::full("aloft");
    let game = &mut game;
    game.start("Aloft", None);
    game.until("the player to stand", 300, |game| game.walker().grounded);
    game.app.world_mut().resource_mut::<ViewPrefs>().on_foot = ViewMode::First;
    let mut watch = Watch::default();
    let feet = ground::jumps(game, &mut watch);
    stages::boots(game, &mut watch);
    stages::glides(game, &mut watch);
    stages::flies(game, &mut watch);
    stages::folds(game, &mut watch);
    ground::lands_running(game, &mut watch, feet);
    ground::walks_third_person(game, &mut watch);
    ground::glides_into_the_sea(game, &mut watch);
    watch.report();
}
