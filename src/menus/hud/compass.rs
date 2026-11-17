//! The compass: a thin bar at the top of the screen with a mark for the home base and one for
//! the ship, each where it lies around the player, with its distance under it.
//!
//! A mark slides along the bar as the player turns and waits at the end of the bar while its
//! target is behind, so the bar always says which way to turn. It only shows on the planet the
//! target stands on.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::needle::{WIDTH, bearing, beside, slot};
use crate::computer::distance;
use crate::home::Base;
use crate::render::GameCamera;
use crate::ship::Ship;
use crate::space::{Anchor, Frame, Location, Travel, up};
use crate::ui::{Icon, Tone, icon, text, theme};

/// Width in pixels of the room of one mark, which is centred on its place on the bar.
const ROOM: f32 = 96.0;

/// Distance in metres under which a target is here and its mark is hidden.
const HERE: f32 = 12.0;

/// What a mark of the compass points at.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Mark {
    /// The home base.
    Home,
    /// The ship, while the player is on foot.
    Ship,
}

/// Marks the bar itself, which hides while it has nothing to point at.
#[derive(Component)]
pub(super) struct Bar;

/// Marks the text under a mark that gives the distance to its target.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Far(Mark);

/// One mark: its icon over its distance, hidden until it has a target.
fn mark(mark: Mark, symbol: Icon) -> impl Bundle {
    (
        mark,
        Node {
            display: Display::None,
            position_type: PositionType::Absolute,
            width: px(ROOM),
            top: px(0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            ..default()
        },
        children![
            icon(symbol, Tone::Plain),
            (Far(mark), text("", theme::FONT_SMALL, theme::MUTED)),
        ],
    )
}

/// The line of the bar, or the tick at its middle: a rule `size` pixels wide and high, `left`
/// percent along the bar.
fn rule(left: f32, size: Vec2, color: Color) -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            left: percent(left),
            top: px(theme::ICON + theme::GAP_XS),
            width: px(size.x),
            height: px(size.y),
            ..default()
        },
        BackgroundColor(color),
    )
}

/// The compass, high in the middle of the screen.
pub(super) fn bar() -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            top: px(theme::GAP_M),
            justify_content: JustifyContent::Center,
            ..default()
        },
        children![(
            Bar,
            Visibility::Hidden,
            Node {
                width: px(WIDTH),
                height: px(theme::ICON * 2.0 + theme::GAP_M),
                ..default()
            },
            children![
                rule(0.0, Vec2::new(WIDTH, theme::LINE), theme::HAIRLINE),
                rule(50.0, Vec2::new(theme::LINE, theme::GAP_S), theme::MUTED),
                mark(Mark::Home, Icon::Home),
                mark(Mark::Ship, Icon::Send),
            ],
        )],
    )
}

/// What the compass reads.
#[derive(SystemParam)]
pub(super) struct Around<'w, 's> {
    /// The render anchor.
    anchor: Option<Res<'w, Anchor>>,
    /// Where the player is.
    location: Option<Res<'w, Location>>,
    /// The home base, once found.
    base: Option<Res<'w, Base>>,
    /// The ship.
    ship: Query<'w, 's, &'static Ship>,
    /// The game camera, which says where the player is and looks.
    camera: Query<'w, 's, &'static GlobalTransform, With<GameCamera>>,
}

impl Around<'_, '_> {
    /// Returns the render position of what `mark` points at, when it stands on the planet the
    /// player is on. The ship has no mark while the player sits in it.
    fn target(&self, mark: Mark) -> Option<Vec3> {
        let anchor = self.anchor.as_deref()?;
        let Frame::Surface(body) = anchor.frame else {
            return None;
        };
        let on_foot = self.location.as_deref()?.travel == Travel::OnFoot;
        let at = match mark {
            Mark::Home => {
                self.base
                    .as_deref()
                    .filter(|base| base.body == body)?
                    .centre
            }
            Mark::Ship => {
                let ship = self.ship.iter().next().filter(|_| on_foot)?;
                (ship.frame == anchor.frame).then_some(ship.pos)?
            }
        };
        Some(anchor.to_render(at))
    }

    /// Returns where `mark` sits on the bar and how far its target is, in metres, or nothing
    /// when it has no target on this planet or the target is right here.
    fn reading(&self, mark: Mark) -> Option<(f32, f32)> {
        let (anchor, camera) = (self.anchor.as_deref()?, self.camera.iter().next()?);
        let eye = camera.translation();
        let offset = self.target(mark)? - eye;
        let around = bearing(offset, camera.forward().as_vec3(), up(anchor, eye))?;
        (offset.length() > HERE).then(|| (slot(around), offset.length()))
    }
}

/// Slides every mark to where its target lies and writes its distance, and hides the bar
/// while no mark has a target.
pub(super) fn update(
    around: Around,
    mut marks: Query<(&Mark, &mut Node)>,
    mut texts: Query<(&Far, &mut Text)>,
    mut bars: Query<&mut Visibility, With<Bar>>,
) {
    let home = around.reading(Mark::Home).map(|(slot, _)| slot);
    let shown = match home.is_some() || around.reading(Mark::Ship).is_some() {
        true => Visibility::Inherited,
        false => Visibility::Hidden,
    };
    for mut bar in &mut bars {
        bar.set_if_neq(shown);
    }
    for (mark, mut node) in &mut marks {
        let reading = around.reading(*mark);
        let display = reading.map_or(Display::None, |_| Display::Flex);
        if node.display != display {
            node.display = display;
        }
        let Some((slot, far)) = reading else {
            continue;
        };
        let at = match mark {
            Mark::Home => slot * WIDTH,
            Mark::Ship => beside(slot, home),
        };
        node.left = px(at - ROOM * 0.5);
        for (_, mut text) in texts.iter_mut().filter(|(of, _)| of.0 == *mark) {
            text.set_if_neq(Text(distance(f64::from(far))));
        }
    }
}
