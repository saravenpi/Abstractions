//! The name of each other player above their robot: white on black text of the interface,
//! moved each frame to where the head is on screen, shown within [`TAG_RANGE`] metres.
//!
//! The game has no floating label of its own; this is one interface node per robot, placed
//! from the game camera, whose small image is stretched over the window.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::Remote;
use super::track::Remotes;
use crate::net::Players;
use crate::render::GameCamera;
use crate::screens::Screen;
use crate::sim::PlayerId;
use crate::ui::{text, theme};

/// Metres from the camera within which a name is shown.
pub const TAG_RANGE: f32 = 200.0;

/// Metres above the centre of the body where the name floats.
const ABOVE: f32 = 1.4;

/// Pixels between the name and the edge of its black box.
const PAD: f32 = 4.0;

/// Marks the name above a robot.
#[derive(Component, Debug, Clone, Copy)]
pub struct Tag;

/// Returns the name shown for `player`.
fn name(players: &Players, player: PlayerId) -> String {
    let listed = players.0.iter().find(|(id, _)| *id == player.0);
    listed.map_or(format!("Player {}", player.0), |(_, name)| name.clone())
}

/// Returns the name node of `name`, hidden until it is placed.
fn label(name: String) -> impl Bundle {
    let node = Node {
        position_type: PositionType::Absolute,
        padding: UiRect::axes(px(PAD), px(PAD * 0.5)),
        ..default()
    };
    let inside = (
        text(name, theme::FONT_SMALL, theme::TEXT),
        TextLayout::no_wrap(),
    );
    let shown = (Visibility::Hidden, DespawnOnEnter(Screen::Title));
    (Tag, node, BackgroundColor(theme::INK), inside, shown)
}

/// Filter of the puppets a name is placed over.
type Seen = (With<Remote>, Without<Tag>);

/// Where the names are placed from.
#[derive(SystemParam)]
pub struct Lens<'w, 's> {
    /// The game camera.
    camera: Query<'w, 's, (&'static Camera, &'static GlobalTransform), With<GameCamera>>,
    /// The window the interface fills.
    window: Query<'w, 's, &'static Window, With<PrimaryWindow>>,
    /// The robot puppets.
    puppets: Query<'w, 's, (&'static GlobalTransform, &'static Visibility), Seen>,
}

impl Lens<'_, '_> {
    /// Returns where on the window the name of the robot `puppet` goes, or nothing when it is
    /// hidden, too far, behind the camera or there is no camera.
    fn spot(&self, puppet: Entity) -> Option<Vec2> {
        let (body, shown) = self.puppets.get(puppet).ok()?;
        let (camera, eye) = self.camera.single().ok()?;
        let head = body.translation() + body.up() * ABOVE;
        let near = head.distance(eye.translation()) <= TAG_RANGE;
        let point = camera
            .world_to_viewport(eye, head)
            .ok()
            .filter(|_| near && *shown != Visibility::Hidden)?;
        let window = self.window.single().ok()?;
        let target = camera.logical_viewport_size()?;
        Some(point * Vec2::new(window.width(), window.height()) / target)
    }
}

/// Shows the name of each other player above their robot, and hides it when it is too far.
pub fn tags(
    mut commands: Commands,
    mut remotes: ResMut<Remotes>,
    players: Res<Players>,
    lens: Lens,
    mut nodes: Query<(&mut Node, &mut Visibility, &Text), With<Tag>>,
) {
    for (player, track) in remotes.robots.iter_mut() {
        let Some(puppet) = track.puppet else {
            continue;
        };
        let tag = *track
            .tag
            .get_or_insert_with(|| commands.spawn(label(name(&players, *player))).id());
        let Ok((mut node, mut shown, words)) = nodes.get_mut(tag) else {
            continue;
        };
        let Some(spot) = lens.spot(puppet) else {
            shown.set_if_neq(Visibility::Hidden);
            continue;
        };
        let width =
            words.0.chars().count() as f32 * theme::FONT_SMALL * theme::GLYPH_ASPECT + 2.0 * PAD;
        node.left = px(spot.x - width * 0.5);
        node.top = px(spot.y - theme::FONT_SMALL - PAD);
        shown.set_if_neq(Visibility::Inherited);
    }
}
