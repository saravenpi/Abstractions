//! Driving the rig: frames, and keys held like a player holds them.

use std::time::{Duration, Instant};

use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput, NativeKey};
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use super::Game;

/// Longest wall time a wait may take, in seconds: background tasks decide when it ends.
const PATIENCE: u64 = 60;

impl Game {
    /// Runs `frames` frames.
    pub fn step(&mut self, frames: usize) {
        for _ in 0..frames {
            self.app.update();
        }
    }

    /// Runs frames until `done` holds and returns how many it took. Fails after `limit` frames.
    pub fn until(&mut self, what: &str, limit: usize, done: impl Fn(&mut Self) -> bool) -> usize {
        for frame in 0..limit {
            if done(self) {
                return frame;
            }
            self.app.update();
        }
        panic!("{what}: not within {limit} frames");
    }

    /// Runs frames until `done` holds, giving background tasks wall time between frames. Use it
    /// only where the game itself waits for a task: loading, streaming, writing a file.
    pub fn wait(&mut self, what: &str, done: impl Fn(&mut Self) -> bool) {
        let start = Instant::now();
        while !done(self) {
            let late = start.elapsed() > Duration::from_secs(PATIENCE);
            assert!(!late, "{what}: not within {PATIENCE} s");
            self.app.update();
            std::thread::sleep(Duration::from_millis(2));
        }
    }

    /// Makes every following frame last `seconds` of game time.
    pub fn pace(&mut self, seconds: f64) {
        let step = TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(seconds));
        self.app.insert_resource(step);
    }

    /// Sends a key event, as the window would.
    fn key(&mut self, key_code: KeyCode, state: ButtonState) {
        self.app.world_mut().write_message(KeyboardInput {
            key_code,
            logical_key: Key::Unidentified(NativeKey::Unidentified),
            state,
            text: None,
            repeat: false,
            window: Entity::PLACEHOLDER,
        });
    }

    /// Presses `key` and holds it.
    pub fn press(&mut self, key: KeyCode) {
        self.key(key, ButtonState::Pressed);
    }

    /// Lets go of `key`.
    pub fn release(&mut self, key: KeyCode) {
        self.key(key, ButtonState::Released);
    }
}
