//! Schedule test: every sky system can be built, so no system asks for conflicting access.
//!
//! A system whose own queries overlap only fails when the game starts, which no other test does.

use bevy::asset::AssetPlugin;
use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;

use crate::screens::Screen;

/// Building the schedules of the day clock, sun, sky, clouds, stars, bodies and effects succeeds.
#[test]
fn sky_systems_can_be_built() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.add_plugins((super::ray_tracing::plugin, super::sky));
    let labels = [
        Update.intern(),
        PostUpdate.intern(),
        OnEnter(Screen::Loading).intern(),
    ];
    for label in labels {
        let built = app
            .world_mut()
            .schedule_scope(label, |world, schedule| schedule.initialize(world));
        assert!(built.is_ok(), "{label:?}: {:?}", built.err());
    }
}
