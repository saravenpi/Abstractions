//! The climate line: a short line under the compass with the air temperature where the player
//! is and the weather, such as "14 C - clear". It shows only in the air of a planet.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::context::Element;
use super::fade::gate;
use super::minimap::Reader;
use crate::computer::temperature;
use crate::render::{PlaceLight, PlaceUniform};
use crate::ui::{Icon, Tone, icon_sized, text, theme};
use crate::universe::{PlanetSpec, Universe};
use crate::world::{Globe, Planet};

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

/// Marks the climate line, a row of two icons and their words.
#[derive(Component)]
pub(super) struct ClimateGauge;

/// Marks the temperature of the climate line.
#[derive(Component)]
pub(super) struct Degrees;

/// Marks the word of the weather on the climate line.
#[derive(Component)]
pub(super) struct Sky;

/// Marks the icon of the weather on the climate line.
#[derive(Component)]
pub(super) struct SkyIcon;

/// Every word the weather feed writes, which [`sky_icon`] has an icon for.
#[cfg(test)]
pub const WORDS: [&str; 7] = ["clear", "overcast", "rain", "snow", "storm", "wind", "dust"];

/// Returns the icon of a weather `word`: a sun by day and a moon by night for a clear sky, and
/// for the rest the picture of what falls or blows, a cloud, rain, snow, a thunderstorm, wind
/// and dust.
pub fn sky_icon(word: &str, day: bool) -> Option<Icon> {
    Some(match word {
        "clear" if day => Icon::Sun,
        "clear" => Icon::ClearNight,
        "overcast" => Icon::Cloudy,
        "rain" => Icon::Rain,
        "snow" => Icon::Snow,
        "storm" => Icon::Storm,
        "wind" => Icon::Air,
        "dust" => Icon::Dust,
        _ => return None,
    })
}

/// Returns true when the star stands above the horizon at the camera in `light`, or when the
/// planet is not lit by a star this way, which reads as day.
pub fn lit(light: &PlaceUniform) -> bool {
    light.air.z == 0.0 || light.star.truncate().dot(light.upward.truncate()) > 0.0
}

/// The climate line, under the coordinates line, hidden until the player is on a planet.
pub(super) fn line() -> impl Bundle {
    (
        ClimateGauge,
        gate(Element::Climate),
        Node {
            display: Display::None,
            align_items: AlignItems::Center,
            column_gap: px(theme::S2),
            padding: UiRect::axes(px(theme::S3), px(theme::S2)),
            ..default()
        },
        BackgroundColor(theme::PANEL),
        children![
            icon_sized(Icon::Temperature, Tone::Plain, theme::ICON_SMALL),
            (
                Degrees,
                text("", theme::FONT_SMALL, theme::TEXT),
                TextLayout::no_wrap()
            ),
            (
                SkyIcon,
                icon_sized(Icon::Sun, Tone::Plain, theme::ICON_SMALL)
            ),
            (
                Sky,
                text("", theme::FONT_SMALL, theme::TEXT),
                TextLayout::no_wrap()
            ),
        ],
    )
}

/// Returns the temperature at `at`, metres from the centre of `spec` in its frame, in the air of
/// its streamed `globe` at the season the globe is at, or nothing above the air of the planet.
pub(super) fn reading(spec: &PlanetSpec, globe: &Globe, at: DVec3) -> Option<String> {
    let altitude = at.length() - spec.radius;
    if altitude > spec.atmosphere_height.max(FLOOR) {
        return None;
    }
    let air = globe.air(at.normalize(), altitude as f32);
    Some(temperature(f64::from(air.temperature)))
}

/// What the climate line reads besides the place of the player.
type Sources<'w> = (
    Option<Res<'w, Universe>>,
    Option<Res<'w, Planet>>,
    Option<Res<'w, HudWeather>>,
    Option<Res<'w, PlaceLight>>,
);

/// The nodes of the climate line: the line itself, its temperature, its word and its icon.
type Parts<'w, 's> = (
    Query<'w, 's, &'static mut Node, With<ClimateGauge>>,
    Query<'w, 's, &'static mut Text, (With<Degrees>, Without<Sky>, Without<SkyIcon>)>,
    Query<'w, 's, &'static mut Text, (With<Sky>, Without<Degrees>, Without<SkyIcon>)>,
    Query<'w, 's, &'static mut Text, (With<SkyIcon>, Without<Degrees>, Without<Sky>)>,
);

/// Writes the temperature and the weather where the player is, and hides the line in space.
/// The planet is looked up once each time the player reaches another one.
pub(super) fn update(
    reader: Reader,
    (universe, planet, weather, light): Sources,
    mut spec: Local<Option<PlanetSpec>>,
    mut parts: Parts,
) {
    let pose = reader.pose();
    if let Some((pose, universe)) = pose.zip(universe.as_deref())
        && spec.as_ref().is_none_or(|spec| spec.id != pose.body)
    {
        *spec = Some(universe.planet(pose.body));
    }
    let word = weather.map_or(CLEAR, |weather| weather.word);
    let planet = planet.filter(|planet| pose.is_some_and(|pose| planet.body == pose.body));
    let shown = pose
        .zip(spec.as_ref())
        .zip(planet.as_deref())
        .filter(|((pose, spec), _)| spec.id == pose.body)
        .and_then(|((pose, spec), planet)| reading(spec, &planet.globe, pose.at));
    let day = light.is_none_or(|light| lit(&light.0));
    write(&mut parts, shown, (word, day));
}

/// Shows the line with `shown` as its temperature and `word` as its weather, or hides it when
/// there is no temperature. `day` picks the icon of a clear sky.
fn write(parts: &mut Parts, shown: Option<String>, (word, day): (&str, bool)) {
    let (lines, degrees, skies, icons) = parts;
    let display = [Display::None, Display::Flex][usize::from(shown.is_some())];
    for mut node in lines {
        if node.display != display {
            node.display = display;
        }
    }
    let Some(shown) = shown else {
        return;
    };
    let glyph = sky_icon(word, day).unwrap_or(Icon::PartlyCloudy).glyph();
    for mut text in degrees {
        text.set_if_neq(Text(shown.clone()));
    }
    for mut text in skies {
        text.set_if_neq(Text(word.to_string()));
    }
    for mut text in icons {
        text.set_if_neq(Text(glyph.clone()));
    }
}
