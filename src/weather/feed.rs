//! Hands the weather at the camera to the sky and to the HUD.

use bevy::prelude::*;

use super::Weather;
use crate::menus::HudWeather;
use crate::render::{LocalWeather, Vantage, WeatherLook};

/// Share of the air above the camera under which the sky draws the planet's mean look again:
/// from orbit the clouds are the planet's, not the weather at one place.
const IN_THE_AIR: f32 = 0.05;

/// Returns the one word the HUD writes for `weather`, the strongest of what is happening.
pub fn word(weather: &Weather) -> &'static str {
    let events = [
        (weather.storm, "storm"),
        (weather.dust, "dust"),
        (weather.snow, "snow"),
        (weather.rain, "rain"),
    ];
    let strongest = events.into_iter().filter(|(level, _)| *level > 0.3);
    match strongest.max_by(|a, b| a.0.total_cmp(&b.0)) {
        Some((_, name)) => name,
        None if weather.cloud > 0.7 => "overcast",
        None => "clear",
    }
}

/// Gives the sky the weather while the camera is in the air of a planet, and the HUD its word.
pub(super) fn feed(
    weather: Res<Weather>,
    vantage: Option<Res<Vantage>>,
    mut sky: ResMut<LocalWeather>,
    mut hud: ResMut<HudWeather>,
) {
    let inside = vantage.is_some_and(|vantage| vantage.air > IN_THE_AIR);
    let look = inside.then_some(WeatherLook {
        cover: weather.cloud,
        haze: weather.rain.max(weather.storm),
        dust: weather.dust,
    });
    if sky.0 != look {
        sky.0 = look;
    }
    let said = HudWeather {
        word: word(&weather),
    };
    if *hud != said {
        *hud = said;
    }
}
