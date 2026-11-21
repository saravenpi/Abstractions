//! Pure editing of individual settings.

use bevy::prelude::KeyCode;

pub use super::binds::Bind;
use super::lights;
use crate::settings::{Graphics, Settings, render_step};
use crate::view::ViewPrefs;
use crate::world::RENDER_STEPS;

/// One editable setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    RenderDistance,
    LightingQuality,
    Shadows,
    ShadowDetail,
    ShadowDistance,
    CloudShadows,
    AutoExposure,
    WhiteBalance,
    LocalShadows,
    WaterGlint,
    Vsync,
    Fullscreen,
    AmbientOcclusion,
    Bloom,
    Clouds,
    ShowFps,
    Fov,
    Sensitivity,
    InvertY,
    Master,
    Music,
    Effects,
    ViewFoot,
    ViewShip,
    HeadBob,
    AutoFollow,
    SpeedFx,
    CheckUpdates,
    Hitstop,
    WorldMarkers,
    Blood,
    Interface,
    Key(Bind),
}

/// Moves a whole number by `dir` inside `lo..=hi`.
fn bump_int(value: u32, dir: i32, lo: u32, hi: u32) -> u32 {
    value.saturating_add_signed(dir).clamp(lo, hi)
}

/// Moves a render distance to the named step `dir` steps away, staying on the first and last.
fn bump_step(rings: u32, dir: i32) -> u32 {
    let last = RENDER_STEPS.len() as i32 - 1;
    let at = (render_step(rings) as i32 + dir).clamp(0, last);
    RENDER_STEPS[at as usize].1
}

/// Moves a value by `dir` tenths inside `lo..=hi`.
fn bump_tenths(value: f32, dir: i32, lo: f32, hi: f32) -> f32 {
    (((value * 10.0).round() + dir as f32) / 10.0).clamp(lo, hi)
}

/// Returns the on/off flag a graphics field edits, if it is a toggle.
fn flag(graphics: &mut Graphics, field: Field) -> Option<&mut bool> {
    match field {
        Field::Shadows => Some(&mut graphics.shadows),
        Field::Vsync => Some(&mut graphics.vsync),
        Field::Fullscreen => Some(&mut graphics.fullscreen),
        Field::AmbientOcclusion => Some(&mut graphics.ambient_occlusion),
        Field::Bloom => Some(&mut graphics.bloom),
        Field::Clouds => Some(&mut graphics.clouds),
        Field::ShowFps => Some(&mut graphics.show_fps),
        _ => None,
    }
}

/// Steps a graphics setting by `dir`. Toggles flip and numbers move and clamp.
fn step_graphics(g: &mut Graphics, field: Field, dir: i32) {
    if let Some(flag) = flag(g, field) {
        *flag = !*flag;
        return;
    }
    match field {
        Field::RenderDistance => g.render_distance = bump_step(g.render_distance, dir),
        Field::Fov => g.fov = bump_int(g.fov, dir * 5, 60, 110),
        _ => lights::step(g, field, dir),
    }
}

/// Steps a camera setting: the two views switch between first and third person, the comfort
/// options flip.
fn step_view(view: &mut ViewPrefs, field: Field) {
    match field {
        Field::ViewFoot => view.on_foot = view.on_foot.toggled(),
        Field::ViewShip => view.ship = view.ship.toggled(),
        Field::HeadBob => view.head_bob = !view.head_bob,
        Field::AutoFollow => view.auto_follow = !view.auto_follow,
        _ => {}
    }
}

/// Steps a setting by `dir`. Toggles flip, numbers move and clamp, keys are untouched.
pub fn step(settings: &mut Settings, field: Field, dir: i32) {
    let (c, a, updates, play) = (
        &mut settings.controls,
        &mut settings.audio,
        &mut settings.updates,
        &mut settings.gameplay,
    );
    step_graphics(&mut settings.graphics, field, dir);
    step_view(&mut settings.view, field);
    match field {
        Field::Sensitivity => c.mouse_sensitivity = bump_tenths(c.mouse_sensitivity, dir, 0.1, 3.0),
        Field::InvertY => c.invert_y = !c.invert_y,
        Field::Master => a.master = bump_tenths(a.master, dir, 0.0, 1.0),
        Field::Music => a.music = bump_tenths(a.music, dir, 0.0, 1.0),
        Field::Effects => a.effects = bump_tenths(a.effects, dir, 0.0, 1.0),
        Field::CheckUpdates => updates.check_at_launch = !updates.check_at_launch,
        Field::WorldMarkers => play.world_markers = !play.world_markers,
        Field::Blood => play.blood = !play.blood,
        Field::Interface => play.interface = play.interface.moved(dir),
        Field::Hitstop => play.hitstop = bump_tenths(play.hitstop, dir, 0.0, 1.0),
        Field::SpeedFx => {
            settings.view.speed_fx = bump_tenths(settings.view.speed_fx, dir, 0.0, 1.0);
        }
        _ => {}
    }
}

/// Assigns a key to an action.
pub fn set_key(settings: &mut Settings, bind: Bind, key: KeyCode) {
    *bind.slot(&mut settings.controls.keys) = key;
}
