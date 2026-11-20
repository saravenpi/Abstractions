//! Universe time: the one clock that orbits, planet spin and saves read.

use bevy::prelude::*;

use crate::menus::Menu;
use crate::net::{Session, in_session};
use crate::screens::Screen;

/// Seconds of universe time since the world was created.
///
/// Planet positions and rotations are pure functions of this value.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct Clock {
    /// Elapsed universe time in seconds.
    pub seconds: f64,
}

/// Seconds after which [`Clock::phase`] starts over: a power of two, so that a phase in `f32` keeps
/// better than a hundredth of a second.
pub const PHASE_PERIOD: f64 = 65_536.0;

impl Clock {
    /// Returns universe time as an `f32` that starts over every [`PHASE_PERIOD`] seconds. What two
    /// players see move with the weather, the waves and the foam reads it, so that it is the same on
    /// every screen of a session.
    pub fn phase(&self) -> f32 {
        (self.seconds % PHASE_PERIOD) as f32
    }
}

/// The step of `PreUpdate` that moves universe time. What corrects it, such as the clock of a
/// session, runs after it.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClockStep;

/// Moves universe time forward by the frame time while the game is being played.
fn advance(time: Res<Time>, mut clock: ResMut<Clock>) {
    clock.seconds += time.delta_secs_f64();
}

/// Run condition: the game is being played, with no menu open. In a session the universe never
/// pauses, so a menu open there keeps the clock running.
///
/// False when the screen states do not exist, as in a headless test app.
fn running(
    screen: Option<Res<State<Screen>>>,
    menu: Option<Res<State<Menu>>>,
    session: Option<Res<Session>>,
) -> bool {
    let on_screen = screen.is_some_and(|screen| *screen.get() == Screen::Gameplay);
    let closed = menu.is_some_and(|menu| *menu.get() == Menu::None);
    on_screen && (closed || in_session(session))
}

/// Registers the universe clock. It moves before `Update`, so that every system of a frame
/// reads the same time.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Clock>()
        .add_systems(PreUpdate, advance.run_if(running).in_set(ClockStep));
}
