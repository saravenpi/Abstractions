//! The calendar of the streamed planet: its globe is told the season every few seconds, so the climate of
//! the moment, the weather and the climate line follow the orbit while the ground keeps the annual mean.

use bevy::prelude::*;

use super::{Planet, reserve};
use crate::climate::season;
use crate::space::Clock;
use crate::universe::{PlanetSpec, Universe};

/// Seconds of play between two seasons told to the globe. A year lasts minutes at the least, so the
/// season moves by well under a hundredth in that time.
const INTERVAL: f32 = 3.0;

/// When the globe was last told its season: the seconds left before the next time, and the spec of
/// the planet it was for, kept so that it is not rebuilt each time.
#[derive(Default)]
pub(super) struct Told {
    /// Seconds of play left before the season is told again.
    wait: f32,
    /// Universe and spec of the planet last told: the same body of another universe is another planet.
    spec: Option<(Universe, PlanetSpec)>,
}

/// Registers the calendar. It runs before the ground of the base is reserved, which swaps in a copy of the
/// globe: the copy is made after the season is told, so it carries it.
pub(super) fn plugin(app: &mut App) {
    let streamed = resource_exists::<Planet>.and_then(resource_exists::<Universe>);
    app.add_systems(Update, turn.run_if(streamed).before(reserve::keep_base));
}

/// Tells the globe of the streamed planet its season at the universe time, every [`INTERVAL`] seconds
/// and at once when another planet, or the same body of another universe, is entered.
pub(super) fn turn(
    time: Res<Time>,
    planet: Res<Planet>,
    universe: Res<Universe>,
    clock: Option<Res<Clock>>,
    mut told: Local<Told>,
) {
    told.wait -= time.delta_secs();
    let known = |(of, spec): &(Universe, PlanetSpec)| *of == *universe && spec.id == planet.body;
    let entered = !told.spec.as_ref().is_some_and(known);
    if told.wait > 0.0 && !entered {
        return;
    }
    told.wait = INTERVAL;
    if entered {
        told.spec = Some((*universe, universe.planet(planet.body)));
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
    use crate::home::Base;
    use crate::world::globe::Face;
    use crate::world::{Globe, Terrain};

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
            .add_plugins(plugin);
        app
    }

    /// The same body id names another planet in another universe: when the universe changes under the
    /// calendar, the globe is told the season of the orbit the body has there, at once.
    #[test]
    fn another_universe_is_another_planet() {
        let (first, second) = (Universe::new(3), Universe::new(4));
        let body = first.home();
        let (old, new) = (first.planet(body), second.planet(body));
        let seconds = 0.3 * old.orbit().seconds;
        assert!((season(&old, seconds) - season(&new, seconds)).abs() > 1e-3);
        let mut app = calendar(first, seconds);
        app.update();
        app.insert_resource(second);
        app.update();
        let globe = app.world().resource::<Planet>().globe.clone();
        assert_eq!(globe.season(), season(&new, seconds));
    }

    /// When the ground of the base is reserved in the frame the season is told, the globe that takes
    /// the place of the first carries the season.
    #[test]
    fn the_reserved_globe_carries_the_season() {
        let universe = Universe::new(3);
        let spec = universe.planet(universe.home());
        let seconds = 0.3 * spec.orbit().seconds;
        let mut app = calendar(universe, seconds);
        let globe = app.world().resource::<Planet>().globe.clone();
        let up = bevy::math::DVec3::X;
        let at = up * spec.radius;
        app.insert_resource(Terrain::on(globe.clone(), globe.face_chart(Face::PosX)))
            .insert_resource(Base {
                body: spec.id,
                centre: at,
                up,
                east: bevy::math::DVec3::Z,
                north: bevy::math::DVec3::Y,
                wake: at,
                pad: at,
                heading: bevy::math::DVec3::Y,
                door: at,
            })
            .add_systems(Update, reserve::keep_base);
        app.update();
        let kept = app.world().resource::<Planet>().globe.clone();
        assert!(kept.reserve().is_some());
        assert_eq!(kept.season(), season(&spec, seconds));
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
