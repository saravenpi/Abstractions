//! Banner tests: a player who joins or leaves shows in the banner, over a planet name.

use bevy::prelude::*;

use super::banner::{Banner, PlanetName, announce, block, presence};
use crate::net::{PlayerJoined, PlayerLeft};

/// The lines read "<name> joined" and "<name> left".
#[test]
fn presence_lines_name_the_player() {
    assert_eq!(presence("Ada", true), "Ada joined");
    assert_eq!(presence("Ada", false), "Ada left");
}

/// A join, then a leave in the same frame, start the banner with the last of them.
#[test]
fn joins_and_leaves_show_in_the_banner() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<Banner>()
        .add_message::<PlayerJoined>()
        .add_message::<PlayerLeft>()
        .add_systems(Update, announce);
    app.world_mut().spawn(block());
    app.update();
    assert_eq!(*app.world().resource::<Banner>(), Banner::default());
    app.world_mut().write_message(PlayerJoined {
        name: "Ada".to_string(),
    });
    app.world_mut().write_message(PlayerLeft {
        name: "Lin".to_string(),
    });
    app.update();
    let mut names = app.world_mut().query_filtered::<&Text, With<PlanetName>>();
    let shown: Vec<String> = names.iter(app.world()).map(|text| text.0.clone()).collect();
    assert_eq!(shown, ["Lin left"]);
    assert_ne!(*app.world().resource::<Banner>(), Banner::default());
}
