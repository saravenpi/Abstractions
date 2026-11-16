//! Loading screen shown while the world generates.

use bevy::prelude::*;

use super::Screen;
use crate::menus::Menu;
use crate::ui::{root_on, text, theme};
use crate::world::{WorldReady, WorldSeed};

/// Marks the bar that sweeps along the track.
#[derive(Component)]
struct Sweep;

/// Returns the left offset, in percent of the track, of the sweeping bar at `seconds`.
fn sweep_offset(seconds: f32) -> f32 {
    75.0 * (0.5 - 0.5 * (seconds * 2.4).cos())
}

/// Thin track with the sweeping bar inside.
fn track() -> impl Bundle {
    (
        Node {
            width: px(theme::MENU_WIDTH),
            height: px(2),
            margin: UiRect::top(px(theme::SPACE)),
            ..default()
        },
        BackgroundColor(theme::HAIRLINE),
        children![(
            Sweep,
            Node {
                position_type: PositionType::Absolute,
                width: percent(25),
                height: percent(100),
                ..default()
            },
            BackgroundColor(theme::ACCENT),
        )],
    )
}

/// Shows the loading text, the seed and a thin sweeping bar at the bottom left.
fn enter(mut commands: Commands, seed: Option<Res<WorldSeed>>) {
    let shown = seed
        .map(|seed| format!("Seed {}", seed.0))
        .unwrap_or_default();
    commands.spawn((
        DespawnOnExit(Screen::Loading),
        root_on(theme::BACKGROUND, JustifyContent::FlexEnd),
        children![
            text("Generating world", theme::FONT_HEADING, theme::TEXT),
            (
                text(shown, theme::FONT_SMALL, theme::MUTED),
                Node {
                    margin: UiRect::vertical(px(theme::SPACE)),
                    ..default()
                }
            ),
            track(),
        ],
    ));
}

/// Slides the bar back and forth.
fn animate(time: Res<Time>, mut bars: Query<&mut Node, With<Sweep>>) {
    for mut node in &mut bars {
        node.left = percent(sweep_offset(time.elapsed_secs()));
    }
}

/// Moves to gameplay once the world reports it is ready.
fn wait_for_world(
    ready: Option<Res<WorldReady>>,
    mut screen: ResMut<NextState<Screen>>,
    mut menu: ResMut<NextState<Menu>>,
) {
    if ready.is_some_and(|ready| ready.0) {
        screen.set(Screen::Gameplay);
        menu.set(Menu::None);
    }
}

/// Registers the loading screen systems.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Loading), enter)
        .add_systems(
            Update,
            (wait_for_world, animate).run_if(in_state(Screen::Loading)),
        );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The bar starts at the left edge and never leaves the track.
    #[test]
    fn sweep_stays_on_track() {
        assert!(sweep_offset(0.0).abs() < 1e-4);
        assert!((0..200).all(|i| (0.0..=75.0001).contains(&sweep_offset(i as f32 * 0.1))));
    }
}
