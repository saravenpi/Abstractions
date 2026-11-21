//! Banner tests: a player who joins or leaves, a line of the network or of the ground, shows in the
//! banner, over a planet name.

use bevy::prelude::*;

use super::banner::{Banner, PlanetName, announce, block, presence};
use crate::craft::ground::{Note, TAKEN};
use crate::net::{PlayerJoined, PlayerLeft, SessionNotice};

/// The lines read "<name> joined" and "<name> left".
#[test]
fn presence_lines_name_the_player() {
    assert_eq!(presence("Ada", true), "Ada joined");
    assert_eq!(presence("Ada", false), "Ada left");
}

/// Returns an app that announces in a banner.
fn app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<Banner>()
        .add_message::<PlayerJoined>()
        .add_message::<PlayerLeft>()
        .add_message::<SessionNotice>()
        .add_message::<Note>()
        .add_systems(Update, announce);
    app.world_mut().spawn(block());
    app
}

/// A join, then a leave in the same frame, start the banner with the last of them; a line of the
/// network replaces it.
#[test]
fn joins_and_leaves_show_in_the_banner() {
    let mut app = app();
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
    app.world_mut().write_message(SessionNotice {
        line: "New server key 7F3A-91C2-0B5E".to_string(),
    });
    app.update();
    let shown: Vec<String> = names.iter(app.world()).map(|text| text.0.clone()).collect();
    assert_eq!(shown, ["New server key 7F3A-91C2-0B5E"]);
}

/// A line of the ground, such as a thing another player took first, shows in the banner.
#[test]
fn a_line_of_the_ground_shows_in_the_banner() {
    let mut app = app();
    app.update();
    app.world_mut().write_message(Note(TAKEN.to_string()));
    app.update();
    let mut names = app.world_mut().query_filtered::<&Text, With<PlanetName>>();
    let shown: Vec<String> = names.iter(app.world()).map(|text| text.0.clone()).collect();
    assert_eq!(shown, [TAKEN]);
}
