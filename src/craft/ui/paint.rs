//! Draws the open screen from the bench and the pack: the load and the line on the intent, then
//! every holder again, from the one [`Scene`] of this frame.

use bevy::prelude::*;

use super::bench::{Bench, Pane};
use super::click::Press;
use super::here::Here;
use super::keys::{Bind, MAKE_BINDS, PACK_BINDS, moves};
use super::scene::{Ground, Scene, scene};
use super::view::{Holder, Tabbed, Words};
use super::{cards, intent, rows, slots, tiles};
use crate::craft::pack::{Burden, Pack};
use crate::ui::{Icon, Tone, action, hints, legend, notice_row, text, theme};

/// Spawns the buttons of `binds` under `parent` when `shown`, the first in the accent when it
/// makes a tool, then `note` when there is one.
fn actions(parent: &mut ChildSpawnerCommands, binds: &[Bind], shown: bool, note: &str) {
    for bind in binds.iter().filter(|_| shown) {
        let tone = match bind.press {
            Press::Make => Tone::Accent,
            _ => Tone::Plain,
        };
        parent.spawn((bind.press, action(bind.icon, bind.label, bind.key, tone)));
    }
    if !note.is_empty() {
        let words = text(note, theme::FONT_SMALL, theme::TEXT);
        parent.spawn(notice_row(Icon::Info, (words, TextLayout::no_wrap())));
    }
}

/// Spawns what `holder` shows of `shown` under `parent`.
fn fill(parent: &mut ChildSpawnerCommands, holder: Holder, shown: &Scene, bench: &Bench) {
    match holder {
        Holder::Tabs => tiles::tabs(parent, bench.pane),
        Holder::List => rows::list(parent, &shown.rows, bench.pane == Pane::Pack),
        Holder::PackDeeds => actions(parent, &PACK_BINDS, !shown.rows.is_empty(), ""),
        Holder::Intents => tiles::intents(parent, bench.intent),
        Holder::Cards => cards::cards(parent, &shown.tools, bench.intent),
        Holder::Deeds => actions(parent, &MAKE_BINDS, shown.tools.is_ok(), &bench.note),
        Holder::Slots => slots::slots(parent, &shown.slots),
        Holder::Legend => {
            parent.spawn(legend(hints(moves(bench.pane))));
        }
    }
}

/// Redraws the screen when the bench or the pack changes.
pub(super) fn draw(
    (bench, pack, burden): (Res<Bench>, Res<Pack>, Res<Burden>),
    here: Here,
    holders: Query<(Entity, &Holder)>,
    mut words: Query<(&mut Text, &Words)>,
    mut commands: Commands,
) {
    let ground = Ground {
        place: here.place(),
        life: here.life(),
        burden: *burden,
    };
    let shown = scene(&bench, &pack, &ground);
    for (mut said, which) in &mut words {
        said.0 = match which {
            Words::Status => shown.status.clone(),
            Words::Explain => intent::line(bench.intent).to_string(),
        };
    }
    for (entity, holder) in &holders {
        let mut held = commands.entity(entity);
        held.despawn_children();
        held.with_children(|parent| fill(parent, *holder, &shown, &bench));
    }
}

/// Shows the pane of the open tab and hides the other.
pub(super) fn show_tab(bench: Res<Bench>, mut panes: Query<(&Tabbed, &mut Node)>) {
    for (Tabbed(pane), mut node) in &mut panes {
        let display = [Display::None, Display::Flex][usize::from(*pane == bench.pane)];
        if node.display != display {
            node.display = display;
        }
    }
}
