//! The fade between two levels of the map: a veil of the panel colour over the map that clears
//! in a moment whenever the level or the chart changes, so one level fades into the next.

use bevy::prelude::*;

use super::level::Level;
use crate::computer::{Board, Tab, Target};
use crate::ui::{motion, theme};

/// Seconds the veil takes to clear.
pub const FADE: f32 = motion::LEVEL_FADE;

/// Marks the veil over the map.
#[derive(Component)]
pub struct Veil;

/// The veil, clear, over the whole room of the map. It lets the pointer through.
pub fn veil() -> impl Bundle {
    let node = Node {
        position_type: PositionType::Absolute,
        width: percent(100),
        height: percent(100),
        ..default()
    };
    (Veil, node, BackgroundColor(Color::NONE))
}

/// Returns the opacity of the veil `since` seconds after a change of level.
pub fn shade(since: f32) -> f32 {
    (1.0 - since / FADE).clamp(0.0, 1.0)
}

/// The level shown and the chart: its tab and what it charts.
type Shown = (Level, Tab, Option<Target>);

/// Tints the veil when the level or the chart changes, then clears it over [`FADE`] seconds.
pub(super) fn fade(
    (time, level, board): (Res<Time>, Res<Level>, Res<Board>),
    mut veils: Query<&mut BackgroundColor, With<Veil>>,
    mut state: Local<(Option<Shown>, f32)>,
) {
    let now = Some((*level, board.tab, board.inside));
    if state.0 != now {
        *state = (now, 0.0);
    }
    if state.1 > FADE {
        return;
    }
    state.1 += time.delta_secs();
    let wanted = BackgroundColor(theme::PANEL.with_alpha(shade(state.1)));
    for mut veil in &mut veils {
        veil.set_if_neq(wanted);
    }
}
