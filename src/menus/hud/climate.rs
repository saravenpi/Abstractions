//! The climate line: a short line under the compass with the air temperature where the player
//! is and the weather, such as "14 C - clear". It shows only in the air of a planet.

use bevy::math::{DVec3, Vec2};
use bevy::prelude::*;

use super::minimap::Reader;
use crate::climate::season as orbit_season;
use crate::climate::{self, Setting, Spot};
use crate::computer::temperature;
use crate::space::Clock;
use crate::ui::{text, theme};
use crate::universe::{PlanetSpec, Universe};

/// Pixels from the top of the screen to the line: just under the compass and its distances.
const TOP: f32 = 64.0;

/// Lowest height in metres the line shows up to, for a planet whose air ends lower or that has
/// no air at all.
const FLOOR: f64 = 2000.0;

/// The word the line uses while nothing says what the weather is.
pub const CLEAR: &str = "clear";

/// The weather the HUD writes: one plain word, such as "rain". The weather module keeps it.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HudWeather {
    /// The word written after the temperature.
    pub word: &'static str,
}

impl Default for HudWeather {
    /// Clear weather.
    fn default() -> Self {
        Self { word: CLEAR }
    }
}

/// Marks the text of the climate line.
#[derive(Component)]
pub(super) struct ClimateGauge;

/// The climate line, centred under the compass and hidden until the player is on a planet.
pub(super) fn line() -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            top: px(TOP),
            justify_content: JustifyContent::Center,
            ..default()
        },
        children![(
            ClimateGauge,
            Node {
                display: Display::None,
                ..default()
            },
            text("", theme::FONT_SMALL, theme::MUTED),
            TextLayout::no_wrap(),
        )],
    )
}

/// Returns the climate line at `at`, metres from the centre of `spec` in its frame, at `time`,
/// or nothing above the air of the planet.
pub(super) fn reading(spec: &PlanetSpec, at: DVec3, time: f64, word: &str) -> Option<String> {
    let altitude = at.length() - spec.radius;
    if altitude > spec.atmosphere_height.max(FLOOR) {
        return None;
    }
    let spot = Spot {
        dir: at.normalize(),
        altitude: altitude as f32,
        inland: 0.0,
        slope: Vec2::ZERO,
    };
    let air = climate::at(&Setting::of(spec), &spot, orbit_season(spec, time));
    Some(format!(
        "{} - {word}",
        temperature(f64::from(air.temperature))
    ))
}

/// What the climate line reads besides the place of the player.
type Sources<'w> = (
    Option<Res<'w, Universe>>,
    Option<Res<'w, Clock>>,
    Option<Res<'w, HudWeather>>,
);

/// Writes the temperature and the weather where the player is, and hides the line in space.
/// The planet is looked up once each time the player reaches another one.
pub(super) fn update(
    reader: Reader,
    (universe, clock, weather): Sources,
    mut spec: Local<Option<PlanetSpec>>,
    mut gauges: Query<(&mut Node, &mut Text), With<ClimateGauge>>,
) {
    let pose = reader.pose();
    if let Some((pose, universe)) = pose.zip(universe.as_deref())
        && spec.as_ref().is_none_or(|spec| spec.id != pose.body)
    {
        *spec = Some(universe.planet(pose.body));
    }
    let word = weather.map_or(CLEAR, |weather| weather.word);
    let time = clock.map_or(0.0, |clock| clock.seconds);
    let shown = pose
        .zip(spec.as_ref())
        .filter(|(pose, spec)| spec.id == pose.body)
        .and_then(|(pose, spec)| reading(spec, pose.at, time, word));
    let display = if shown.is_some() {
        Display::Flex
    } else {
        Display::None
    };
    for (mut node, mut text) in &mut gauges {
        if node.display != display {
            node.display = display;
        }
        if let Some(shown) = &shown {
            text.set_if_neq(Text(shown.clone()));
        }
    }
}
