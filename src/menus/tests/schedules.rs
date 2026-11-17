//! Schedule tests: every system of the UI can be built, so none asks for conflicting access.
//!
//! A system whose own parameters overlap only fails when it first runs. Systems behind a run
//! condition never run in a short test, so the schedules are built here by hand.

use bevy::ecs::schedule::{InternedScheduleLabel, ScheduleLabel};
use bevy::prelude::*;

use super::app;
use crate::computer::Computer;
use crate::menus::Menu;
use crate::screens::{Screen, WarmUp};

/// Every menu state.
const MENUS: [Menu; 6] = [
    Menu::None,
    Menu::Main,
    Menu::NewGame,
    Menu::Worlds,
    Menu::Settings,
    Menu::Pause,
];

/// Returns every schedule the screens, the menus, the UI and the computer add systems to.
fn labels() -> Vec<InternedScheduleLabel> {
    let mut labels = vec![
        Startup.intern(),
        PreUpdate.intern(),
        Update.intern(),
        PostUpdate.intern(),
        OnEnter(Screen::Title).intern(),
        OnEnter(Screen::Loading).intern(),
        OnEnter(Screen::Gameplay).intern(),
        OnExit(Screen::Gameplay).intern(),
        OnEnter(WarmUp::Hidden).intern(),
        OnEnter(Computer::Open).intern(),
        OnExit(Computer::Open).intern(),
    ];
    for menu in MENUS {
        labels.extend([OnEnter(menu).intern(), OnExit(menu).intern()]);
    }
    labels
}

/// Building every schedule of the UI succeeds, and the main ones exist.
#[test]
fn every_ui_schedule_can_be_built() {
    let mut app = app("schedules");
    let mut built = 0;
    for label in labels() {
        if !app.world().resource::<Schedules>().contains(label) {
            continue;
        }
        let result = app
            .world_mut()
            .schedule_scope(label, |world, schedule| schedule.initialize(world));
        assert!(result.is_ok(), "{label:?}: {:?}", result.err());
        built += 1;
    }
    assert!(built >= 15, "only {built} schedules were found");
}
