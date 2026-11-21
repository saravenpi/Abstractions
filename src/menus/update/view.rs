//! The update menu: one screen whose page follows the updater.

use bevy::prelude::*;

use super::pages;
use super::parts::WIDTH;
use crate::menus::Menu;
use crate::ui::{column, footer, heading, hints, panel, root, theme};
use crate::update::{Outcome, Plan, Stage, Updates};

/// Key hints shown at the bottom of the screen.
const HINTS: [(&str, &str); 4] = [
    ("Move", "Up Down"),
    ("Choose", "Enter"),
    ("Scroll the notes", "PgUp PgDn"),
    ("Back", "Esc"),
];

/// The page the update menu shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum View {
    /// No newer release is known.
    Nothing,
    /// A newer release, with what the game can do about it.
    Offer,
    /// The download runs.
    Downloading,
    /// The new game is in place.
    Restart,
    /// The disk image with this file name is in Downloads.
    Image(String),
    /// The install failed, for this reason.
    Failed(String),
}

impl View {
    /// Returns the page for the state of the updater.
    pub fn of(updates: &Updates) -> Self {
        match (updates.stage(), updates.release()) {
            (Stage::Failed(reason), _) => View::Failed(reason.clone()),
            (_, None) => View::Nothing,
            (Stage::Idle, Some(_)) => View::Offer,
            (Stage::Downloading, Some(_)) => View::Downloading,
            (Stage::Ready(Outcome::Swapped), Some(_)) => View::Restart,
            (Stage::Ready(Outcome::Image(path)), Some(_)) => {
                let name = path.file_name().unwrap_or_default();
                View::Image(name.to_string_lossy().to_string())
            }
        }
    }
}

/// The part of the menu that is rebuilt when the page changes, with the page it shows and how
/// the release it shows can be installed.
#[derive(Component, Debug, Clone, PartialEq, Eq, Default)]
pub struct Body(Option<(View, Plan)>);

/// Spawns the update menu with an empty body, which [`refresh`] fills.
pub(super) fn spawn(mut commands: Commands) {
    commands.spawn((
        DespawnOnExit(Menu::Update),
        root(),
        children![
            (
                panel(),
                children![
                    heading("Update"),
                    (Body::default(), column(px(WIDTH), theme::GAP_M))
                ]
            ),
            footer(hints(&HINTS)),
        ],
    ));
}

/// Rebuilds the body when the page it should show changed. The main button of the new page
/// takes the focus.
pub(super) fn refresh(
    mut commands: Commands,
    updates: Res<Updates>,
    mut bodies: Query<(Entity, &mut Body)>,
) {
    let wanted = Some((View::of(&updates), updates.plan()));
    for (entity, mut body) in &mut bodies {
        if body.0 == wanted {
            continue;
        }
        body.0 = wanted.clone();
        let Some((view, _)) = &wanted else {
            continue;
        };
        commands
            .entity(entity)
            .despawn_related::<Children>()
            .with_children(|page| pages::fill(page, view, &updates));
    }
}
