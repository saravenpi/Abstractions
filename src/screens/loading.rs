//! Loading screen shown while the world generates. It says what is being generated.

use bevy::prelude::*;

use super::{Screen, WarmUp};
use crate::menus::Menu;
use crate::saves::ActiveWorld;
use crate::ui::{root_on, text, theme};
use crate::universe::Universe;
use crate::world::{WorldReady, WorldSeed};

/// Seconds of play under which a world counts as new: it gets the full wake-up sequence.
const NEW_WORLD_SECONDS: f64 = 1.0;

/// Where the player is about to arrive, and whether for the first time.
#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct Arrival {
    /// True for a world that was never played.
    pub fresh: bool,
    /// Name of the planet.
    pub planet: String,
    /// Name of its star.
    pub star: String,
}

impl Arrival {
    /// Returns the arrival of the active world: its saved planet, or the home planet of a new
    /// world. Without an active world, as in test runs, it is a return to the home planet.
    pub fn of(active: Option<&ActiveWorld>, seed: u64) -> Self {
        let universe = Universe::new(seed);
        let save = active.map(|active| &active.0.save);
        let body = save.map_or_else(|| universe.home(), |save| save.body(&universe));
        Self {
            fresh: save.is_some_and(|save| save.play_seconds < NEW_WORLD_SECONDS),
            planet: universe.planet(body).name,
            star: universe.system(body.star).star.name,
        }
    }
}

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
            margin: UiRect::top(px(theme::GAP_S)),
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
            BackgroundColor(theme::TEXT),
        )],
    )
}

/// Shows what is being generated, the system, the seed and a thin sweeping bar at the bottom left.
fn enter(mut commands: Commands, seed: Option<Res<WorldSeed>>, active: Option<Res<ActiveWorld>>) {
    let seed = seed.map_or(0, |seed| seed.0);
    let arrival = Arrival::of(active.as_deref(), seed);
    let detail = format!("{} system, seed {seed}", arrival.star);
    commands.spawn((
        DespawnOnExit(Screen::Loading),
        root_on(theme::BACKGROUND, JustifyContent::FlexEnd),
        children![
            text(
                format!("Generating {}", arrival.planet),
                theme::FONT_HEADING,
                theme::TEXT
            ),
            (
                text(detail, theme::FONT_SMALL, theme::MUTED),
                Node {
                    margin: UiRect::vertical(px(theme::GAP_S)),
                    ..default()
                }
            ),
            track(),
        ],
    ));
    commands.insert_resource(arrival);
}

/// Slides the bar back and forth.
fn animate(time: Res<Time>, mut bars: Query<&mut Node, With<Sweep>>) {
    for mut node in &mut bars {
        node.left = percent(sweep_offset(time.elapsed_secs()));
    }
}

/// Moves to gameplay, hidden behind the boot screen, once the world reports it is ready around
/// the place the body starts at.
fn wait_for_world(
    ready: Option<Res<WorldReady>>,
    mut screen: ResMut<NextState<Screen>>,
    mut menu: ResMut<NextState<Menu>>,
    mut warm: ResMut<NextState<WarmUp>>,
) {
    if ready.is_some_and(|ready| ready.0) {
        screen.set(Screen::Gameplay);
        menu.set(Menu::None);
        warm.set(WarmUp::Hidden);
    }
}

/// Registers the loading screen systems.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Loading), enter)
        .add_systems(
            Update,
            (wait_for_world.run_if(crate::player::awake), animate)
                .run_if(in_state(Screen::Loading)),
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
