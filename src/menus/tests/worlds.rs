//! The worlds menu inside the app: rename, duplicate, sort and delete with the keyboard.

use bevy::input::keyboard::Key;
use bevy::prelude::*;

use super::{app, press, tap};
use crate::menus::Menu;
use crate::saves::{SaveDir, WorldEntry, list, new_entry, write};

/// Writes two worlds and opens the worlds menu. "Reef" was played last, so it is selected.
pub(super) fn with_two_worlds(tag: &str) -> App {
    let mut app = app(tag);
    let dir = app.world().resource::<SaveDir>().0.clone();
    for (name, seed, played) in [("Isle", 1, 100), ("Reef", 2, 200)] {
        let mut entry = new_entry(&dir, name, seed);
        entry.save.last_played = played;
        write(&dir, &entry.id, &entry.save).unwrap();
    }
    app.world_mut()
        .resource_mut::<NextState<Menu>>()
        .set(Menu::Worlds);
    for _ in 0..3 {
        app.update();
    }
    app
}

/// Returns the worlds on disk, ordered by id.
pub(super) fn on_disk(app: &App) -> Vec<WorldEntry> {
    let mut worlds = list(&app.world().resource::<SaveDir>().0);
    worlds.sort_by(|a, b| a.id.cmp(&b.id));
    worlds
}

/// Types `text`, one key per character.
fn type_text(app: &mut App, text: &str) {
    for c in text.chars() {
        let typed = c.to_string();
        press(
            app,
            KeyCode::KeyA,
            Key::Character(typed.as_str().into()),
            Some(&typed),
        );
    }
}

/// R opens the name field, a refused name keeps it open, Escape drops it, Enter applies it.
#[test]
fn rename_types_refuses_cancels_and_applies() {
    let mut app = with_two_worlds("rename");
    tap(&mut app, KeyCode::KeyR);
    for _ in 0..4 {
        tap(&mut app, KeyCode::Backspace);
    }
    type_text(&mut app, "isle");
    tap(&mut app, KeyCode::Enter);
    assert_eq!(
        on_disk(&app)[1].save.name,
        "Reef",
        "a taken name is refused"
    );
    tap(&mut app, KeyCode::Escape);
    assert_eq!(*app.world().resource::<State<Menu>>().get(), Menu::Worlds);
    tap(&mut app, KeyCode::KeyR);
    type_text(&mut app, " two ");
    tap(&mut app, KeyCode::Enter);
    let worlds = on_disk(&app);
    assert_eq!(
        (worlds[1].id.as_str(), worlds[1].save.name.as_str()),
        ("reef", "Reef two")
    );
    assert_eq!(worlds[0].save.name, "Isle");
    assert_eq!(*app.world().resource::<State<Menu>>().get(), Menu::Worlds);
}

/// C copies the selected world. Delete asks first: Enter alone keeps the world, and only
/// choosing delete, then Enter, removes it and nothing else.
#[test]
fn duplicate_and_delete_confirm_first() {
    let mut app = with_two_worlds("delete");
    tap(&mut app, KeyCode::KeyC);
    let names =
        |app: &App| -> Vec<String> { on_disk(app).into_iter().map(|w| w.save.name).collect() };
    assert_eq!(names(&app), ["Isle", "Reef", "Reef (copy)"]);
    assert_eq!(on_disk(&app)[2].save.seed, 2);
    tap(&mut app, KeyCode::Delete);
    tap(&mut app, KeyCode::Enter);
    assert_eq!(names(&app).len(), 3, "the default choice keeps the world");
    tap(&mut app, KeyCode::Delete);
    tap(&mut app, KeyCode::Escape);
    assert_eq!(names(&app).len(), 3);
    assert_eq!(*app.world().resource::<State<Menu>>().get(), Menu::Worlds);
    tap(&mut app, KeyCode::Delete);
    tap(&mut app, KeyCode::ArrowRight);
    tap(&mut app, KeyCode::Enter);
    assert_eq!(
        names(&app),
        ["Isle", "Reef"],
        "only the selected copy is gone"
    );
    for _ in 0..2 {
        tap(&mut app, KeyCode::Delete);
        tap(&mut app, KeyCode::ArrowRight);
        tap(&mut app, KeyCode::Enter);
    }
    assert!(names(&app).is_empty());
    app.update();
    assert_eq!(*app.world().resource::<State<Menu>>().get(), Menu::Worlds);
}
