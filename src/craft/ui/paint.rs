//! Draws the open screen from the bench and the pack: 6x4 pack grid, categorized Crafting Workshop,
//! 3D previews, hotbar slots, and tool solver cards.

use bevy::prelude::*;

use super::bench::Bench;
use super::click::Press;
use super::here::Here;
use super::keys::{Bind, MAKE_BINDS, PACK_BINDS, moves};
use super::scene::{Ground, Scene, scene};
use super::view::{Holder, Tabbed, ToolsOnly, Words};
use super::{cards, catalog_panel, catalog_ui, grid, grid_detail, intent, slots, tiles};
use crate::craft::build::Place;
use crate::craft::build::catalog::Category;
use crate::craft::pack::{Burden, Pack};
use crate::ui::units::fixed;
use crate::ui::{Icon, Tone, action, hints, legend, meter, notice_row, text, theme};

/// Spawns action buttons cleanly inside a horizontal strip alongside an optional note.
fn actions(parent: &mut ChildSpawnerCommands, binds: &[Bind], shown: bool, note: &str) {
    for bind in binds.iter().filter(|_| shown) {
        let tone = match bind.press {
            Press::Make => Tone::Accent,
            _ => Tone::Plain,
        };
        parent.spawn((bind.press, action(bind.icon, bind.label, bind.code, tone)));
    }
    if !note.is_empty() {
        let words = text(note, theme::FONT_SMALL, theme::TEXT);
        parent.spawn(notice_row(Icon::Info, (words, TextLayout::no_wrap())));
    }
}

/// Spawns the carry-load summary bar under `parent`: label, meter and carried over capacity.
fn load_bar(parent: &mut ChildSpawnerCommands, burden: &Burden) {
    let share = if burden.capacity > f32::EPSILON {
        (burden.load / burden.capacity).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let color = if burden.load > burden.capacity {
        theme::WARN
    } else {
        theme::TEXT
    };
    let words = format!(
        "{} / {} kg",
        fixed(burden.load, 1),
        fixed(burden.capacity, 1)
    );
    parent.spawn((
        text("Load", theme::FONT_SMALL, theme::TEXT_2),
        TextLayout::no_wrap(),
    ));
    parent.spawn(meter(theme::span(3), share));
    parent.spawn((text(words, theme::FONT_SMALL, color), TextLayout::no_wrap()));
}

/// Spawns what `holder` shows of `shown` under `parent`.
fn fill(
    parent: &mut ChildSpawnerCommands,
    holder: Holder,
    (shown, pack, bench, place): (&Scene, &Pack, &Bench, &Place),
) {
    match holder {
        Holder::Tabs => tiles::tabs(parent, bench.pane),
        Holder::Load => load_bar(parent, &shown.burden),
        Holder::List => grid::pack_grid(parent, pack, bench, place),
        Holder::Detail => {
            grid_detail::inspect_slot(
                parent,
                pack,
                bench.selected_slot,
                place,
                shown.detail.as_ref(),
            );
        }
        Holder::PackDeeds => actions(parent, &PACK_BINDS, true, &bench.note),
        Holder::Categories => catalog_ui::categories(parent, bench.category),
        Holder::Intents => tiles::intents(parent, bench.intent),
        Holder::Cards => {
            if bench.category == Category::Tools {
                cards::cards(parent, &shown.tools, bench.intent);
            } else {
                parent
                    .spawn(Node {
                        width: percent(100),
                        flex_direction: FlexDirection::Row,
                        column_gap: px(theme::GUTTER),
                        align_items: AlignItems::FlexStart,
                        ..default()
                    })
                    .with_children(|row| {
                        catalog_ui::recipe_list(row, pack, bench);
                        catalog_panel::recipe_inspector(row, pack, bench);
                    });
            }
        }
        Holder::Deeds => {
            if bench.category == Category::Tools {
                actions(parent, &MAKE_BINDS, shown.tools.is_ok(), &bench.note);
            } else {
                catalog_panel::catalog_deeds(parent, pack, bench);
            }
        }
        Holder::Slots => slots::slots_from_pack(parent, pack, bench, place),
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
    let place = here.place();
    let ground = Ground {
        place,
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
        held.with_children(|parent| fill(parent, *holder, (&shown, &pack, &bench, &place)));
    }
}

/// Shows the active tab pane and hides tool-intent rows when browsing crafting categories.
pub(super) fn show_tab(
    bench: Res<Bench>,
    mut panes: Query<(&Tabbed, &mut Node)>,
    mut tools_only: Query<&mut Node, (With<ToolsOnly>, Without<Tabbed>)>,
) {
    for (Tabbed(pane), mut node) in &mut panes {
        let display = [Display::None, Display::Flex][usize::from(*pane == bench.pane)];
        if node.display != display {
            node.display = display;
        }
    }
    let tools_disp = [Display::None, Display::Flex][usize::from(bench.category == Category::Tools)];
    for mut node in &mut tools_only {
        if node.display != tools_disp {
            node.display = tools_disp;
        }
    }
}
