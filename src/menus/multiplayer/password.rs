//! The password prompt: after a server refused the password, the menu asks for one and joins
//! the same server again with it.

use bevy::prelude::*;

use super::{Form, FormKind, Mode};
use crate::net::PasswordNeeded;

/// The server that refused the last password, waiting for the player to type one: its address
/// and the key its beacon announced.
#[derive(Resource, Debug, Default, Clone, PartialEq, Eq)]
pub(super) struct Retry(pub Option<(String, Option<[u8; 32]>)>);

/// Keeps the server that asked for a password, whichever menu is open.
pub(super) fn remember(mut needs: MessageReader<PasswordNeeded>, mut retry: ResMut<Retry>) {
    if let Some(need) = needs.read().last() {
        retry.0 = Some((need.address.clone(), need.expected_key));
    }
}

/// Opens the password form once the list is back while a server waits for a password.
pub(super) fn ask(retry: Res<Retry>, mut mode: ResMut<Mode>) {
    if retry.0.is_some() && *mode == Mode::Browse {
        *mode = Mode::Form(Form::new(FormKind::Password, String::new(), String::new()));
    }
}
