//! The reticle: four ticks around the crosshair while a tool is drawn, closing in and lit when an
//! animal is within its reach; a hit mark at the four corners after a blow, bright for a clean one
//! and dim for a glance; and under it, while the fight lasts, the name of the animal and how much
//! health it has left. A swing that meets nothing shows nothing: its whoosh says it.

use bevy::prelude::*;

use super::context::Element;
use super::fade::gate;
use crate::avatar::strike::{Mark, Outcome, Sight};
use crate::fauna::Beast;
use crate::player::Squared;
use crate::ui::{text, theme};
use crate::world::Terrain;

/// Thickness and length of a tick, in pixels.
const TICK: (f32, f32) = (2.0, 8.0);

/// Pixels from the centre to a tick with nothing in reach, and with an animal in reach.
const GAP: (f32, f32) = (16.0, 8.0);

/// Side of a corner of the hit mark, and its pixels from the centre along each axis.
const CORNER: (f32, f32) = (4.0, 12.0);

/// Width of the health bar of the animal fought, its height, and how far under the centre its
/// strip sits, in pixels.
const BAR: (f32, f32, f32) = (64.0, 4.0, 48.0);

/// Marks tick `0` up, `1` right, `2` down, `3` left.
#[derive(Component)]
pub(super) struct Tick(usize);

/// Marks a corner of the hit mark.
#[derive(Component)]
pub(super) struct Corner;

/// Marks the strip of the animal fought.
#[derive(Component)]
pub(super) struct Foe;

/// Marks the name of the animal fought.
#[derive(Component)]
pub(super) struct FoeName;

/// Marks the fill of the health bar of the animal fought.
#[derive(Component)]
pub(super) struct FoeBar;

/// Returns the left, top, width and height of tick `k` at `gap` pixels from the centre.
pub fn tick(k: usize, gap: f32) -> [f32; 4] {
    let (thin, long) = TICK;
    match k {
        0 => [-thin / 2.0, -gap - long, thin, long],
        1 => [gap, -thin / 2.0, long, thin],
        2 => [-thin / 2.0, gap, thin, long],
        _ => [-gap - long, -thin / 2.0, long, thin],
    }
}

/// Returns a node at `left`, `top` from the centre of `width` by `height` pixels.
fn at([left, top, width, height]: [f32; 4]) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: px(left),
        top: px(top),
        width: px(width),
        height: px(height),
        ..default()
    }
}

/// Returns the strip of the animal fought: its name over its health bar, straight over the world
/// with no panel, hidden.
fn foe() -> impl Bundle {
    let strip = Node {
        display: Display::None,
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        row_gap: px(theme::S1),
        height: Val::Auto,
        ..at([-BAR.0, BAR.2, 2.0 * BAR.0, 0.0])
    };
    let track = Node {
        width: px(BAR.0),
        height: px(BAR.1),
        ..default()
    };
    let fill = (FoeBar, Node::default(), BackgroundColor(theme::TEXT));
    let name = text("", theme::FONT_SMALL, theme::TEXT);
    (
        (Foe, strip),
        children![
            (FoeName, name, TextLayout::no_wrap()),
            (track, BackgroundColor(theme::HAIRLINE), children![fill]),
        ],
    )
}

/// Returns the reticle: a point at the centre of the screen holding the ticks, the corners of the
/// hit mark and the strip of the animal fought.
pub(super) fn layer() -> impl Bundle {
    let centre = Node {
        position_type: PositionType::Absolute,
        left: percent(50),
        top: percent(50),
        width: px(0),
        height: px(0),
        ..default()
    };
    let hidden = |k: usize| Node {
        display: Display::None,
        ..at(tick(k, GAP.0))
    };
    let ticks = (0..4).map(move |k| (Tick(k), hidden(k), BackgroundColor(theme::TEXT_2)));
    let (side, out) = CORNER;
    let corners = [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)]
        .into_iter()
        .map(move |(x, y)| {
            let place = [x * out - side / 2.0, y * out - side / 2.0, side, side];
            (Corner, at(place), BackgroundColor(Color::NONE))
        });
    (
        gate(Element::Reticle),
        centre,
        Pickable::IGNORE,
        Children::spawn((SpawnIter(ticks), SpawnIter(corners), Spawn(foe()))),
    )
}

/// Shows the ticks while a tool is drawn, closed in and lit when an animal is within its reach.
pub(super) fn aim(
    (sight, squared): (Res<Sight>, Res<Squared>),
    mut ticks: Query<(&Tick, &mut Node, &mut BackgroundColor)>,
) {
    let gap = [GAP.0, GAP.1][usize::from(sight.reach)];
    let tone = [theme::TEXT_2, theme::TEXT][usize::from(sight.reach)];
    let display = [Display::None, Display::Flex][usize::from(squared.drawn)];
    for (tick_of, mut node, mut back) in &mut ticks {
        let place = at(tick(tick_of.0, gap));
        if node.left != place.left || node.top != place.top || node.display != display {
            *node = Node { display, ..place };
        }
        back.set_if_neq(BackgroundColor(tone));
    }
}

/// Returns the colour of the hit mark of a blow that came to `outcome`, faded to `left` of it.
pub fn marked(outcome: Outcome, left: f32) -> Color {
    let whole = match outcome {
        Outcome::Clean | Outcome::Stopped => theme::TEXT,
        Outcome::Deflected => theme::TEXT_2,
        Outcome::Miss => Color::NONE,
    };
    whole.with_alpha(whole.alpha() * left.clamp(0.0, 1.0))
}

/// The corners of the hit mark, apart from the strip of the struck animal.
type Corners = (With<Corner>, Without<FoeBar>, Without<Foe>);

/// Shows the hit mark of the last blow at the corners, and the name and health of the animal it
/// struck under the crosshair while the fight lasts.
pub(super) fn strike(
    (mark, beasts, terrain): (Res<Mark>, Query<&Beast>, Option<Res<Terrain>>),
    mut corners: Query<&mut BackgroundColor, Corners>,
    mut strips: Query<&mut Node, (With<Foe>, Without<FoeBar>)>,
    mut bars: Query<&mut Node, (With<FoeBar>, Without<Foe>)>,
    mut names: Query<&mut Text, With<FoeName>>,
) {
    let tone = mark
        .shown()
        .map_or(Color::NONE, |(o, left)| marked(o, left));
    for mut back in &mut corners {
        back.set_if_neq(BackgroundColor(tone));
    }
    let foe = mark.victim.and_then(|victim| beasts.get(victim).ok());
    let life = terrain.as_deref().and_then(|t| t.globe().life());
    let named =
        foe.and_then(|beast| Some((beast, &life?.species.get(usize::from(beast.index))?.name)));
    let display = [Display::None, Display::Flex][usize::from(named.is_some())];
    for mut strip in &mut strips {
        strip.display = display;
    }
    let Some((beast, name)) = named else { return };
    for mut bar in &mut bars {
        bar.width = percent(100.0 * beast.wits.health.share());
    }
    for mut text in &mut names {
        if text.0 != *name {
            text.0.clone_from(name);
        }
    }
}

/// Registers the reticle: its ticks and its marks follow the sight and the blows while the player
/// controls the game.
pub(super) fn plugin(app: &mut App) {
    let aimed =
        resource_exists_and_changed::<Sight>.or_else(resource_exists_and_changed::<Squared>);
    app.init_resource::<Sight>()
        .init_resource::<Mark>()
        .init_resource::<Squared>()
        .add_systems(
            Update,
            (
                aim.run_if(aimed),
                strike.run_if(resource_exists_and_changed::<Mark>),
            )
                .in_set(super::Step::Playing),
        );
}
