//! Button handling for the settings menu.

use bevy::ecs::system::SystemParam;
use bevy::input_focus::InputFocus;
use bevy::prelude::*;
use bevy::ui_widgets::Activate;

use super::field::{Bind, Field, step};
use super::rebind::Rebinding;
use super::{SettingsTab, Tab};
use crate::settings::{Persistent, Settings};

/// What a settings button does when activated.
#[derive(Component, Debug, Clone, Copy)]
pub enum SettingsAction {
    Adjust(Field, i32),
    Rebind(Bind),
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
            ctx.rebinding.0 = Some(bind);
            ctx.focus.clear();
        }
        SettingsAction::Tab(tab) => ctx.tab.0 = tab,
    }
}

/// Registers the settings button observer.
pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_activate);
}
