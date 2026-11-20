//! The action line of the course in the ship: the engage key and "Engage autopilot to" the
//! destination while the autopilot can take the course, or the first reason it cannot, or how
//! far along the route is. Hidden on foot and without a course.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::{Prompt, PromptAction, PromptKey};
use crate::computer::{Destination, RouteStatus, action};
use crate::menus::key_name;
use crate::settings::{Persistent, Settings};
use crate::space::{Location, Travel};
use crate::ui::{Tone, key_cap, label, theme};
use crate::universe::Universe;

/// Marks the action line of the course.
#[derive(Component)]
pub(in crate::menus::hud) struct CourseRow;

/// Marks the key of the action line.
#[derive(Component)]
pub(in crate::menus::hud) struct CourseKey;

/// Marks the words of the action line.
#[derive(Component)]
pub(in crate::menus::hud) struct CourseWords;

/// The hidden action line of the course.
pub(super) fn row() -> impl Bundle {
    (
        CourseRow,
        Node {
            display: Display::None,
            align_items: AlignItems::Center,
            column_gap: px(theme::GAP_S),
            padding: UiRect::axes(px(theme::GAP_S), px(theme::GAP_XS)),
            ..default()
        },
        BackgroundColor(theme::PANEL),
        children![
            (CourseKey, key_cap("")),
            (CourseWords, label("", theme::FONT_SMALL, Tone::Plain)),
        ],
    )
}

/// The texts of the action line, apart from those of the prompt.
type Words<'w, 's, M, N> = Query<
    'w,
    's,
    &'static mut Text,
    (
        With<M>,
        Without<N>,
        Without<PromptAction>,
        Without<PromptKey>,
    ),
>;

/// What the action line is written from and to.
#[derive(SystemParam)]
pub(in crate::menus::hud) struct Line<'w, 's> {
    location: Option<Res<'w, Location>>,
    course: (Res<'w, Destination>, Res<'w, RouteStatus>),
    world: (Option<Res<'w, Universe>>, Res<'w, Persistent<Settings>>),
    rows: Query<'w, 's, &'static mut Node, (With<CourseRow>, Without<Prompt>)>,
    fresh: Query<'w, 's, (), Added<CourseRow>>,
    words: Words<'w, 's, CourseWords, CourseKey>,
    keys: Words<'w, 's, CourseKey, CourseWords>,
}

impl Line<'_, '_> {
    /// Returns the key and the words of the line, or nothing while it is hidden.
    fn said(&self) -> Option<(String, String)> {
        let seated = self.location.as_deref()?.travel == Travel::InShip;
        let (destination, route) = (&self.course.0, *self.course.1);
        let target = destination.0.filter(|_| seated)?;
        let name = target.name(self.world.0.as_deref()?);
        let (engage, words) = action(&name, route);
        let key = self.world.1.controls.keys.engage;
        Some((if engage { key_name(key) } else { String::new() }, words))
    }

    /// Writes the line when the course, the route or the seat changed, or the HUD was just
    /// spawned, and hides it while there is nothing to say.
    pub(super) fn write(&mut self) {
        let (destination, route) = (&self.course.0, &self.course.1);
        let moved = self.location.as_ref().is_some_and(|at| at.is_changed());
        if !(destination.is_changed() || route.is_changed() || moved || !self.fresh.is_empty()) {
            return;
        }
        let said = self.said();
        for mut node in &mut self.rows {
            node.display = [Display::None, Display::Flex][usize::from(said.is_some())];
        }
        let Some((key, words)) = said else {
            return;
        };
        for mut text in &mut self.keys {
            text.set_if_neq(Text(key.clone()));
        }
        for mut text in &mut self.words {
            text.set_if_neq(Text(words.clone()));
        }
    }
}
