//! The animals in view: the name of each over a bar of its health, straight over the world above
//! its head. Every living animal on screen within [`FAR`] metres carries one from a pool of
//! [`SLOTS`] strips, bright up close and fading out with distance, the animal fought and the one
//! looked at brightest. A strip follows its animal on screen, hides when it is off screen or behind
//! the camera, never sits over the crosshair, and when a herd stands bunched the names of the
//! farther ones step aside while their bars stay.
//!
//! The cost of a frame: one distance check per living animal, then for each of at most [`SLOTS`]
//! in range the bounds of its own parts (about ten), one projection and a check of its name
//! against the names laid before (at most 48 × 48 rectangle tests), then one pass over each of the
//! four kinds of nodes of the pool. Under 0.05 ms by our count; nothing is allocated once the
//! lists have grown to the herd.

mod boss;
mod boss_bar;
mod draw;
mod lay;
mod pick;
mod place;
#[cfg(test)]
mod tests_lay;
#[cfg(test)]
mod tests_pick;
#[cfg(test)]
mod tests_shown;
#[cfg(test)]
mod tests_track;

pub use lay::{FAR, Found, Label, lay};
pub use pick::looked;
#[cfg(test)]
pub use place::hung;
pub use place::over;

use bevy::prelude::*;

use super::context::{Element, Where};
use super::fade::gate;
use super::onscreen::place::Frustum;
use crate::avatar::strike::Mark;
use crate::fauna::{Beast, Live};
use crate::space::{Anchor, Placed};
use crate::ui::{OverWorld, text, theme};
use crate::world::Terrain;
use draw::{Strips, write};
use place::{Bounds, Lens, rooted, sight, top};

/// Width of the health bar and its height, in pixels.
const BAR: (f32, f32) = (64.0, 4.0);

/// Height of the strip, the name over the bar, in pixels.
const STRIP: f32 = 28.0;

/// Strips in the pool: more than the animals drawn near the robot at once.
pub const SLOTS: usize = 48;

/// Marks the strip of slot `0`.
#[derive(Component)]
pub(super) struct Foe(usize);

/// Marks the name in the strip of slot `0`.
#[derive(Component)]
pub(super) struct FoeName(usize);

/// Marks the track of the health bar in the strip of slot `0`.
#[derive(Component)]
pub(super) struct FoeTrack(usize);

/// Marks the fill of the health bar in the strip of slot `0`.
#[derive(Component)]
pub(super) struct FoeBar(usize);

/// Returns the layer of the strips: its own root over the whole screen, gated by
/// [`Element::Labels`] and not by the reticle, so it shows on foot with or without a tool, and a
/// point at the centre of the screen the strips hang from.
pub(super) fn layer() -> impl Bundle {
    let whole = Node {
        position_type: PositionType::Absolute,
        width: percent(100),
        height: percent(100),
        ..default()
    };
    let centre = Node {
        position_type: PositionType::Absolute,
        left: percent(50),
        top: percent(50),
        width: px(0),
        height: px(0),
        ..default()
    };
    let strips = (0..SLOTS).map(strip);
    (
        (gate(Element::Labels), OverWorld, whole, Pickable::IGNORE),
        children![
            (centre, Pickable::IGNORE, Children::spawn(SpawnIter(strips))),
            boss_bar::bar()
        ],
    )
}

/// Returns the strip of slot `slot`, to hang from the centre of the screen: a name over a health
/// bar, with no panel, hidden.
fn strip(slot: usize) -> impl Bundle {
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
    let fill = (FoeBar(slot), Node::default(), BackgroundColor(theme::TEXT));
    let name = text("", theme::FONT_SMALL, theme::TEXT);
    let track = (FoeTrack(slot), track, BackgroundColor(theme::HAIRLINE));
    (
        (Foe(slot), node),
        children![
            (FoeName(slot), name, TextLayout::no_wrap()),
            (track, children![fill])
        ],
    )
}

/// The lists a frame of labels reuses: the animals found in view, the names laid and the label of
/// each strip.
#[derive(Default)]
pub(super) struct Scratch {
    /// The animals in view.
    found: Vec<Found>,
    /// The rectangles of the names laid.
    taken: Vec<Rect>,
    /// What each strip of the pool shows.
    labels: Vec<Option<Label>>,
}

/// The living animals with their pose and their render place.
type Animals<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Beast,
        &'static Transform,
        Option<&'static Placed>,
    ),
    Live,
>;

/// What the labels read of the animals: the living ones, the parts of their bodies and where
/// render space sits.
type Herd<'a, 'w, 's> = (&'a Animals<'w, 's>, &'a Bounds<'w, 's>, Option<&'a Anchor>);

/// Fills `found` with every animal of the herd within [`FAR`] metres that shows on screen through
/// `sight`, its head from this frame's pose of its root and its parts, the one `fought` and the one
/// looked at marked keen.
fn gather(
    found: &mut Vec<Found>,
    sight: (&GlobalTransform, Vec2, Frustum),
    (beasts, parts, anchor): Herd,
    fought: Option<Entity>,
) {
    let (camera, screen, frustum) = sight;
    let (eye, dir) = (camera.translation(), camera.forward().as_vec3());
    let heads = || {
        beasts
            .iter()
            .filter_map(move |(animal, beast, transform, placed)| {
                let root = rooted(anchor, transform, placed);
                let (foot, head) = top(&root, beast.motion.root.place, &beast.parts, parts);
                (foot.distance(eye) <= FAR).then_some((animal, beast, foot, head))
            })
    };
    let keen = looked(
        heads().map(|(a, _, foot, head)| (a, (foot + head) / 2.0)),
        (eye, dir),
    );
    found.clear();
    found.extend(heads().filter_map(|(animal, beast, _, head)| {
        Some(Found {
            species: beast.index,
            health: beast.wits.health.share(),
            pos: over(head, camera, (frustum, screen))?,
            distance: eye.distance(head),
            keen: Some(animal) == keen || Some(animal) == fought,
        })
    }));
}

/// The last blow, the animals, the terrain and the anchor the labels read.
type Struck<'w, 's> = (
    Res<'w, Mark>,
    Animals<'w, 's>,
    Option<Res<'w, Terrain>>,
    Option<Res<'w, Anchor>>,
);

/// Labels every living animal in view within [`FAR`] metres over its head.
pub(super) fn show(
    (mark, beasts, terrain, anchor): Struck,
    (parts, lens): (Bounds, Lens),
    mut strips: Strips,
    mut scratch: Local<Scratch>,
) {
    let Scratch {
        found,
        taken,
        labels,
    } = &mut *scratch;
    labels.resize(SLOTS, None);
    match sight(&lens) {
        Some((camera, screen, frustum)) => {
            gather(
                found,
                (&camera, screen, frustum),
                (&beasts, &parts, anchor.as_deref()),
                mark.victim,
            );
            lay(found, screen, labels, taken);
        }
        None => labels.iter_mut().for_each(|label| *label = None),
    }
    let life = terrain
        .as_deref()
        .and_then(|terrain| terrain.globe().life());
    write(&mut strips, labels, life);
}

/// Run condition: animal labels mean something where the player is.
fn enabled(now: Res<Where>) -> bool {
    now.has(Element::Labels)
}

/// Registers the strips: they follow the animals every frame while the player controls the game,
/// after the eye of the HUD is captured.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (show, boss::update)
            .after(super::eye::capture)
            .after(crate::fauna::Posed)
            .run_if(enabled)
            .in_set(super::Step::Playing),
    );
}
