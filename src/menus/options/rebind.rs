//! Key rebinding: activate a row, then press a key.

use bevy::input::ButtonState;
use bevy::input::keyboard::KeyboardInput;
use bevy::input_focus::{FocusCause, InputFocus};
use bevy::prelude::*;

use super::field::{Bind, set_key};
use crate::menus::Menu;
use crate::menus::nav::MenuKeys;
use crate::settings::{Persistent, Settings};

/// The action waiting for its next key press, and the row that asked for it.
///
/// The row loses the focus meanwhile, so that Enter or Space can be bound without pressing it.
#[derive(Resource, Default)]
pub struct Rebinding(pub Option<(Bind, Entity)>);

impl Rebinding {
    /// Returns the action waiting for a key.
    pub fn bind(&self) -> Option<Bind> {
        self.0.map(|(bind, _)| bind)
    }
}

/// Run condition: true while a row waits for its next key.
pub fn capturing(rebinding: Res<Rebinding>) -> bool {
    rebinding.0.is_some()
}

/// Binds the next pressed key, or cancels on Escape, then hands the focus back to the row.
/// Keys of the frame that armed it are dropped.
fn capture_key(
    mut keys: MessageReader<KeyboardInput>,
    mut rebinding: ResMut<Rebinding>,
    mut settings: ResMut<Persistent<Settings>>,
    mut focus: ResMut<InputFocus>,
) {
    if rebinding.is_changed() {
        keys.clear();
        return;
    }
    for key in keys.read().filter(|key| key.state == ButtonState::Pressed) {
        let Some((bind, row)) = rebinding.0.take() else {
            return;
        };
        focus.set(row, FocusCause::Navigated);
        if key.key_code == KeyCode::Escape {
            return;
        }
        let code = key.key_code;
        if let Err(error) = settings.update(|s| set_key(s, bind, code)) {
            warn!("could not save settings: {error}");
        }
    }
}

/// Drops any pending rebind when the menu closes.
fn cancel(mut rebinding: ResMut<Rebinding>) {
    rebinding.0 = None;
}

/// Registers the rebinding systems.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Rebinding>()
        .add_systems(OnExit(Menu::Settings), cancel)
        .add_systems(
            Update,
            capture_key.after(MenuKeys).run_if(in_state(Menu::Settings)),
        );
}
