//! Telling the player how far opening a world with an older release has gone.

use bevy::prelude::*;

use super::Notice;
use crate::update::{Older, Updates};

/// Writes the state of an older release's opening under the table, while one is under way or
/// has ended.
pub(super) fn report(updates: Res<Updates>, mut notice: ResMut<Notice>) {
    if !updates.is_changed() {
        return;
    }
    let text = match updates.older() {
        Older::Idle => return,
        Older::Working => "Opening with the older release, downloading it first if needed".into(),
        Older::Opened => "Opened with the older release".to_string(),
        Older::Failed(reason) => format!("Could not open it: {reason}"),
    };
    notice.set_if_neq(Notice(text));
}
