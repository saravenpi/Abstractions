//! Markers over the world: the active pin, the course, home, the ship on foot, friends and the
//! pins within reach, drawn where they really are. In view, the icon sits at the projected
//! place of the target with its distance under it. Out of view, it slides to the border of the
//! screen with an arrow toward it, at the height of the real position.
//!
//! One source of truth: the shared [`Markers`] list the compass and the minimap read. A pool of
//! [`MAX`] nodes is spawned once and refilled every frame without allocating. Nothing shows in
//! the map, in menus or when the setting is off.

mod place;
#[cfg(test)]
mod tests;

pub use place::{Placed, choose, dimmed, place};

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::compass::{Around, FAVOURITE_REACH, icon};
use super::context::{Element, Where, marker_shown};
use super::fade::gate;
use crate::computer::distance;
use crate::menus::{Marker, MarkerKind, Markers};
use crate::settings::{Persistent, Settings};
use crate::ui::{ICON_FONT, text, theme};
use place::MAX;
use std::fmt::Write;

/// Metres under which a marker is hidden: it is right here.
const HERE: f64 = 5.0;

/// Pixels the safe rectangle must be crossed by, going back on screen, once a marker sits on the
/// border: it stops the icon from flickering between the two forms.
const HYSTERESIS: f32 = 8.0;

/// How fast an icon follows its target, per second.
const FOLLOW: f32 = 25.0;

/// Width of the room of one marker, centred on its icon.
const ROOM: f32 = 96.0;

/// Pixels from the centre of an icon to the centre of its arrow.
const ARROW_REACH: f32 = 22.0;

/// Marks the room of the marker of the pool at this slot.
#[derive(Component)]
struct Slot(usize);

/// Marks the icon of the marker of the pool at this slot.
#[derive(Component)]
struct Glyph(usize);

/// Marks the arrow of the marker of the pool at this slot.
#[derive(Component)]
struct Arrow(usize);

/// Marks the distance of the marker of the pool at this slot.
#[derive(Component)]
struct Label(usize);

/// One marker chosen to show this frame.
#[derive(Debug, Clone, Copy)]
struct Pick {
    /// Its index in [`Markers`].
    index: usize,
    /// Its direction in the camera frame.
    dir: Vec3,
    /// Its distance in metres.
    gap: f64,
}

/// What a slot drew last frame.
#[derive(Debug, Default, Clone, Copy)]
struct Memory {
    /// The marker it showed: its kind and index.
    who: Option<(MarkerKind, usize)>,
    /// Where its icon was, in pixels.
    pos: Vec2,
    /// Whether it sat on the border.
    edge: bool,
}

/// One slot of the pool: a room with an arrow, an icon and a distance, hidden.
fn slot(at: usize) -> impl Bundle {
    let shadow = TextShadow::default();
    (
        Slot(at),
        Node {
            display: Display::None,
            position_type: PositionType::Absolute,
            width: px(ROOM),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            ..default()
        },
        Pickable::IGNORE,
        children![
            (
                Glyph(at),
                Text::default(),
                TextFont::from_font_size(theme::ICON).with_font(ICON_FONT),
                TextLayout::no_wrap(),
                TextColor(Color::WHITE),
                shadow,
            ),
            (
                Arrow(at),
                Text::new(crate::ui::Icon::Arrow.glyph()),
                TextFont::from_font_size(theme::ICON).with_font(ICON_FONT),
                TextLayout::no_wrap(),
                TextColor(Color::WHITE),
                shadow,
                Node {
                    display: Display::None,
                    position_type: PositionType::Absolute,
                    left: px((ROOM - theme::ICON) * 0.5),
                    top: px(0),
                    ..default()
                },
                UiTransform::default(),
            ),
            (
                Label(at),
                text("", theme::FONT_SMALL, theme::TEXT),
                TextLayout::no_wrap(),
                shadow,
            ),
        ],
    )
}

/// The layer of markers over the world, full screen, with its pool of hidden slots.
pub(super) fn layer() -> impl Bundle {
    (
        gate(Element::Markers),
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            ..default()
        },
        Pickable::IGNORE,
        Children::spawn(SpawnIter((0..MAX).map(slot))),
    )
}

/// The adapter between the markers and the pin system: whether `marker` is the active pin,
/// which shows at any distance. Today it is the player's own waypoint. When `Places` lands,
/// replace the body by a comparison with `Places::active()`.
fn is_active(marker: &Marker) -> bool {
    marker.kind == MarkerKind::Waypoint && marker.owner == 0
}

/// Returns whether `marker` at `gap` metres is worth drawing: the course and the active pin at
/// any distance, other pins within reach, the rest unless right here.
fn worth(marker: &Marker, gap: f64) -> bool {
    let pin = matches!(marker.kind, MarkerKind::Waypoint | MarkerKind::Favourite);
    gap > HERE && (!pin || is_active(marker) || gap <= FAVOURITE_REACH)
}

/// Returns the colour a marker is drawn in: the accent for the course, the pin colour for pins,
/// its own for friends, white for home and the ship.
fn tint(marker: &Marker) -> Color {
    match marker.kind {
        MarkerKind::Course => theme::ACCENT,
        MarkerKind::Waypoint | MarkerKind::Favourite => theme::PIN,
        MarkerKind::Friend => marker.color,
        _ => theme::TEXT,
    }
}

/// The nodes of the pool.
type Nodes<'w, 's> = (
    Query<'w, 's, (&'static Slot, &'static mut Node), (Without<Arrow>, Without<Glyph>)>,
    Query<
        'w,
        's,
        (&'static Glyph, &'static mut Text, &'static mut TextColor),
        (Without<Arrow>, Without<Label>),
    >,
    Query<
        'w,
        's,
        (
            &'static Arrow,
            &'static mut Node,
            &'static mut UiTransform,
            &'static mut TextColor,
        ),
        (Without<Slot>, Without<Glyph>, Without<Label>),
    >,
    Query<
        'w,
        's,
        (&'static Label, &'static mut Text, &'static mut TextColor),
        (Without<Glyph>, Without<Arrow>),
    >,
);

/// Chooses the markers to show, places each and writes the pool.
pub(super) fn update(
    (around, now): (Around, Res<Where>),
    (markers, settings, scale, time): (
        Res<Markers>,
        Res<Persistent<Settings>>,
        Option<Res<UiScale>>,
        Res<Time>,
    ),
    window: Single<&Window, With<PrimaryWindow>>,
    (mut picks, mut items, mut order): (Local<Vec<Pick>>, Local<Vec<(bool, f64)>>, Local<Vec<usize>>),
    (mut memory, mut words): (Local<[Memory; MAX]>, Local<String>),
    (mut slots, mut glyphs, mut arrows, mut labels): Nodes,
) {
    picks.clear();
    items.clear();
    for (index, marker) in markers.0.iter().enumerate() {
        if !marker_shown(now.0, marker.kind) || marker.kind == MarkerKind::Herd {
            continue;
        }
        let Some((dir, gap)) = around.seen(marker) else {
            continue;
        };
        if worth(marker, gap) {
            picks.push(Pick { index, dir, gap });
            let always = marker.kind == MarkerKind::Course || is_active(marker);
            items.push((always, gap));
        }
    }
    choose(&items, &mut order);
    let screen = Vec2::new(window.width(), window.height()) / scale.map_or(1.0, |scale| scale.0).max(0.1);
    let fov = (settings.graphics.fov as f32).to_radians();
    let follow = 1.0 - (-FOLLOW * time.delta_secs()).exp();
    let inset = theme::EDGE + theme::ICON * 0.5;
    for at in 0..MAX {
        let pick = order.get(at).map(|chosen| picks[*chosen]);
        let marker = pick.map(|pick| &markers.0[pick.index]);
        let placed = pick.map(|pick| {
            let more = if memory[at].edge { HYSTERESIS } else { 0.0 };
            place(pick.dir, fov, screen, inset + more)
        });
        if let (Some(pick), Some(placed), Some(marker)) = (pick, placed, marker) {
            let who = Some((marker.kind, pick.index));
            let moved = if memory[at].who == who {
                memory[at].pos.lerp(placed.pos, follow)
            } else {
                placed.pos
            };
            memory[at] = Memory {
                who,
                pos: moved,
                edge: placed.arrow.is_some(),
            };
        } else {
            memory[at].who = None;
        }
        for (Slot(slot), mut node) in &mut slots {
            if *slot != at {
                continue;
            }
            let display = [Display::None, Display::Flex][usize::from(pick.is_some())];
            let (left, top) = (
                px(memory[at].pos.x - ROOM * 0.5),
                px(memory[at].pos.y - theme::ICON * 0.5),
            );
            if (node.display, node.left, node.top) != (display, left, top) {
                (node.display, node.left, node.top) = (display, left, top);
            }
        }
        let (Some(pick), Some(placed), Some(marker)) = (pick, placed, marker) else {
            continue;
        };
        let dim = match placed.arrow {
            None => dimmed((-pick.dir.z).clamp(-1.0, 1.0).acos()),
            Some(_) => 1.0,
        };
        let color = tint(marker).with_alpha(dim);
        let symbol = icon(marker.kind);
        for (Glyph(slot), mut text, mut paint) in &mut glyphs {
            if *slot == at {
                if !text.0.starts_with(symbol.codepoint()) {
                    text.0 = symbol.glyph();
                }
                paint.set_if_neq(TextColor(color));
            }
        }
        for (Arrow(slot), mut node, mut turn, mut paint) in &mut arrows {
            if *slot != at {
                continue;
            }
            let display = [Display::None, Display::Flex][usize::from(placed.arrow.is_some())];
            if node.display != display {
                node.display = display;
            }
            if let Some(angle) = placed.arrow {
                let (x, y) = (angle.sin() * ARROW_REACH, -angle.cos() * ARROW_REACH);
                let wanted = UiTransform {
                    translation: Val2::px(x, y),
                    rotation: Rot2::radians(angle),
                    ..UiTransform::IDENTITY
                };
                turn.set_if_neq(wanted);
            }
            paint.set_if_neq(TextColor(color));
        }
        words.clear();
        let _ = write!(words, "{}", distance(pick.gap));
        for (Label(slot), mut text, mut paint) in &mut labels {
            if *slot == at {
                if text.0 != *words {
                    text.0.clone_from(&words);
                }
                paint.set_if_neq(TextColor(theme::TEXT.with_alpha(dim)));
            }
        }
    }
}

/// Run condition: the player turned markers over the world on.
fn enabled(settings: Res<Persistent<Settings>>, now: Res<Where>) -> bool {
    settings.gameplay.world_markers && now.has(Element::Markers)
}

/// Registers the markers over the world, drawn after the markers are filled and the compass
/// read them.
pub(super) fn plugin(app: &mut App) {
    use super::MarkerSet;
    use crate::screens::{Screen, in_control};
    app.add_systems(
        Update,
        update
            .after(MarkerSet::Fill)
            .run_if(in_control)
            .run_if(enabled)
            .run_if(in_state(Screen::Gameplay)),
    );
}
