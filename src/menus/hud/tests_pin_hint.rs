//! Tests of the pin hint: a pin dropped in any way retires it for good, the retired state lives in
//! the settings file and survives a new session, the hint shows ten seconds in all in a session
//! however often the player opens the map, and the prompt line never offers to drop a pin when
//! there is nothing to do.

use std::path::{Path, PathBuf};

use bevy::math::DVec3;
use bevy::prelude::*;
use bevy_persistent::StorageFormat;

use super::context::Context;
use super::hints::{Clock, HOLD, Hint, done, due, retire, with};
use super::prompt::{State, choose};
use crate::map::Places;
use crate::saves::Pin;
use crate::settings::{Persistent, Settings};
use crate::universe::BodyId;

/// Returns an empty scratch directory of its own for one test.
fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "abstractions-pin-hint-{tag}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

/// Returns the settings stored in `dir`, read back from the file when there is one.
fn stored(dir: &Path) -> Persistent<Settings> {
    Persistent::<Settings>::builder()
        .name("settings")
        .format(StorageFormat::Toml)
        .path(dir.join("settings.toml"))
        .default(Settings::default())
        .build()
        .unwrap()
}

/// Returns a headless app that retires the pin hint as the game does, its settings in `dir`.
fn app(dir: &Path) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<Places>()
        .insert_resource(stored(dir))
        .add_systems(Update, retire.run_if(resource_exists_and_changed::<Places>));
    app.update();
    app
}

/// Returns whether the pin hint is done in the settings of `app`.
fn retired(app: &App) -> bool {
    done(
        app.world().resource::<Persistent<Settings>>().hints,
        Hint::Pin,
    )
}

/// A way to leave a pin with the player: its name and what it does to the places.
type Way = (&'static str, fn(&mut Places));

/// The pin key at the feet, a right click on the map and a pin kept in the world each retire
/// the hint: they all leave a pin with the player.
#[test]
fn a_pin_dropped_in_any_way_retires_the_hint() {
    let body = BodyId::default();
    let ways: [Way; 3] = [
        ("key", |places| {
            places.drop(BodyId::default(), DVec3::X);
        }),
        ("map", |places| {
            places.drop(BodyId::default(), DVec3::new(0.2, 0.9, -0.4));
        }),
        ("world", |places| places.pins = vec![Pin::default()]),
    ];
    for (tag, way) in ways {
        let mut app = app(&scratch(tag));
        assert!(!retired(&app), "{tag}");
        way(&mut app.world_mut().resource_mut::<Places>());
        app.update();
        assert!(retired(&app), "{tag}");
        assert_eq!(app.world().resource::<Places>().pins[0].body, body);
    }
}

/// The hint retired in one session is still retired in the next, read back from the file.
#[test]
fn the_retired_hint_survives_a_new_session() {
    let dir = scratch("session");
    let mut app = app(&dir);
    app.world_mut()
        .resource_mut::<Places>()
        .drop(BodyId::default(), DVec3::Y);
    app.update();
    assert!(retired(&app));
    assert!(done(stored(&dir).hints, Hint::Pin));
    let clock = Clock {
        afoot: 600.0,
        ..Clock::default()
    };
    assert_eq!(
        due(
            stored(&dir).hints | Hint::Map.bit() | Hint::Minimap.bit(),
            &clock,
            Context::OnFoot
        ),
        None
    );
}

/// The pin hint shows ten seconds in all in a session: opening the map, which hides it, and
/// closing it again does not give it ten more.
#[test]
fn the_hint_time_adds_up_across_the_map() {
    let bits = with(with(0, Hint::Map), Hint::Minimap);
    let mut clock = Clock {
        afoot: 120.0,
        ..Clock::default()
    };
    for _ in 0..4 {
        for _ in 0..60 {
            let wanted = due(bits, &clock, Context::OnFoot);
            clock.tick(wanted, HOLD / 200.0);
        }
        clock.tick(due(bits, &clock, Context::Menu), 1.0);
    }
    assert!(done(clock.hushed, Hint::Pin));
    assert_eq!(due(bits, &clock, Context::OnFoot), None);
}

/// With nothing to do and every hint done, the prompt line shows nothing: dropping a pin is never
/// a standing offer.
#[test]
fn the_prompt_does_not_offer_a_pin_with_nothing_to_do() {
    let all = Hint::ALL.iter().fold(0, |bits, hint| with(bits, *hint));
    let clock = Clock {
        seated: 600.0,
        afoot: 600.0,
        ..Clock::default()
    };
    for context in Context::ALL {
        assert_eq!(due(all, &clock, context), None, "{context:?}");
    }
    assert_eq!(choose(&State::default()), None);
}
