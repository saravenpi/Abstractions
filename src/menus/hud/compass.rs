//! The compass: a thin bar at the top of the screen with a mark for the home base and one for
//! the ship, each where it lies around the player, with its distance under it.
//!
//! A mark slides along the bar as the player turns and waits at the end of the bar while its
//! target is behind, so the bar always says which way to turn. The ship only shows on the
//! planet it stands on. Away from the home planet, in space or on another world, the home
//! mark points at the next place on the way home: the home planet, its star or its galaxy.

use bevy::ecs::system::SystemParam;
use bevy::math::DQuat;
use bevy::prelude::*;

use super::needle::{WIDTH, bearing, beside, slot};
use crate::computer::{distance, homeward};
use crate::home::Base;
use crate::render::GameCamera;
use crate::ship::Ship;
use crate::space::{Anchor, Clock, Frame, Location, Travel, up};
use crate::ui::{Icon, Tone, icon_sized, text, theme};
use crate::universe::Universe;
use crate::world::{Planet, spin};

/// Width in pixels of the room of one mark, which is centred on its place on the bar.
const ROOM: f32 = 96.0;

/// Height in pixels of the icon of a mark, small enough to leave the view clear.
const MARK: f32 = 14.0;

/// Distance in metres under which a target is here and its mark is hidden.
const HERE: f64 = 12.0;

/// What a mark of the compass points at.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Mark {
    /// The home base, or the way home away from the home planet.
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
            icon_sized(symbol, Tone::Plain, MARK),
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
            top: px(MARK + theme::GAP_XS),
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
                height: px(MARK * 2.0 + theme::GAP_M),
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
    /// The universe and its time, which say where home is from anywhere.
    sky: Option<(Res<'w, Universe>, Res<'w, Clock>)>,
    /// The planet the player is on, whose spin turns the sky.
    planet: Option<Res<'w, Planet>>,
    /// The ship.
    ship: Query<'w, 's, &'static Ship>,
    /// The game camera, which says where the player is and looks.
    camera: Query<'w, 's, &'static GlobalTransform, With<GameCamera>>,
}

impl Around<'_, '_> {
    /// Returns the way from the eye at `eye` to what `mark` points at, in render axes, and
    /// its distance in metres: the base or the ship on the planet the player is on, and the
    /// way home anywhere else. The ship has no mark while the player sits in it.
    fn target(&self, mark: Mark, eye: Vec3) -> Option<(Vec3, f64)> {
        let (anchor, location) = (self.anchor.as_deref()?, self.location.as_deref()?);
        let ground = match anchor.frame {
            Frame::Surface(body) => Some(body),
            _ => None,
        };
        let at = match mark {
            Mark::Home => {
                let base = self.base.as_deref();
                let Some(base) = base.filter(|base| Some(base.body) == ground) else {
                    return self.far(eye);
                };
                base.centre
            }
            Mark::Ship => {
                let on_foot = location.travel == Travel::OnFoot && ground.is_some();
                let ship = self.ship.iter().next().filter(|_| on_foot)?;
                (ship.frame == anchor.frame).then_some(ship.pos)?
            }
        };
        let offset = anchor.to_render(at) - eye;
        Some((offset, f64::from(offset.length())))
    }

    /// Returns the way from the eye at `eye` to the next place on the way home, in render
    /// axes, and its distance in metres. On a planet the sky turns against its spin.
    fn far(&self, eye: Vec3) -> Option<(Vec3, f64)> {
        let (anchor, (universe, clock)) = (self.anchor.as_deref()?, self.sky.as_ref()?);
        let at = anchor.to_frame(eye);
        let (toward, gap) = homeward(universe, anchor.frame, at, clock.seconds)?;
        let planet = self.planet.as_deref();
        let day = planet.filter(|planet| Frame::Surface(planet.body) == anchor.frame);
        let turn = day.map_or(DQuat::IDENTITY, |planet| {
            spin(planet.day_seconds, clock.seconds)
        });
        Some(((anchor.rotation * turn.inverse() * toward).as_vec3(), gap))
    }

    /// Returns where `mark` sits on the bar and how far its target is, in metres, or nothing
    /// when it has no target or the target is right here. On a planet the bar turns around
    /// the way up; in space, around the top of the view.
    fn reading(&self, mark: Mark) -> Option<(f32, f64)> {
        let (anchor, camera) = (self.anchor.as_deref()?, self.camera.iter().next()?);
        let eye = camera.translation();
        let (offset, gap) = self.target(mark, eye)?;
        let above = match anchor.frame {
            Frame::Surface(_) => up(anchor, eye),
            _ => camera.up().as_vec3(),
        };
        let around = bearing(offset, camera.forward().as_vec3(), above)?;
        (gap > HERE).then(|| (slot(around), gap))
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
            text.set_if_neq(Text(distance(far)));
        }
    }
}
