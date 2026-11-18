//! Hands the weather at the camera to the sky and to the HUD.

use bevy::prelude::*;

use super::Weather;
use crate::menus::HudWeather;
use crate::render::smoothstep as ramp;
use crate::render::{LocalSystem, LocalWeather, Vantage, WeatherLook};

/// Shares of the air above the camera under which the sky draws the planet's mean look, and
/// from which it draws the weather of the place: from orbit the clouds are the planet's, not
/// the weather at one place, and on the way down one turns into the other without a step.
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

/// Returns what `weather` does to the sky where it is: its cloud, its dust, and the haze of
/// its rain or of the storm under its clouds.
pub fn local(weather: &Weather) -> WeatherLook {
    WeatherLook {
        cover: weather.cloud,
        haze: weather.rain.max(weather.squall()),
        dust: weather.dust,
    }
}

/// Returns the weather the sky draws with `air` of the atmosphere left above the camera, over
/// a planet whose class look is `mean`: nothing, which is the class look, out of the air and
/// in its thin top, then the class look turning into `weather` over [`BAND`].
pub fn seen(weather: &Weather, mean: WeatherLook, air: f32) -> Option<WeatherLook> {
    let share = ramp(BAND.0, BAND.1, air);
    let here = local(weather);
    (share > 0.0).then(|| WeatherLook {
        cover: mean.cover.lerp(here.cover, share),
        haze: mean.haze.lerp(here.haze, share),
        dust: mean.dust.lerp(here.dust, share),
    })
}

/// Gives the sky the weather while the camera is in the air of a planet, and the HUD its word.
pub(super) fn feed(
    weather: Res<Weather>,
    vantage: Option<Res<Vantage>>,
    system: Option<Res<LocalSystem>>,
    mut sky: ResMut<LocalWeather>,
    mut hud: ResMut<HudWeather>,
) {
    let look = vantage.zip(system).and_then(|(vantage, system)| {
        let mean = WeatherLook::of(&system.air_below(&vantage));
        seen(&weather, mean, vantage.air)
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
