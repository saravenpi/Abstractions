//! Universe time: the one clock that orbits, planet spin and saves read.

use bevy::prelude::*;

use crate::menus::Menu;
use crate::screens::Screen;

/// Seconds of universe time since the world was created.
///
/// Planet positions and rotations are pure functions of this value.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct Clock {
    /// Elapsed universe time in seconds.
    pub seconds: f64,
}

/// Moves universe time forward by the frame time while the game is being played.
fn advance(time: Res<Time>, mut clock: ResMut<Clock>) {
    clock.seconds += time.delta_secs_f64();
}

/// Run condition: the game is being played, with no menu open.
///
/// False when the screen states do not exist, as in a headless test app.
fn running(screen: Option<Res<State<Screen>>>, menu: Option<Res<State<Menu>>>) -> bool {
    let on_screen = screen.is_some_and(|screen| *screen.get() == Screen::Gameplay);
    on_screen && menu.is_some_and(|menu| *menu.get() == Menu::None)
}

/// Registers the universe clock.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Clock>()
        .add_systems(Update, advance.run_if(running));
}
