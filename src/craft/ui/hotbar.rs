//! The hotbar: six slots at the bottom centre of the game, black and white, the held one
//! lit and marked by the accent bar along its foot, each with its number, the silhouette of its
//! tool and a thin wear bar. The slots keep a very light fill, so a full slot and an empty one
//! read apart; the numbers carry the HUD shadow.

use bevy::prelude::*;

use super::glyph::{silhouette, wear_bar};
use super::here::Here;
use crate::craft::pack::{Pack, SLOTS};
use crate::screens::{Screen, in_control};
use crate::space::on_foot;
use crate::ui::{OverWorld, text, theme};

/// Side in pixels of one slot, which is also the height of the hotbar the prompt line stands above.
pub const SIDE: f32 = 56.0;

/// Height in pixels of the accent bar that marks the held slot.
const MARK: f32 = theme::S1;

/// Width in pixels of the wear bar of a slot.
const BAR: f32 = 32.0;

/// Marks the row of slots.
#[derive(Component)]
struct Hotbar;

/// Spawns the empty hotbar for the world being entered.
fn spawn(mut commands: Commands) {
    let node = Node {
        position_type: PositionType::Absolute,
        bottom: px(theme::EDGE),
        width: percent(100),
        justify_content: JustifyContent::Center,
        column_gap: px(theme::S2),
        ..default()
    };
    commands.spawn((
        Hotbar,
        OverWorld,
        DespawnOnExit(Screen::Gameplay),
        node,
        Pickable::IGNORE,
        Visibility::Hidden,
    ));
}

/// The silhouette of a tool: shaft length, head size and form, if its build is whole.
type Shape = Option<(f32, f32, crate::craft::build::Form)>;

/// Returns the bar along the foot of a slot: the accent while `held`, else clear.
fn accent(held: bool) -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            bottom: px(0),
            width: percent(100),
            height: px(MARK),
            ..default()
        },
        BackgroundColor([Color::NONE, theme::ACCENT][usize::from(held)]),
    )
}

/// Returns one slot: its number, the silhouette of its tool and the wear left, lit when `held`.
fn slot(at: usize, tool: Option<(Shape, f32)>, held: bool) -> impl Bundle {
    let node = Node {
        width: px(SIDE),
        height: px(SIDE),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::SpaceBetween,
        padding: UiRect::axes(px(theme::S1), px(theme::S2)),
        ..default()
    };
    let back = match tool.is_some() {
        true => theme::SLOT_FULL,
        false => theme::SLOT_EMPTY,
    };
    let ink = [theme::TEXT_2, theme::TEXT][usize::from(held)];
    let (shape, worn) = tool.unwrap_or((None, 1.0));
    let drawn = match tool.is_some() {
        true => Visibility::Inherited,
        false => Visibility::Hidden,
    };
    (
        node,
        BackgroundColor(back),
        children![
            text(format!("{}", at + 1), theme::FONT_SMALL, ink),
            (silhouette(shape), drawn),
            (wear_bar(BAR, worn), drawn),
            accent(held),
        ],
    )
}

/// Draws the slots again when the pack changes.
fn fill(pack: Res<Pack>, here: Here, bars: Query<Entity, With<Hotbar>>, mut commands: Commands) {
    let place = here.place();
    let tool = |at: usize| {
        let item = pack.bar.slots[at].and_then(|id| pack.item(id))?;
        Some((item.shape(), item.worn(&place).0))
    };
    for bar in &bars {
        commands.entity(bar).despawn_children();
        let slots: Vec<_> = (0..SLOTS)
            .map(|at| slot(at, tool(at), at == pack.bar.held))
            .collect();
        commands.entity(bar).with_children(|parent| {
            for one in slots {
                parent.spawn(one);
            }
        });
    }
}

/// Shows the hotbar while the player controls the robot on foot.
fn show(
    In(control): In<bool>,
    foot: Option<Res<crate::space::Location>>,
    mut bars: Query<&mut Visibility, With<Hotbar>>,
) {
    let walking = on_foot(foot);
    let wanted = match control && walking {
        true => Visibility::Inherited,
        false => Visibility::Hidden,
    };
    for mut visibility in &mut bars {
        visibility.set_if_neq(wanted);
    }
}

/// Registers the hotbar, its number keys and the held tool.
pub(crate) fn plugin(app: &mut App) {
    let drawn = resource_changed::<Pack>.or_else(any_match_filter::<Added<Hotbar>>);
    app.add_systems(OnEnter(Screen::Gameplay), spawn)
        .add_systems(
            Update,
            (
                in_control.pipe(show),
                crate::craft::wield::choose.run_if(in_control.and_then(on_foot)),
                fill.run_if(drawn),
            )
                .chain()
                .run_if(in_state(Screen::Gameplay)),
        );
}
