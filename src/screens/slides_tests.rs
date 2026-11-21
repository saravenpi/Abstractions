//! Tests of the title slideshow: it advances after the slide time, the two pictures always share
//! the whole view, it wraps after the last, it drifts without ever uncovering the window, and
//! nothing of it is left once the game is played.

use std::time::Duration;

use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;

use super::Screen;
use super::slides::{Slideshow, zoom};
use super::title::{Incoming, Picture, SLIDES, cover};
use crate::menus::Menu;
use crate::ui::motion::{SLIDE, SLIDE_DRIFT, SLIDE_FADE};

/// Seconds of one step of the tests.
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

/// Returns how many layers of the slideshow exist, and how many images are in memory.
fn held(app: &mut App) -> (usize, usize) {
    let mut layers = app.world_mut().query::<&Picture>();
    let count = layers.iter(app.world()).count();
    (count, app.world().resource::<Assets<Image>>().len())
}

/// A picture stays for the slide time, then the next one is shown, and the clock starts again.
#[test]
fn the_slideshow_advances_after_the_slide_time() {
    let mut show = Slideshow::starting(0, 3);
    let steps = (SLIDE / STEP).ceil() as u32;
    for _ in 1..steps {
        show.tick(STEP, 3);
        assert_eq!((show.slide, show.turn), (0, 0), "{}", show.clock);
    }
    assert!(show.fading());
    show.tick(STEP, 3);
    assert_eq!((show.slide, show.turn), (1, 1));
    assert!(
        show.clock < STEP + 1.0e-3 && !show.fading(),
        "{}",
        show.clock
    );
}

/// The shares of the two pictures sum to 1 at every moment, the next one has none before its
/// fade, rises without ever falling during it, and has it all at the end.
#[test]
fn the_two_layers_always_sum_to_one() {
    let mut show = Slideshow::starting(0, 3);
    let mut last = 0.0;
    let mut risen = false;
    while show.turn == 0 {
        let (shown, next) = show.weights();
        assert!((shown + next - 1.0).abs() < 1.0e-6, "{shown} {next}");
        assert!((0.0..=1.0).contains(&next) && next >= last, "{next} {last}");
        assert_eq!(
            next > 0.0,
            show.clock > SLIDE - SLIDE_FADE,
            "{}",
            show.clock
        );
        (last, risen) = (next, risen || next > 0.5);
        show.tick(0.01, 3);
    }
    assert!(risen && last > 0.98, "{last}");
    assert_eq!(show.weights(), (1.0, 0.0));
}

/// After the last picture comes the first, whatever picture the seed opened on, and a set of
/// one picture shows it again.
#[test]
fn the_slideshow_wraps_after_the_last() {
    for seed in 0..7_u64 {
        let mut show = Slideshow::starting(seed, 3);
        assert_eq!(show.slide, (seed % 3) as usize);
        let mut seen = vec![show.slide];
        for _ in 0..3 {
            show.tick(SLIDE, 3);
            seen.push(show.slide);
        }
        let first = seen[0];
        assert_eq!(seen, [first, (first + 1) % 3, (first + 2) % 3, first]);
    }
    let mut alone = Slideshow::starting(5, 1);
    alone.tick(SLIDE, 1);
    assert_eq!((alone.slide, alone.next(1), alone.turn), (0, 0, 1));
}

/// A picture is never still and never smaller than the window: its scale moves every moment
/// between 1 and 1 plus the drift, and it does not jump when it becomes the one shown.
#[test]
fn a_picture_drifts_and_never_jumps() {
    let mut show = Slideshow::starting(0, 3);
    let mut before = show.zooms();
    for _ in 0..(3.0 * SLIDE / 0.05) as u32 {
        let turn = show.turn;
        show.tick(0.05, 3);
        let now = show.zooms();
        let came = if show.turn == turn {
            before.0
        } else {
            before.1
        };
        assert!((1.0..=1.0 + SLIDE_DRIFT).contains(&now.0), "{now:?}");
        assert!(
            now.0 != came && (now.0 - came).abs() < 1.0e-3,
            "{came} {now:?}"
        );
        before = now;
    }
    assert_eq!(zoom(0, 0.0), 1.0);
    assert!((zoom(1, 0.0) - 1.0 - SLIDE_DRIFT).abs() < 1.0e-6);
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
        let (layers, images) = held(&mut app);
        assert!(
            layers == 2 && (1..=2).contains(&images),
            "{layers} {images}"
        );
    }
    let steps = (SLIDE / STEP) as i32;
    assert!((frames - steps).abs() <= 2, "{frames} frames");
    let show = *app.world().resource::<Slideshow>();
    assert_eq!(show.slide, (first + 1) % SLIDES.len());
    let mut back = app
        .world_mut()
        .query_filtered::<(&Picture, &ImageNode), Without<Incoming>>();
    let (picture, node) = back.single(app.world()).unwrap();
    assert_eq!((picture.slide, node.color.alpha()), (show.slide, 1.0));
    app.world_mut()
        .resource_mut::<NextState<Screen>>()
        .set(Screen::Gameplay);
    for _ in 0..3 {
        app.update();
    }
    assert!(!app.world().contains_resource::<Slideshow>());
    assert_eq!(held(&mut app), (0, 0));
}

/// Every picture of the set covers every window the title tests check, at every scale of its
/// drift: the scale grows it about its middle, so it only ever covers more.
#[test]
fn every_slide_covers_every_window() {
    let sizes = [
        Vec2::new(1920.0, 600.0),
        Vec2::new(800.0, 1000.0),
        Vec2::new(1000.0, 500.0),
        Vec2::new(960.0, 540.0),
        Vec2::new(1280.0, 720.0),
        Vec2::new(3840.0, 2160.0),
    ];
    for (size, aspect) in sizes.into_iter().flat_map(|s| [(s, 16.0 / 9.0), (s, 2.15)]) {
        let (shown, corner) = cover(size, aspect);
        for scale in [1.0, 1.0 + 0.5 * SLIDE_DRIFT, 1.0 + SLIDE_DRIFT] {
            let middle = corner + shown * 0.5;
            let (low, high) = (middle - shown * 0.5 * scale, middle + shown * 0.5 * scale);
            assert!(low.x <= 1.0e-3 && low.y <= 1.0e-3, "{size} {aspect} {low}");
            assert!(high.x >= size.x - 1.0e-3 && high.y >= size.y - 1.0e-3);
        }
    }
}
