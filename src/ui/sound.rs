//! Interface sounds, sent once for every menu and screen: move, confirm and cancel.

use bevy::input_focus::InputFocus;
use bevy::prelude::*;
use bevy::ui_widgets::Activate;

use super::widgets::Overlay;
use crate::audio::AudioCue;

/// Marks a button whose activation backs out: it sounds like a cancel instead of a confirm.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Cancels;

/// Returns the cue for a focus that went from `last` to `now`. Only a move between two buttons
/// that both exist is heard, so opening a screen is silent.
pub fn move_cue(last: Option<Entity>, now: Option<Entity>, last_alive: bool) -> Option<AudioCue> {
    (now.is_some() && last.is_some() && last != now && last_alive).then_some(AudioCue::UiMove)
}

/// Sounds a move when the keyboard focus goes from one button to another.
pub(super) fn moved(
    focus: Res<InputFocus>,
    alive: Query<Entity>,
    mut last: Local<Option<Entity>>,
    mut cues: MessageWriter<AudioCue>,
) {
    let now = focus.get();
    let last_alive = last.is_some_and(|entity| alive.contains(entity));
    if let Some(cue) = move_cue(*last, now, last_alive) {
        cues.write(cue);
    }
    *last = now;
}

/// Sounds a confirm when any button is activated, or a cancel when the button backs out.
pub(super) fn activated(
    event: On<Activate>,
    cancels: Query<(), With<Cancels>>,
    mut cues: MessageWriter<AudioCue>,
) {
    cues.write(match cancels.contains(event.entity) {
        true => AudioCue::UiCancel,
        false => AudioCue::UiConfirm,
    });
}

/// Sounds a cancel when Escape is pressed while a menu or a screen is open.
pub(super) fn escaped(
    keys: Res<ButtonInput<KeyCode>>,
    overlays: Query<(), With<Overlay>>,
    mut cues: MessageWriter<AudioCue>,
) {
    if keys.just_pressed(KeyCode::Escape) && !overlays.is_empty() {
        cues.write(AudioCue::UiCancel);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A move is heard between two live buttons only.
    #[test]
    fn only_a_move_between_live_buttons_is_heard() {
        let (a, b) = (
            Entity::from_raw_u32(1).unwrap(),
            Entity::from_raw_u32(2).unwrap(),
        );
        assert_eq!(move_cue(Some(a), Some(b), true), Some(AudioCue::UiMove));
        assert_eq!(move_cue(Some(a), Some(a), true), None);
        assert_eq!(move_cue(None, Some(b), false), None);
        assert_eq!(move_cue(Some(a), Some(b), false), None);
        assert_eq!(move_cue(Some(a), None, true), None);
    }
}
