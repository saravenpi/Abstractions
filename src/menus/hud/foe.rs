//! The animal fought: its name over a bar of its health, straight over the world above its head
//! while the fight lasts, as a marker over the world is placed. It follows the animal on screen,
//! hides when the animal is off screen or behind the camera, and never sits over the crosshair.

use bevy::camera::primitives::Aabb;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::onscreen::place;
use crate::avatar::strike::Mark;
use crate::fauna::{Beast, Part};
use crate::render::GameCamera;
use crate::settings::{Persistent, Settings};
use crate::ui::{text, theme};
use crate::world::Terrain;

/// Width of the health bar and its height, in pixels.
const BAR: (f32, f32) = (64.0, 4.0);

/// Height of the strip, the name over the bar, in pixels.
const STRIP: f32 = 28.0;

/// Pixels between the top of the head on screen and the bottom of the strip.
const LIFT: f32 = 8.0;

/// Metres above the top of the animal the strip is anchored at.
const HEAD: f32 = 0.15;

/// Half the side of the square around the crosshair the strip keeps out of, in pixels: past the
/// ticks at their widest.
const CLEAR: f32 = 28.0;

/// Marks the strip of the animal fought.
#[derive(Component)]
pub(super) struct Foe;

/// Marks the name of the animal fought.
#[derive(Component)]
pub(super) struct FoeName;

/// Marks the fill of the health bar of the animal fought.
#[derive(Component)]
pub(super) struct FoeBar;

/// Returns the strip of the animal fought, to hang from the centre of the screen: its name over its
/// health bar, with no panel, hidden.
pub(super) fn strip() -> impl Bundle {
    let node = Node {
        display: Display::None,
        position_type: PositionType::Absolute,
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        row_gap: px(theme::S1),
        width: px(2.0 * BAR.0),
        height: px(STRIP),
        ..default()
    };
    let track = Node {
        width: px(BAR.0),
        height: px(BAR.1),
        ..default()
    };
    let fill = (FoeBar, Node::default(), BackgroundColor(theme::TEXT));
    let name = text("", theme::FONT_SMALL, theme::TEXT);
    (
        (Foe, node),
        children![
            (FoeName, name, TextLayout::no_wrap()),
            (track, BackgroundColor(theme::HAIRLINE), children![fill]),
        ],
    )
}

/// Returns where the render position `head` shows on a `screen` of interface pixels seen by
/// `camera` with a vertical field of view of `fov` radians, or nothing when it is behind the camera
/// or off the screen.
pub fn over(head: Vec3, camera: &GlobalTransform, (fov, screen): (f32, Vec2)) -> Option<Vec2> {
    let dir = camera.affine().inverse().transform_point3(head);
    let whole = Rect::from_corners(Vec2::ZERO, screen);
    let placed = place::place(dir, fov, screen, whole);
    placed.arrow.is_none().then_some(placed.pos)
}

/// Returns the left and the bottom of the strip from the centre of a `screen` for a head at `pos`:
/// centred over it, [`LIFT`] above it, lifted clear of the crosshair, on whole steps of 4 pixels.
pub fn hung(pos: Vec2, screen: Vec2) -> (f32, f32) {
    let left = pos.x - screen.x / 2.0 - BAR.0;
    let bottom = screen.y / 2.0 - pos.y + LIFT;
    let across = left < CLEAR && left + 2.0 * BAR.0 > -CLEAR;
    let crossed = across && bottom < CLEAR && bottom + STRIP > -CLEAR;
    let bottom = if crossed { CLEAR } else { bottom };
    let step = |v: f32| (v / 4.0).round() * 4.0;
    (step(left), step(bottom))
}

/// The placed bounds of the parts of the animals, with the animal of each.
type Bounds<'w, 's> =
    Query<'w, 's, (&'static GlobalTransform, &'static Aabb, &'static ChildOf), With<Part>>;

/// Returns the top of the animal `root`, from the bounds of its parts among `parts`, raised by
/// [`HEAD`] along its up.
fn top(root: (Entity, &GlobalTransform), parts: &Bounds) -> Vec3 {
    let (animal, placed) = root;
    let (at, up) = (placed.translation(), placed.up().as_vec3());
    let height = parts
        .iter()
        .filter(|(_, _, of)| of.parent() == animal)
        .map(|(part, aabb, _)| {
            let centre = part.transform_point(Vec3::from(aabb.center));
            let reach = (Vec3::from(aabb.half_extents) * part.scale().abs()).max_element();
            up.dot(centre - at) + reach
        })
        .fold(0.0_f32, f32::max);
    at + up * (height + HEAD)
}

/// What the strip reads of the screen: the camera, the window, the scale of the interface and the
/// field of view in the settings.
type Lens<'w, 's> = (
    Query<'w, 's, &'static GlobalTransform, With<GameCamera>>,
    Query<'w, 's, &'static Window, With<PrimaryWindow>>,
    Option<Res<'w, UiScale>>,
    Res<'w, Persistent<Settings>>,
);

/// Returns where the head of `root` shows through `lens`, with the screen in interface pixels.
fn seen(root: (Entity, &GlobalTransform), parts: &Bounds, lens: &Lens) -> Option<(Vec2, Vec2)> {
    let (camera, window, scale, settings) = lens;
    let (camera, window) = (camera.single().ok()?, window.single().ok()?);
    let scale = scale.as_ref().map_or(1.0, |scale| scale.0).max(0.1);
    let screen = Vec2::new(window.width(), window.height()) / scale;
    let fov = (settings.graphics.fov as f32).to_radians();
    over(top(root, parts), camera, (fov, screen)).map(|pos| (pos, screen))
}

/// The strip, the fill of its bar and its name.
type Strips<'w, 's> = (
    Query<'w, 's, &'static mut Node, (With<Foe>, Without<FoeBar>)>,
    Query<'w, 's, &'static mut Node, (With<FoeBar>, Without<Foe>)>,
    Query<'w, 's, &'static mut Text, With<FoeName>>,
);

/// The last blow, the animals and the terrain.
type Struck<'w, 's> = (
    Res<'w, Mark>,
    Query<'w, 's, (&'static Beast, &'static GlobalTransform)>,
    Option<Res<'w, Terrain>>,
);

/// Shows the name and the health of the animal the last blow struck over its head while the fight
/// lasts and it is on screen.
pub(super) fn show(
    (mark, beasts, terrain): Struck,
    parts: Bounds,
    lens: Lens,
    (mut strips, mut bars, mut names): Strips,
) {
    let foe = mark
        .victim
        .and_then(|victim| Some((victim, beasts.get(victim).ok()?)));
    let life = terrain.as_deref().and_then(|t| t.globe().life());
    let named = foe.and_then(|(victim, (beast, placed))| {
        let name = &life?.species.get(usize::from(beast.index))?.name;
        Some((beast, name, seen((victim, placed), &parts, &lens)?))
    });
    for mut node in &mut strips {
        let (display, (left, bottom)) = match named {
            Some((_, _, (pos, screen))) => (Display::Flex, hung(pos, screen)),
            None => (Display::None, (0.0, 0.0)),
        };
        (node.display, node.left, node.bottom) = (display, px(left), px(bottom));
    }
    let Some((beast, name, _)) = named else {
        return;
    };
    for mut bar in &mut bars {
        bar.width = percent(100.0 * beast.wits.health.share());
    }
    for mut text in &mut names {
        if text.0 != *name {
            text.0.clone_from(name);
        }
    }
}
