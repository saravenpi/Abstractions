//! Frame rate readout in the top-right corner, shown when the graphics setting is on.

use bevy::prelude::*;

use super::theme;
use crate::settings::{Persistent, Settings};

/// Seconds between two updates of the text.
const REFRESH: f32 = 0.25;

/// Weight of the newest frame time in the smoothed value.
const SMOOTHING: f32 = 0.1;

/// Marks the frame rate text and holds its smoothing state.
#[derive(Component, Default)]
pub(super) struct FpsText {
    frame: f32,
    since: f32,
}

/// Formats a frame time in seconds as "60 fps  16.6 ms".
fn readout(frame: f32) -> String {
    format!(
        "{:.0} fps  {:.1} ms",
        1.0 / frame.max(f32::EPSILON),
        frame * 1000.0
    )
}

/// Spawns the hidden text in the top-right corner.
pub(super) fn spawn(mut commands: Commands) {
    commands.spawn((
        FpsText::default(),
        Text::new(String::new()),
        TextFont::from_font_size(theme::FONT_SMALL),
        TextColor(theme::MUTED),
        Node {
            position_type: PositionType::Absolute,
            top: px(theme::GAP_S),
            right: px(theme::GAP_S),
            ..default()
        },
        Visibility::Hidden,
        Pickable::IGNORE,
        GlobalZIndex(100),
    ));
}

/// Smooths the frame time, refreshes the text a few times a second and follows the setting.
pub(super) fn update(
    time: Res<Time<Real>>,
    settings: Res<Persistent<Settings>>,
    mut text: Single<(&mut FpsText, &mut Text, &mut Visibility)>,
) {
    let (state, text, visibility) = &mut *text;
    let shown = match settings.graphics.show_fps {
        true => Visibility::Inherited,
        false => Visibility::Hidden,
    };
    if **visibility != shown {
        **visibility = shown;
    }
    let dt = time.delta_secs();
    if !settings.graphics.show_fps || dt <= 0.0 {
        return;
    }
    state.frame = match state.frame {
        0.0 => dt,
        f => f + (dt - f) * SMOOTHING,
    };
    state.since += dt;
    if state.since >= REFRESH {
        state.since = 0.0;
        text.0 = readout(state.frame);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 60 fps frame reads as 60 fps and 16.7 ms.
    #[test]
    fn readout_formats_rate_and_time() {
        assert_eq!(readout(1.0 / 60.0), "60 fps  16.7 ms");
    }
}
