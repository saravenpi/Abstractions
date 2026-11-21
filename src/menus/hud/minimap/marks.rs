//! Laying the ground view out every frame: the image is sized, moved and turned, the marks
//! slide to their places, and what has nothing to show is hidden.

use bevy::prelude::*;
use bevy::ui::Val2;

use super::patch::Patch;
use super::pose::{Pose, Reader};
use super::range::{Mode, range};
use super::scope::Scope;
use super::sight::{Sight, sight};
use super::widget::{Caption, Part};
use crate::computer::distance;
use crate::home::Base;

/// Shows or hides `node` where it is.
fn show(node: &mut Mut<Node>, shown: bool) {
    let display = [Display::None, Display::Flex][usize::from(shown)];
    if node.display != display {
        node.display = display;
    }
}

/// Shows or hides `node`, and moves it to the map point `at` when it has one.
pub fn put(node: &mut Mut<Node>, at: Option<Vec2>) {
    let display = at.map_or(Display::None, |_| Display::Flex);
    let share = at.map(|at| (Vec2::new(at.x, -at.y) * 0.5 + 0.5) * 100.0);
    let (left, top) = share.map_or((node.left, node.top), |s| (percent(s.x), percent(s.y)));
    if (node.display, node.left, node.top) != (display, left, top) {
        (node.display, node.left, node.top) = (display, left, top);
    }
}

/// Sizes the image of the ground so that it shows what `sight` asks for, or hides it.
fn spread(node: &mut Mut<Node>, sight: Option<&Sight>) {
    let Some(fit) = sight.map(|sight| sight.fit) else {
        show(node, false);
        return;
    };
    let (side, corner) = (percent(fit.side), percent(50.0 - 0.5 * fit.side));
    let wanted = (Display::Flex, corner, corner, side, side);
    if (node.display, node.left, node.top, node.width, node.height) != wanted {
        (node.display, node.left, node.top, node.width, node.height) = wanted;
    }
}

/// Returns the move and the turn of the image of the ground for `sight`.
fn turned(sight: Option<&Sight>) -> UiTransform {
    let Some(fit) = sight.map(|sight| sight.fit) else {
        return UiTransform::IDENTITY;
    };
    UiTransform {
        translation: Val2::percent(fit.shift.x, fit.shift.y),
        rotation: Rot2::radians(fit.turn),
        ..UiTransform::IDENTITY
    }
}

/// Returns the player on the planet and the patch the marks are placed through, or nothing
/// in space or before the map has a range. Before the first patch is painted the marks are
/// placed on a patch around the player.
pub fn lens(reader: &Reader, scope: &Scope) -> Option<(Pose, Patch)> {
    let (pose, globe) = reader.pose().zip(reader.globe())?;
    let around = || Patch::around(pose.at.normalize(), globe.radius(), scope.radius);
    let patch = scope.patch.unwrap_or_else(around);
    (scope.radius > 0.0).then_some((pose, patch))
}

/// Returns the ground view of this frame, or nothing in space.
fn seen(reader: &Reader, scope: &Scope, base: Option<&Base>) -> Option<Sight> {
    let (pose, patch) = lens(reader, scope)?;
    let home = base.filter(|base| base.body == pose.body);
    Some(sight(&pose, &patch, scope.radius, home.map(Base::home)))
}

/// Places every part of the ground view: the frame hides while the map is off, the image
/// follows the player and turns with the heading, and the marks sit where their targets lie.
pub fn place(
    reader: Reader,
    scope: Res<Scope>,
    base: Option<Res<Base>>,
    mut parts: Query<(&Part, &mut Node, Option<&mut UiTransform>)>,
) {
    let sight = seen(&reader, &scope, base.as_deref());
    let middle = sight.map(|_| Vec2::ZERO);
    for (part, mut node, turn) in &mut parts {
        match part {
            Part::Frame => show(&mut node, scope.mode != Mode::Off),
            Part::Ground => spread(&mut node, sight.as_ref().filter(|_| scope.patch.is_some())),
            Part::Arrow => put(&mut node, middle),
            Part::Home => put(&mut node, sight.and_then(|sight| sight.home)),
            Part::Ship => put(&mut node, sight.and_then(|sight| sight.ship)),
            Part::North => put(&mut node, sight.map(|sight| sight.north)),
        }
        if let Some(mut turn) = turn.filter(|_| *part == Part::Ground) {
            turn.set_if_neq(turned(sight.as_ref()));
        }
    }
}

/// Writes the range of the ground view in the corner when it changes.
pub fn caption(
    reader: Reader,
    scope: Res<Scope>,
    mut captions: Query<&mut Text, With<Caption>>,
    mut last: Local<Option<f32>>,
) {
    let globe = reader.pose().and(reader.globe());
    let shown = globe.and_then(|globe| range(scope.mode, scope.step, globe.radius()));
    if *last == shown {
        return;
    }
    *last = shown;
    for mut text in captions.iter_mut().filter(|_| shown.is_some()) {
        text.0 = distance(f64::from(shown.unwrap_or_default()));
    }
}
