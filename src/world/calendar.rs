//! The calendar of the streamed planet: its globe is told the season every few seconds, so the climate of
//! the moment, the weather and the climate line follow the orbit while the ground keeps the annual mean.

use bevy::prelude::*;

use super::Planet;
use crate::climate::season;
use crate::space::Clock;
use crate::universe::{BodyId, PlanetSpec, Universe};

/// Seconds of play between two seasons told to the globe. A year lasts minutes at the least, so the
/// season moves by well under a hundredth in that time.
const INTERVAL: f32 = 3.0;

/// When the globe was last told its season: the seconds left before the next time, and the spec of
/// the planet it was for, kept so that it is not rebuilt each time.
#[derive(Default)]
pub(super) struct Told {
    /// Seconds of play left before the season is told again.
    wait: f32,
    /// Body and spec of the planet last told.
    spec: Option<(BodyId, PlanetSpec)>,
}

/// Tells the globe of the streamed planet its season at the universe time, every [`INTERVAL`] seconds
/// and at once when another planet is entered.
pub(super) fn turn(
    time: Res<Time>,
    planet: Res<Planet>,
    universe: Res<Universe>,
    clock: Option<Res<Clock>>,
    mut told: Local<Told>,
) {
    told.wait -= time.delta_secs();
    let entered = told
        .spec
        .as_ref()
        .is_none_or(|(body, _)| *body != planet.body);
    if told.wait > 0.0 && !entered {
        return;
    }
    told.wait = INTERVAL;
    if entered {
        told.spec = Some((planet.body, universe.planet(planet.body)));
    }
    let seconds = clock.map_or(0.0, |clock| clock.seconds);
    if let Some((_, spec)) = &told.spec {
        planet.globe.set_season(season(spec, seconds));
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;

    use bevy::time::TimeUpdateStrategy;

    use super::*;
    use crate::world::Globe;

    /// Returns an app streaming the home planet of `universe` at `seconds` of universe time, with the
    /// calendar running and a quarter second of play per update.
    fn calendar(universe: Universe, seconds: f64) -> App {
        let spec = universe.planet(universe.home());
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
                250,
            )))
            .insert_resource(Clock { seconds })
            .insert_resource(Planet {
                body: spec.id,
                globe: Arc::new(Globe::of(&spec)),
                day_seconds: spec.day_seconds,
            })
            .insert_resource(universe)
            .add_systems(Update, turn);
        app
    }

    /// The globe is told the season of the orbit at once, keeps it between two tellings and follows the
    /// calendar a few seconds later.
    #[test]
    fn the_globe_follows_the_calendar() {
        let universe = Universe::new(3);
        let spec = universe.planet(universe.home());
        let quarter = spec.orbit().seconds / 4.0;
        let mut app = calendar(universe, quarter);
        app.update();
        let globe = app.world().resource::<Planet>().globe.clone();
        assert_eq!(globe.season(), season(&spec, quarter));
        app.world_mut().resource_mut::<Clock>().seconds = 2.0 * quarter;
        app.update();
        assert_eq!(globe.season(), season(&spec, quarter));
        (0..12).for_each(|_| app.update());
        assert_eq!(globe.season(), season(&spec, 2.0 * quarter));
        let moved = (globe.season() - season(&spec, quarter)).rem_euclid(1.0);
        assert!((moved - 0.25).abs() < 1e-3, "{moved}");
    }
}
