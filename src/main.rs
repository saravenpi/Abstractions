//! Abstractions game binary.

use bevy::prelude::*;

/// Starts the game.
fn main() -> AppExit {
    App::new().add_plugins(abstractions::AppPlugin).run()
}
