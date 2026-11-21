//! Display text of individual settings.

use super::field::Field;
use super::lights;
use crate::settings::{Graphics, KeyLabels, Settings, render_step};
use crate::view::{ViewMode, ViewPrefs};
use crate::world::RENDER_STEPS;

/// Formats a boolean as On or Off.
pub fn on_off(value: bool) -> String {
    if value { "On" } else { "Off" }.to_string()
}

/// Formats a strength from 0 to 1 in tenths, Off at 0.
pub fn strength(value: f32) -> String {
    match value > 0.0 {
        true => format!("{value:.1}"),
        false => "Off".to_string(),
    }
}

/// Returns the value of a graphics toggle, if the field is one.
fn flag_value(g: &Graphics, field: Field) -> Option<bool> {
    match field {
        Field::Shadows => Some(g.shadows),
        Field::Vsync => Some(g.vsync),
        Field::Fullscreen => Some(g.fullscreen),
        Field::AmbientOcclusion => Some(g.ambient_occlusion),
        Field::Bloom => Some(g.bloom),
        Field::Clouds => Some(g.clouds),
        Field::ShowFps => Some(g.show_fps),
        _ => None,
    }
}

/// Formats a graphics setting, or `None` when the field is not a graphics one.
fn display_graphics(g: &Graphics, field: Field) -> Option<String> {
    if let Some(value) = flag_value(g, field) {
        return Some(on_off(value));
    }
    match field {
        Field::RenderDistance => Some(RENDER_STEPS[render_step(g.render_distance)].0.to_string()),
        Field::Fov => Some(format!("{} deg", g.fov)),
        _ => lights::display(g, field),
    }
}

/// Returns the name of a view mode.
pub fn view_name(mode: ViewMode) -> &'static str {
    match mode {
        ViewMode::First => "First person",
        ViewMode::Third => "Third person",
    }
}

/// Formats a camera setting, or `None` when the field is not a camera one.
fn display_view(view: &ViewPrefs, field: Field) -> Option<String> {
    match field {
        Field::ViewFoot => Some(view_name(view.on_foot).to_string()),
        Field::ViewShip => Some(view_name(view.ship).to_string()),
        Field::HeadBob => Some(on_off(view.head_bob)),
        Field::AutoFollow => Some(on_off(view.auto_follow)),
        Field::SpeedFx => Some(strength(view.speed_fx)),
        _ => None,
    }
}

/// Formats the current value of a setting for display. A key is named as `labels` prints it.
pub fn display(settings: &Settings, labels: &KeyLabels, field: Field) -> String {
    let (c, a) = (&settings.controls, &settings.audio);
    let known = display_graphics(&settings.graphics, field);
    if let Some(shown) = known.or_else(|| display_view(&settings.view, field)) {
        return shown;
    }
    match field {
        Field::Sensitivity => format!("{:.1}", c.mouse_sensitivity),
        Field::InvertY => on_off(c.invert_y),
        Field::Master => format!("{:.1}", a.master),
        Field::Music => format!("{:.1}", a.music),
        Field::Effects => format!("{:.1}", a.effects),
        Field::CheckUpdates => on_off(settings.updates.check_at_launch),
        Field::WorldMarkers => on_off(settings.gameplay.world_markers),
        Field::Blood => on_off(settings.gameplay.blood),
        Field::Hitstop => strength(settings.gameplay.hitstop),
        Field::Key(bind) => labels.label(bind.key(&c.keys)),
        _ => String::new(),
    }
}
