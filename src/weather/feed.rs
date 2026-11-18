//! Hands the weather at the camera to the sky and to the HUD.

use bevy::prelude::*;

use super::Weather;
use crate::menus::HudWeather;
use crate::render::smoothstep as ramp;
use crate::render::{LocalWeather, Vantage, WeatherLook};

/// Shares of the air above the camera under which the sky draws no haze or dust of the place,
/// and from which it draws all of it: the palette of the sky from orbit is the planet's, and on
/// the way down the weather of the place comes in without a step. The clouds need no such
/// blend: they are the same field from every height.
const BAND: (f32, f32) = (0.05, 0.3);

/// Level from which something that happens is worth the word of the HUD.
const SAID: f32 = 0.3;

/// Returns the one word the HUD writes for `weather`, the strongest of what is happening. A
/// storm is one only under its clouds: what is left of it under a clear sky is wind.
pub fn word(weather: &Weather) -> &'static str {
    let squall = weather.squall();
    let events = [
        (weather.storm - squall, "wind"),
        (weather.dust, "dust"),
        (weather.snow, "snow"),
        (weather.rain, "rain"),
        (squall, "storm"),
    ];
    let strongest = events.into_iter().filter(|(level, _)| *level > SAID);
    match strongest.max_by(|a, b| a.0.total_cmp(&b.0)) {
        Some((_, name)) => name,
        None if weather.cloud > 0.7 => "overcast",
        None => "clear",
    }
}

/// Returns what `weather` does to the sky where it is: its dust, and the haze of its rain or
/// of the storm under its clouds.
pub fn local(weather: &Weather) -> WeatherLook {
    WeatherLook {
        haze: weather.rain.max(weather.squall()),
        dust: weather.dust,
    }
}

/// Returns the weather the sky draws with `air` of the atmosphere left above the camera:
/// nothing out of the air and in its thin top, then the calm sky turning into `weather` over
/// [`BAND`].
pub fn seen(weather: &Weather, air: f32) -> Option<WeatherLook> {
    let share = ramp(BAND.0, BAND.1, air);
    let here = local(weather);
    (share > 0.0).then_some(WeatherLook {
        haze: here.haze * share,
        dust: here.dust * share,
    })
}

/// Gives the sky the weather while the camera is in the air of a planet, and the HUD its word.
pub(super) fn feed(
    weather: Res<Weather>,
    vantage: Option<Res<Vantage>>,
    mut sky: ResMut<LocalWeather>,
    mut hud: ResMut<HudWeather>,
) {
    let look = vantage.and_then(|vantage| seen(&weather, vantage.air));
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
