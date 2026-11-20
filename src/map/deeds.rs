//! The buttons on the card of the selected place, under its name: set the course there,
//! engage the autopilot, clear the course, focus, set a waypoint or save it as a favourite,
//! as the menu of a right click on the place offers them. Under the buttons, the reason the
//! drive cannot engage, while the place is the course.

use bevy::prelude::*;
use bevy::ui_widgets::Activate;

use super::menu::{Choice, Hands, Under, choices, words};
use crate::computer::{Act, Board, Destination, RouteStatus, Target};
use crate::ui::{Icon, Tone, column, entry, frame, text, theme};

/// What a button under the card does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Deed {
    /// A choice of the menu of a right click on the place.
    Choice(Choice),
    /// Hand the course to the autopilot.
    Engage,
    /// Forget the course.
    Clear,
}

/// Marks a button that runs a choice of the menu on a place.
#[derive(Component, Debug, Clone, Copy)]
pub struct DeedChoice(pub Choice, pub Target);

/// Marks the row the buttons are in.
#[derive(Component)]
pub struct Deeds;

/// Marks the line that says why the drive cannot engage.
#[derive(Component)]
pub struct Reason;

/// Returns the buttons of the card of `target` with the course `course` in the state
/// `status`, in the order of the menu: setting the course while the place is not the course,
/// and while it is, engaging the autopilot when the route is idle and clearing the course.
pub fn offer(target: Target, course: Option<Target>, status: RouteStatus) -> Vec<Deed> {
    let ours = course == Some(target);
    let engage = (status == RouteStatus::Idle).then_some(Deed::Engage);
    let course_deeds: Vec<Deed> = match ours {
        true => engage.into_iter().chain([Deed::Clear]).collect(),
        false => vec![Deed::Choice(Choice::Course)],
    };
    let each = |choice: &Choice| match choice {
        Choice::Course => course_deeds.clone(),
        other => vec![Deed::Choice(*other)],
    };
    choices(&Under::Place(target))
        .iter()
        .flat_map(each)
        .collect()
}

/// Returns why the drive cannot engage on the course `course` in the state `status`, in plain
/// words, when the place `target` is that course, and nothing otherwise.
pub fn reason(target: Target, course: Option<Target>, status: RouteStatus) -> String {
    match status {
        RouteStatus::Blocked(block) if course == Some(target) => block.words(),
        _ => String::new(),
    }
}

/// Returns the icon and the words of the button `deed` on `target`: the words of the menu,
/// cut to one or two words so the buttons of the card fit two rows.
pub fn named(deed: Deed, target: Target) -> (Icon, &'static str) {
    match deed {
        Deed::Choice(Choice::Favourite) => (Icon::Star, "Save place"),
        Deed::Choice(choice) => words(choice, &Under::Place(target)),
        Deed::Engage => (Icon::Play, "Engage"),
        Deed::Clear => (Icon::Close, "Clear"),
    }
}

/// The buttons under the card, empty until [`write`] fills them, over the line that says why
/// the drive cannot engage.
pub fn deeds() -> impl Bundle {
    let row = Node {
        width: percent(100),
        flex_wrap: FlexWrap::Wrap,
        column_gap: px(theme::GAP_XS),
        row_gap: px(theme::GAP_XS),
        ..default()
    };
    (
        column(percent(100), theme::GAP_XS),
        children![
            (Deeds, row),
            (Reason, text("", theme::FONT_SMALL, theme::TEXT))
        ],
    )
}

/// Spawns the button `deed` on `target` under `row`. Engaging and clearing run the acts of the
/// computer, as their keys do; the others run the choice of the menu.
fn spawn(row: &mut ChildSpawnerCommands, deed: Deed, target: Target) {
    let (icon, said) = named(deed, target);
    let node = Node {
        align_items: AlignItems::Center,
        padding: UiRect::horizontal(px(theme::GAP_XS)),
        ..default()
    };
    let shown = entry(icon, said, theme::FONT_SMALL, Tone::Plain);
    let mut button = row.spawn((frame(), node, children![shown]));
    match deed {
        Deed::Choice(choice) => button.insert(DeedChoice(choice, target)),
        Deed::Engage => button.insert(Act::Engage),
        Deed::Clear => button.insert(Act::Clear),
    };
}

/// What the buttons are worked out from.
type Course<'w> = (Res<'w, Board>, Res<'w, Destination>, Res<'w, RouteStatus>);

/// What the buttons showed last: their row, the place and the buttons.
type Shown = Option<(Entity, Option<Target>, Vec<Deed>)>;

/// Writes the buttons of the row under the cursor and the reason the drive cannot engage, when
/// the table, the course or the route changed, or the screen was just opened.
pub(super) fn write(
    (board, destination, route): Course,
    (rows, fresh): (Query<Entity, With<Deeds>>, Query<(), Added<Deeds>>),
    mut reasons: Query<&mut Text, With<Reason>>,
    mut commands: Commands,
    mut last: Local<Shown>,
) {
    let changed = board.is_changed() || destination.is_changed() || route.is_changed();
    let Some(row) = rows.iter().next().filter(|_| changed || !fresh.is_empty()) else {
        return;
    };
    let target = board.lines.get(board.cursor).map(|line| line.target);
    let said = target.map_or(String::new(), |at| reason(at, destination.0, *route));
    for mut text in &mut reasons {
        text.set_if_neq(Text(said.clone()));
    }
    let deeds = target.map_or(Vec::new(), |at| offer(at, destination.0, *route));
    let now = Some((row, target, deeds));
    if *last == now {
        return;
    }
    commands.entity(row).despawn_children();
    if let Some((_, Some(at), deeds)) = &now {
        commands.entity(row).with_children(|row| {
            deeds.iter().for_each(|deed| spawn(row, *deed, *at));
        });
    }
    *last = now;
}

/// Runs the choice of the menu a button under the card stands for, on its place.
pub(super) fn on_press(event: On<Activate>, buttons: Query<&DeedChoice>, mut hands: Hands) {
    if let Ok(DeedChoice(choice, target)) = buttons.get(event.entity) {
        hands.run(*choice, Under::Place(*target));
    }
}
