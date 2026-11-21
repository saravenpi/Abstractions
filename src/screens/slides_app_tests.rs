//! Tests of the title slideshow in the app: it runs on the title screen with two layers and at
//! most two pictures in memory, and nothing of it is left once the game is played.

use std::time::Duration;

use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;

use super::Screen;
use super::slides::Slideshow;
use super::title::{Incoming, Picture, SLIDES};
use crate::menus::Menu;
use crate::ui::motion::SLIDE;

/// Seconds of one frame of the tests.
const STEP: f32 = 0.25;

/// Returns a headless app on the title screen, with images but no renderer, that moves a
/// quarter of a second each frame.
fn app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default(), StatesPlugin))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            STEP,
        )))
        .init_asset::<Image>()
        .init_state::<Screen>()
        .init_state::<Menu>()
        .add_plugins(super::title::plugin);
    app
}

/// Returns how many images are in memory.
fn images(app: &App) -> usize {
    app.world().resource::<Assets<Image>>().len()
}

/// On the title screen the slideshow moves on by itself with two layers, the back one opaque,
/// and at most two pictures in memory. Once the game is played no layer, no state and no
/// picture of it is left.
#[test]
fn nothing_of_the_slideshow_exists_in_gameplay() {
    let mut app = app();
    app.update();
    let first = app.world().resource::<Slideshow>().slide;
    let mut frames = 0;
    while app.world().resource::<Slideshow>().turn == 0 && frames < 200 {
        app.update();
        frames += 1;
        assert!((1..=2).contains(&images(&app)), "{}", images(&app));
    }
    assert!(
        (frames - (SLIDE / STEP) as i32).abs() <= 2,
        "{frames} frames"
    );
    let show = *app.world().resource::<Slideshow>();
    assert_eq!(show.slide, (first + 1) % SLIDES.len());
    let mut layers = app
        .world_mut()
        .query::<(&Picture, &ImageNode, Has<Incoming>)>();
    let found: Vec<_> = layers.iter(app.world()).collect();
    let back = found.iter().find(|layer| !layer.2).unwrap();
    assert_eq!(
        (found.len(), back.0.slide, back.1.color.alpha()),
        (2, show.slide, 1.0)
    );
    let mut next = app.world_mut().resource_mut::<NextState<Screen>>();
    next.set(Screen::Gameplay);
    (0..3).for_each(|_| app.update());
    assert!(!app.world().contains_resource::<Slideshow>());
    assert_eq!((layers.iter(app.world()).count(), images(&app)), (0, 0));
}
