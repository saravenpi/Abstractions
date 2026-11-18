//! The pointer on the map: pointing at a mark, clicking it, dragging the chart and zooming it
//! with the wheel.

use bevy::input::mouse::{AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use bevy::ui_widgets::Activate;

use super::board::Board;
use super::chart::{Dot, chart, span};
use super::format::distance;
use super::labels::scale_bar;
use super::lens::{Lens, hit};
use super::map::{Field, SIZE, ScaleBar, ScaleText, Sheet};
use super::open::Computer;
use super::tab::Tab;
use super::view::Slot;
use crate::ui::theme;

/// Pixels around a mark within which the pointer is on it.
pub(super) const REACH: f32 = 10.0;

/// Pixels the pointer travels with the button held before a click becomes a drag.
pub(super) const SLOP: f32 = 3.0;

/// How much closer one notch of the wheel brings the map.
pub(super) const ZOOM_STEP: f32 = 1.25;

/// Seconds between two clicks on empty map that still count as a double click.
pub(super) const DOUBLE_CLICK: f32 = 0.4;

/// The key that shows the whole chart again.
pub(super) const RESET_KEY: KeyCode = KeyCode::Digit0;

/// How the map is looked at and what the pointer does on it.
#[derive(Resource, Debug, Clone, Default, PartialEq)]
pub(super) struct MapView {
    /// The zoom and the centre of the map.
    pub lens: Lens,
    /// The tab the lens belongs to. Another tab starts from the whole chart.
    pub tab: Tab,
    /// Every dot of the chart with its chart point, kept from the last time the table changed.
    pub dots: Vec<(Dot, Vec2)>,
    /// True while the pointer is over the map.
    pub over: bool,
    /// The pixel of the map the pointer is at.
    pub at: Vec2,
    /// The mark under the pointer.
    pub hover: Option<Dot>,
    /// Where the pointer was last seen with the left button held, since it went down on the map.
    pub held: Option<Vec2>,
    /// True once the held pointer moved far enough to drag the chart.
    pub dragged: bool,
    /// What the last click was on, and when in seconds.
    pub clicked: Option<(Option<Dot>, f32)>,
    /// The row the map last brought into view.
    pub cursor: usize,
    /// Side of the map on screen in pixels, or 0 before the window sized it.
    pub side: f32,
    /// Metres one chart unit stands for, or nothing on a chart that is not to scale.
    pub span: Option<f64>,
}

/// Follows the pointer: finds the mark under it when it moves, and gives the table cursor to
/// the row of that mark. A resting pointer leaves the cursor to the keyboard.
pub(super) fn aim(
    fields: Query<&RelativeCursorPosition, With<Field>>,
    mut board: ResMut<Board>,
    mut view: ResMut<MapView>,
) {
    let seen = fields.iter().next();
    let pointer = seen.and_then(|field| Some((field.cursor_over, field.normalized?)));
    let (over, at) = match pointer {
        Some((over, share)) => (over, (share + 0.5) * SIZE),
        None => (false, view.at),
    };
    let still = at == view.at && over == view.over;
    (view.over, view.at) = (over, at);
    if still || view.held.is_some() {
        return;
    }
    let hover = hit(&view.dots, view.lens, at, REACH).filter(|_| over);
    if hover == view.hover {
        return;
    }
    view.hover = hover;
    if let Some(Dot::Row(index)) = hover {
        let by = index as isize - board.cursor as isize;
        board.step(by);
    }
}

/// Applies one click at `now` seconds. A click on the mark of a row gives it the cursor, and
/// returns true when the row already had it from the click before: the row is then chosen.
/// Two quick clicks on empty map show the whole chart again.
pub(super) fn tap(view: &mut MapView, board: &mut Board, now: f32) -> bool {
    let last = view.clicked.replace((view.hover, now));
    let again = last.is_some_and(|(mark, _)| mark == view.hover);
    match view.hover {
        Some(Dot::Row(index)) if again && board.cursor == index => true,
        Some(Dot::Row(index)) => {
            board.step(index as isize - board.cursor as isize);
            false
        }
        None if again && last.is_some_and(|(_, then)| now - then < DOUBLE_CLICK) => {
            (view.lens, view.clicked) = (Lens::default(), None);
            false
        }
        _ => false,
    }
}

/// Drags the chart while the left button is held from the map, and on a release that did
/// not drag, clicks: the second click on a mark activates its row as a click on the row does.
pub(super) fn press(
    (buttons, time): (Res<ButtonInput<MouseButton>>, Res<Time>),
    slots: Query<(Entity, &Slot)>,
    mut board: ResMut<Board>,
    mut view: ResMut<MapView>,
    mut commands: Commands,
) {
    if buttons.just_pressed(MouseButton::Left) && view.over {
        (view.held, view.dragged) = (Some(view.at), false);
    }
    let Some(from) = view.held else {
        return;
    };
    if buttons.pressed(MouseButton::Left) {
        if view.dragged || from.distance(view.at) > SLOP {
            view.lens = view.lens.panned(view.at - from);
            (view.held, view.dragged) = (Some(view.at), true);
        }
        return;
    }
    view.held = None;
    if view.dragged || !tap(&mut view, &mut board, time.elapsed_secs()) {
        return;
    }
    let drawn = board.cursor - board.top;
    if let Some((entity, _)) = slots.iter().find(|(_, slot)| slot.0 == drawn) {
        commands.trigger(Activate { entity });
    }
}

/// Zooms around the pointer with the wheel while the pointer is over the map, and shows the
/// whole chart again on the reset key.
pub(super) fn zoom(
    wheel: Res<AccumulatedMouseScroll>,
    keys: Res<ButtonInput<KeyCode>>,
    mut view: ResMut<MapView>,
) {
    if keys.just_pressed(RESET_KEY) {
        view.lens = Lens::default();
    }
    let notches = match wheel.unit {
        MouseScrollUnit::Line => wheel.delta.y,
        MouseScrollUnit::Pixel => wheel.delta.y / theme::ROW_HEIGHT,
    };
    if view.over && notches != 0.0 {
        view.lens = view.lens.zoomed(view.at, ZOOM_STEP.powf(notches));
    }
}

/// Lays the chart out again when the table changed. Another tab starts from the whole chart,
/// and a cursor that moved to a row outside the map brings that row into view.
pub(super) fn plot(board: Res<Board>, mut view: ResMut<MapView>) {
    if !board.is_changed() {
        return;
    }
    view.dots = chart(board.tab, &board.lines, board.ship);
    view.span = span(board.tab, &board.lines, board.ship);
    if view.tab != board.tab {
        (view.tab, view.lens) = (board.tab, Lens::default());
        (view.hover, view.clicked) = (None, None);
    }
    if view.cursor == board.cursor {
        return;
    }
    view.cursor = board.cursor;
    let row = Dot::Row(board.cursor);
    if let Some((_, at)) = view.dots.iter().find(|(dot, _)| *dot == row) {
        view.lens = view.lens.revealing(*at);
    }
}

/// Moves and sizes the sheet the chart is drawn on, so the map shows what the lens looks at.
pub(super) fn slide(view: Res<MapView>, mut sheets: Query<&mut Node, With<Sheet>>) {
    let corner = view.lens.to_map(Vec2::new(-1.0, 1.0)) / SIZE * 100.0;
    let side = percent(view.lens.zoom * 100.0);
    for mut node in &mut sheets {
        let wanted = (percent(corner.x), percent(corner.y), side, side);
        if (node.left, node.top, node.width, node.height) != wanted {
            (node.left, node.top, node.width, node.height) = wanted;
        }
    }
}

/// What the scale of the map says on a chart whose distances are not drawn true.
pub(super) const NOT_TO_SCALE: &str = "orbits not to scale";

/// Writes the scale of the map when the zoom, the size of the map or the chart changed: a
/// round distance and a bar of that length.
pub(super) fn rule(
    view: Res<MapView>,
    fresh: Query<(), Added<ScaleText>>,
    mut bars: Query<&mut Node, With<ScaleBar>>,
    mut texts: Query<&mut Text, With<ScaleText>>,
    mut last: Local<(f32, Option<f64>)>,
) {
    let now = (view.lens.zoom * view.side, view.span);
    if *last == now && fresh.is_empty() {
        return;
    }
    *last = now;
    let true_scale = view.span.filter(|_| now.0 > 0.0);
    let bar = true_scale.map(|span| scale_bar(span * 2.0 / f64::from(now.0)));
    let (width, words) = match bar {
        Some((pixels, metres)) => (pixels, distance(metres)),
        None => (0.0, NOT_TO_SCALE.to_string()),
    };
    for mut node in &mut bars {
        node.width = px(width);
    }
    for mut text in &mut texts {
        text.set_if_neq(Text(words.clone()));
    }
}

/// Registers the lens and the systems of the pointer. An opened screen shows the whole chart.
pub(super) fn plugin(app: &mut App) {
    let whole = |mut view: ResMut<MapView>| {
        *view = MapView {
            side: view.side,
            ..default()
        }
    };
    app.init_resource::<MapView>()
        .add_systems(OnEnter(Computer::Open), whole)
        .add_systems(
            Update,
            (aim, press, zoom)
                .chain()
                .in_set(super::Input)
                .after(super::keys::follow_focus)
                .before(super::keys::navigate)
                .run_if(super::open::ready),
        );
}
