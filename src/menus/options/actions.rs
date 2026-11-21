//! Button handling for the settings menu.

use bevy::ecs::system::SystemParam;
use bevy::input_focus::InputFocus;
use bevy::prelude::*;
use bevy::ui_widgets::Activate;

use super::field::{Bind, Field, step};
use super::rebind::{Rebinding, capturing};
use super::{SettingsTab, Tab};
use crate::menus::Menu;
use crate::settings::{Keymap, Persistent, Settings};

/// What a settings button does when activated.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub enum SettingsAction {
    Adjust(Field, i32),
    Rebind(Bind),
    ResetKeys,
    Tab(Tab),
}

/// State the settings buttons read and write.
#[derive(SystemParam)]
struct ActionCtx<'w> {
    settings: ResMut<'w, Persistent<Settings>>,
    rebinding: ResMut<'w, Rebinding>,
    tab: ResMut<'w, SettingsTab>,
    focus: ResMut<'w, InputFocus>,
}

/// Applies a settings button press and saves the result.
fn on_activate(event: On<Activate>, actions: Query<&SettingsAction>, mut ctx: ActionCtx) {
    let Ok(action) = actions.get(event.entity) else {
        return;
    };
    match *action {
        SettingsAction::Adjust(field, dir) => {
            if let Err(error) = ctx.settings.update(|s| step(s, field, dir)) {
                warn!("could not save settings: {error}");
            }
        }
        SettingsAction::Rebind(bind) => {
            ctx.rebinding.0 = Some((bind, event.entity));
            ctx.focus.clear();
        }
        SettingsAction::ResetKeys => {
            if let Err(error) = ctx.settings.update(|s| s.controls.keys = Keymap::default()) {
                warn!("could not save settings: {error}");
            }
        }
        SettingsAction::Tab(tab) => ctx.tab.0 = tab,
    }
}

/// Steps the focused row with Left and Right or A and D.
fn adjust_with_keys(
    keys: Res<ButtonInput<KeyCode>>,
    focus: Res<InputFocus>,
    actions: Query<&SettingsAction>,
    mut settings: ResMut<Persistent<Settings>>,
) {
    let dir = i32::from(keys.any_just_pressed([KeyCode::ArrowRight, KeyCode::KeyD]))
        - i32::from(keys.any_just_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]));
    let Some(SettingsAction::Adjust(field, _)) = focus.get().and_then(|e| actions.get(e).ok())
    else {
        return;
    };
    if dir != 0 {
        let field = *field;
        if let Err(error) = settings.update(|s| step(s, field, dir)) {
            warn!("could not save settings: {error}");
        }
    }
}

/// Registers the settings button observer and key stepping.
pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_activate).add_systems(
        Update,
        adjust_with_keys
            .run_if(in_state(Menu::Settings))
            .run_if(not(capturing)),
    );
}
