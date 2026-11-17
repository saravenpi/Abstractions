//! The board computer, the HUD and the boot screen inside the app.

use bevy::prelude::*;

use super::{app, tap};
use crate::computer::{Atlas, Computer};
use crate::menus::Menu;
use crate::screens::{Screen, WarmUp};
use crate::space::{Anchor, Clock, Location, Travel};
use crate::universe::Universe;
use crate::world::{Terrain, WorldReady};

/// Puts a world in the app with the player seated in the ship, and starts playing.
fn seated(tag: &str) -> App {
    let mut app = app(tag);
    let (universe, terrain) = (Universe::new(7), Terrain::new(7));
    app.insert_resource(Anchor::surface(universe.home(), &terrain, Vec3::ZERO))
        .insert_resource(Location {
            body: universe.home(),
            travel: Travel::InShip,
        })
        .insert_resource(Atlas::new(&universe))
        .insert_resource(Clock { seconds: 0.0 })
        .insert_resource(universe);
    app.world_mut()
        .resource_mut::<NextState<Screen>>()
        .set(Screen::Gameplay);
    app.world_mut()
        .resource_mut::<NextState<Menu>>()
        .set(Menu::None);
    for _ in 0..3 {
        app.update();
    }
    app
}

/// Returns the state of the computer screen and the open menu.
fn states(app: &App) -> (Computer, Menu) {
    let computer = *app.world().resource::<State<Computer>>().get();
    (computer, *app.world().resource::<State<Menu>>().get())
}

/// The computer opens on its key, runs every tab, and Escape closes it without pausing the
/// game. The next Escape pauses.
#[test]
fn the_computer_opens_and_closes_without_pausing() {
    let mut app = seated("computer");
    tap(&mut app, KeyCode::Tab);
    assert_eq!(states(&app), (Computer::Open, Menu::None));
    for _ in 0..40 {
        app.update();
    }
    for key in [
        KeyCode::ArrowDown,
        KeyCode::ArrowRight,
        KeyCode::ArrowRight,
        KeyCode::End,
    ] {
        tap(&mut app, key);
    }
    tap(&mut app, KeyCode::Escape);
    app.update();
    assert_eq!(states(&app), (Computer::Closed, Menu::None));
    tap(&mut app, KeyCode::Tab);
    tap(&mut app, KeyCode::Tab);
    assert_eq!(states(&app), (Computer::Closed, Menu::None));
    tap(&mut app, KeyCode::Escape);
    assert_eq!(states(&app), (Computer::Closed, Menu::Pause));
}

/// Covers the world with the boot screen and runs `frames` frames.
fn boot(app: &mut App, frames: usize) -> WarmUp {
    app.world_mut()
        .resource_mut::<NextState<WarmUp>>()
        .set(WarmUp::Hidden);
    for _ in 0..frames {
        app.update();
    }
    *app.world().resource::<State<WarmUp>>().get()
}

/// The boot screen hides the world for more than half a second and ends on its own.
#[test]
fn the_boot_screen_ends_on_its_own() {
    let mut app = seated("boot");
    assert_eq!(boot(&mut app, 40), WarmUp::Hidden);
    for _ in 0..120 {
        app.update();
    }
    assert_eq!(*app.world().resource::<State<WarmUp>>().get(), WarmUp::Idle);
}

/// Any key skips the boot screen, and Escape skips it without pausing the game.
#[test]
fn the_boot_screen_can_be_skipped() {
    let mut app = seated("skip");
    assert_eq!(boot(&mut app, 2), WarmUp::Hidden);
    tap(&mut app, KeyCode::Escape);
    assert_eq!(states(&app).1, Menu::None);
    for _ in 0..55 {
        app.update();
    }
    assert_eq!(*app.world().resource::<State<WarmUp>>().get(), WarmUp::Idle);
}

/// A loaded world goes from the loading screen to gameplay behind the boot screen, then the
/// boot screen hands the world over and the HUD runs.
#[test]
fn a_world_loads_behind_the_boot_screen() {
    let mut app = seated("load");
    app.world_mut()
        .resource_mut::<NextState<Screen>>()
        .set(Screen::Loading);
    for _ in 0..3 {
        app.update();
    }
    assert_eq!(
        *app.world().resource::<State<Screen>>().get(),
        Screen::Loading
    );
    app.insert_resource(WorldReady(true));
    app.update();
    app.update();
    let warm = *app.world().resource::<State<WarmUp>>().get();
    let screen = *app.world().resource::<State<Screen>>().get();
    assert_eq!((screen, warm), (Screen::Gameplay, WarmUp::Hidden));
    for _ in 0..160 {
        app.update();
    }
    let warm = *app.world().resource::<State<WarmUp>>().get();
    assert_eq!((warm, states(&app).1), (WarmUp::Idle, Menu::None));
}
