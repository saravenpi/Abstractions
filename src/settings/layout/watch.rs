//! Learning the labels of the keys from the key events of the session.

use bevy::input::ButtonState;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;

use super::KeyLabels;

/// The keys that change what a key prints while they are held. A press made with one of them
/// down teaches nothing.
const MODIFIERS: [KeyCode; 8] = [
    KeyCode::ShiftLeft,
    KeyCode::ShiftRight,
    KeyCode::ControlLeft,
    KeyCode::ControlRight,
    KeyCode::AltLeft,
    KeyCode::AltRight,
    KeyCode::SuperLeft,
    KeyCode::SuperRight,
];

/// Learns from the keys pressed this frame with no modifier held. The labels count as changed
/// only when one did change, so the texts that show keys are written again only then.
fn learn(
    mut pressed: MessageReader<KeyboardInput>,
    held: Res<ButtonInput<KeyCode>>,
    mut labels: ResMut<KeyLabels>,
) {
    let modified = held.any_pressed(MODIFIERS);
    let mut changed = false;
    for key in pressed.read() {
        if key.state == ButtonState::Pressed && !key.repeat && !modified {
            changed |= labels
                .bypass_change_detection()
                .learn(key.key_code, &key.logical_key);
        }
    }
    if changed {
        labels.set_changed();
    }
}

/// Registers the labels and the system that learns them. It brings the key messages and the key
/// state it reads, so an app without the input plugin still runs it.
pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<KeyLabels>()
        .init_resource::<ButtonInput<KeyCode>>()
        .add_message::<KeyboardInput>()
        .add_systems(Update, learn.run_if(on_message::<KeyboardInput>));
}
