//! Tests of the key that hides the interface: the game image is drawn over every interface
//! layer only while it is hidden and the player controls the game.

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;

use super::{HIDE_KEY, HideHud, Screen, WarmUp};
use crate::menus::Menu;
use crate::ui::WorldView;

/// Returns a headless app in the world, in control, with the game image under the interface.
fn app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .init_resource::<ButtonInput<KeyCode>>()
        .init_state::<Screen>()
        .init_state::<WarmUp>()
        .init_state::<Menu>()
        .add_plugins(super::gameplay::plugin);
    app.world_mut().spawn((WorldView, GlobalZIndex(i32::MIN)));
    go(&mut app, Screen::Gameplay);
    app
}

/// Moves `app` to `state` and runs the two frames the change takes.
fn go<S: bevy::state::state::FreelyMutableState>(app: &mut App, state: S) {
    app.world_mut().resource_mut::<NextState<S>>().set(state);
    app.update();
    app.update();
}

/// Presses the hide key for one frame.
fn press(app: &mut App) {
    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.press(HIDE_KEY);
    app.update();
    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.release(HIDE_KEY);
    keys.clear();
    app.update();
}

/// Returns whether the game image is drawn over the interface, and whether the switch is set.
fn state(app: &mut App) -> (bool, bool) {
    let mut views = app
        .world_mut()
        .query_filtered::<&GlobalZIndex, With<WorldView>>();
    let over = views.single(app.world()).unwrap().0 == i32::MAX;
    (over, app.world().resource::<HideHud>().0)
}

/// The key hides every interface layer and shows them again.
#[test]
fn the_key_hides_and_shows_the_interface() {
    let mut app = app();
    assert_eq!(state(&mut app), (false, false));
    press(&mut app);
    assert_eq!(state(&mut app), (true, true));
    press(&mut app);
    assert_eq!(state(&mut app), (false, false));
}

/// A menu shows over the game even with the interface hidden, and closing it hides the
/// interface again. The key does nothing in a menu.
#[test]
fn a_menu_still_shows_while_the_interface_is_hidden() {
    let mut app = app();
    press(&mut app);
    go(&mut app, Menu::Pause);
    assert_eq!(state(&mut app), (false, true));
    press(&mut app);
    assert_eq!(state(&mut app), (false, true));
    go(&mut app, Menu::None);
    assert_eq!(state(&mut app), (true, true));
}

/// Leaving the world shows the interface again, so the next world starts with it.
#[test]
fn leaving_the_world_shows_the_interface_again() {
    let mut app = app();
    press(&mut app);
    go(&mut app, Screen::Title);
    assert_eq!(state(&mut app), (false, false));
}
