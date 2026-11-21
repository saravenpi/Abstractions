//! Title screen: a slideshow of pictures of the game filling the window behind the main menu, and
//! the version label under it. Each picture drifts slowly and the next one fades in over it.

use std::time::{SystemTime, UNIX_EPOCH};

use bevy::asset::RenderAssetUsages;
use bevy::image::{CompressedImageFormats, ImageSampler, ImageType};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::Screen;
use super::slides::{Slideshow, advance};
use crate::menus::Menu;
use crate::ui::{root_on, text, theme};

/// The title pictures, in the order they show. The set is this list: add a file to
/// `assets/images/title` and a line here. Each is a PNG in 256 colours, 1280 by 720 pixels.
pub const SLIDES: [&[u8]; 1] = [include_bytes!("../../assets/images/title/01.png")];

/// Share of the height a picture loses at its top when the window is wider than it, as the
/// website frames it: the horizon and the robot stay in view.
const FOCUS: f32 = 0.62;

/// One of the two layers of the slideshow, and the picture it holds.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Picture {
    /// Index in [`SLIDES`] of the picture held, or past the end while the layer is empty.
    pub slide: usize,
    /// Width over height of that picture.
    pub aspect: f32,
}

/// Marks the layer on top, where the next picture fades in. The other one holds the picture shown.
#[derive(Component)]
pub struct Incoming;

/// What the slideshow writes of a layer.
type Layer = (
    &'static mut Picture,
    &'static mut ImageNode,
    &'static mut UiTransform,
);

/// Returns the size and the top-left corner, in pixels, of a picture of width over height
/// `aspect` that covers a window of `size`: as small as it can be while it fills the window,
/// centred across and framed at [`FOCUS`] down.
pub fn cover(size: Vec2, aspect: f32) -> (Vec2, Vec2) {
    let width = size.x.max(size.y * aspect);
    let shown = Vec2::new(width, width / aspect);
    let corner = (size - shown) * Vec2::new(0.5, FOCUS);
    (shown, corner)
}

/// Spawns the two empty layers under the version label, starts the slideshow on a picture the
/// clock picks and opens the main menu. Without a renderer no picture can be read and the
/// background stays flat.
fn enter(
    mut commands: Commands,
    images: Option<Res<Assets<Image>>>,
    mut menu: ResMut<NextState<Menu>>,
) {
    let now = SystemTime::now().duration_since(UNIX_EPOCH);
    let seed = now.map_or(0, |since| since.as_secs());
    commands.insert_resource(Slideshow::starting(seed, SLIDES.len()));
    let flat = root_on(theme::BACKGROUND, JustifyContent::FlexStart);
    let root = commands.spawn((DespawnOnExit(Screen::Title), flat)).id();
    if images.is_some() {
        commands.entity(root).with_child(layer());
        commands.entity(root).with_child((layer(), Incoming));
    }
    commands.entity(root).with_child(version());
    menu.set(Menu::Main);
}

/// An empty layer of the slideshow.
fn layer() -> impl Bundle {
    let empty = Picture {
        slide: usize::MAX,
        aspect: 1.0,
    };
    let node = Node {
        position_type: PositionType::Absolute,
        ..default()
    };
    (empty, node, ImageNode::default(), UiTransform::IDENTITY)
}

/// The version label in the bottom-right corner, on a strip so it reads over any picture.
fn version() -> impl Bundle {
    let label = format!("v{}", env!("CARGO_PKG_VERSION"));
    (
        Node {
            position_type: PositionType::Absolute,
            right: px(theme::EDGE),
            bottom: px(theme::EDGE),
            padding: UiRect::axes(px(theme::S2), px(theme::S1)),
            ..default()
        },
        BackgroundColor(theme::PANEL),
        children![text(label, theme::FONT_SMALL, theme::TEXT_2)],
    )
}

/// Returns picture `slide` and its texture: the one `held` when it is that picture, else the
/// file decoded now. The texture lives in the renderer only, so no copy of it stays in memory.
/// A file that cannot be read gives the empty texture, once, and the background shows.
fn fetch(
    slide: usize,
    held: &Picture,
    texture: &Handle<Image>,
    images: &mut Assets<Image>,
) -> (Picture, Handle<Image>) {
    if held.slide == slide {
        return (*held, texture.clone());
    }
    let (kind, formats) = (ImageType::Extension("png"), CompressedImageFormats::NONE);
    let (sampler, usage) = (ImageSampler::linear(), RenderAssetUsages::RENDER_WORLD);
    let file = SLIDES[slide % SLIDES.len()];
    let read = Image::from_buffer(file, kind, formats, true, sampler, usage).ok();
    let shape = |image: &Image| image.width() as f32 / image.height().max(1) as f32;
    let aspect = read.as_ref().map_or(1.0, shape);
    let texture = read.map_or_else(|| ImageNode::default().image, |image| images.add(image));
    (Picture { slide, aspect }, texture)
}

/// Shows the slideshow on its two layers: the back one holds the picture shown and takes it from
/// the front one when the turn changes, the front one holds the next picture while it fades in.
/// A picture is decoded once, when its fade begins, and freed when no layer holds it: two are
/// in memory at most. Both drift by their scale, which costs no layout.
fn paint(
    show: Res<Slideshow>,
    mut images: ResMut<Assets<Image>>,
    mut back: Single<Layer, Without<Incoming>>,
    mut front: Single<Layer, With<Incoming>>,
) {
    let (back, front) = (&mut *back, &mut *front);
    if back.0.slide != show.slide {
        (*back.0, back.1.image) = fetch(show.slide, &front.0, &front.1.image, &mut images);
    }
    let next = show.next(SLIDES.len());
    if show.fading() && front.0.slide != next {
        (*front.0, front.1.image) = fetch(next, &back.0, &back.1.image, &mut images);
    }
    let ((_, incoming), (shown, entering)) = (show.weights(), show.zooms());
    back.2.scale = Vec2::splat(shown);
    if front.1.color.alpha() != incoming {
        front.1.color.set_alpha(incoming);
    }
    if show.fading() {
        front.2.scale = Vec2::splat(entering);
    }
}

/// Keeps each layer covering the window as it is resized, in shares of the window so the
/// interface scale does not move it.
fn fit(windows: Query<&Window, With<PrimaryWindow>>, mut pictures: Query<(&Picture, &mut Node)>) {
    let Some(size) = windows.iter().next().map(|w| w.size().max(Vec2::ONE)) else {
        return;
    };
    for (picture, mut node) in &mut pictures {
        let (shown, corner) = cover(size, picture.aspect);
        let (shown, corner) = (100.0 * shown / size, 100.0 * corner / size);
        let wanted = (
            percent(shown.x),
            percent(shown.y),
            percent(corner.x),
            percent(corner.y),
        );
        if (node.width, node.height, node.left, node.top) != wanted {
            (node.width, node.height, node.left, node.top) = wanted;
        }
    }
}

/// Registers the title screen systems. The slideshow runs on the title screen only, and its
/// state is removed with the screen.
pub(super) fn plugin(app: &mut App) {
    let pictures = (paint.run_if(resource_exists::<Assets<Image>>), fit).chain();
    let show = (advance, pictures).chain();
    app.add_systems(OnEnter(Screen::Title), enter)
        .add_systems(OnExit(Screen::Title), |mut commands: Commands| {
            commands.remove_resource::<Slideshow>()
        })
        .add_systems(
            Update,
            show.run_if(in_state(Screen::Title))
                .run_if(resource_exists::<Slideshow>),
        );
}
