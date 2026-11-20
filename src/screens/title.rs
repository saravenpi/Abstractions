//! Title screen: a picture of the robot on the home planet filling the window behind the main menu, and
//! the version label under it.

use bevy::asset::RenderAssetUsages;
use bevy::image::{CompressedImageFormats, ImageSampler, ImageType};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::Screen;
use crate::menus::Menu;
use crate::ui::{root_on, text, theme};

/// The title picture, the hero image of the website at half size in 256 colours.
const PICTURE: &[u8] = include_bytes!("../../assets/images/title.png");

/// Width over height of [`PICTURE`].
const ASPECT: f32 = 1120.0 / 521.0;

/// Share of the height the picture loses at its top when the window is wider than it, as the
/// website frames it: the horizon and the robot stay in view.
const FOCUS: f32 = 0.62;

/// Marks the picture behind the main menu.
#[derive(Component)]
pub struct Picture;

/// Returns the size and the top-left corner, in pixels, of a picture of width over height
/// `aspect` that covers a window of `size`: as small as it can be while it fills the window,
/// centred across and framed at [`FOCUS`] down.
pub fn cover(size: Vec2, aspect: f32) -> (Vec2, Vec2) {
    let width = size.x.max(size.y * aspect);
    let shown = Vec2::new(width, width / aspect);
    let corner = (size - shown) * Vec2::new(0.5, FOCUS);
    (shown, corner)
}

/// Spawns the picture under the version label and opens the main menu. Without a renderer the
/// picture cannot be read and the background stays flat.
fn enter(
    mut commands: Commands,
    mut images: Option<ResMut<Assets<Image>>>,
    mut menu: ResMut<NextState<Menu>>,
) {
    let usage = RenderAssetUsages::default();
    let (kind, formats) = (ImageType::Extension("png"), CompressedImageFormats::NONE);
    let read = Image::from_buffer(PICTURE, kind, formats, true, ImageSampler::nearest(), usage);
    let handle = images
        .as_mut()
        .zip(read.ok())
        .map(|(images, image)| images.add(image));
    let root = commands
        .spawn((
            DespawnOnExit(Screen::Title),
            root_on(theme::BACKGROUND, JustifyContent::FlexStart),
        ))
        .id();
    if let Some(handle) = handle {
        let node = Node {
            position_type: PositionType::Absolute,
            ..default()
        };
        commands
            .entity(root)
            .with_child((Picture, node, ImageNode::new(handle)));
    }
    commands.entity(root).with_child(version());
    menu.set(Menu::Main);
}

/// The version label in the bottom-right corner.
fn version() -> impl Bundle {
    (
        text(
            format!("v{}", env!("CARGO_PKG_VERSION")),
            theme::FONT_SMALL,
            theme::MUTED,
        ),
        Node {
            position_type: PositionType::Absolute,
            right: px(theme::EDGE),
            bottom: px(theme::EDGE),
            ..default()
        },
    )
}

/// Keeps the picture covering the window as it is resized.
fn fit(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut pictures: Query<&mut Node, With<Picture>>,
) {
    let Some(window) = windows.iter().next() else {
        return;
    };
    let (shown, corner) = cover(window.size(), ASPECT);
    let wanted = (px(shown.x), px(shown.y), px(corner.x), px(corner.y));
    for mut node in &mut pictures {
        if (node.width, node.height, node.left, node.top) != wanted {
            (node.width, node.height, node.left, node.top) = wanted;
        }
    }
}

/// Registers the title screen systems.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Title), enter)
        .add_systems(Update, fit.run_if(in_state(Screen::Title)));
}
