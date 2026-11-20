//! Writes the open screen from the bench and the pack: the tab names, the count line, the drawn
//! rows of the list, the card and the key hints.

use bevy::prelude::*;

use super::bench::{Bench, Tab};
use super::card::{Shown, make_card, pack_card};
use super::glyph::{silhouette, wear_bar};
use super::here::Here;
use super::layout::{BAR, NAME, SIGNATURE};
use super::lines::{Line, arrange, lines};
use super::offer::offer;
use super::sheet::sheet;
use super::view::{Facts, Hints, MAKE_HINTS, PACK_HINTS, Rows, TabName, Tally};
use crate::craft::matter::Treatment;
use crate::craft::pack::{Burden, Pack};
use crate::craft::wield::Held;
use crate::fauna::guide::card::Card;
use crate::fauna::guide::show::{clip, rows};
use crate::ui::{Selected, Tone, frame, label, text, theme};

/// Returns the rows of the pack as the bench filters and orders them.
pub fn arranged(bench: &Bench, pack: &Pack, here: &Here) -> Vec<Line> {
    arrange(
        lines(pack, &here.place(), here.life()),
        &bench.filter,
        bench.sort,
    )
}

/// Returns one row of the list for `line`, marked when `chosen`.
fn row(line: &Line, chosen: bool) -> impl Bundle {
    let node = Node {
        width: percent(100),
        height: px(crate::fauna::guide::layout::ROW),
        flex_shrink: 0.0,
        align_items: AlignItems::Center,
        column_gap: px(theme::GAP_S),
        padding: UiRect::horizontal(px(theme::GAP_S)),
        ..default()
    };
    let grow = Node {
        flex_grow: 1.0,
        ..default()
    };
    let small = |words: String| label(words, theme::FONT_SMALL, Tone::Muted);
    let name = label(clip(&line.name, NAME), theme::FONT_BODY, Tone::Plain);
    let sign = small(clip(&line.signature, SIGNATURE));
    let mass = small(format!("{:.2} kg", line.mass));
    (
        frame(),
        Selected(chosen),
        node,
        children![
            silhouette(line.shape),
            (name, grow),
            sign,
            mass,
            wear_bar(BAR, line.wear.unwrap_or(0.0))
        ],
    )
}

/// The texts of the header and the footer.
type Texts<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Text,
        &'static mut TextColor,
        Option<&'static TabName>,
        Has<Hints>,
    ),
    Or<(With<TabName>, With<Tally>, With<Hints>)>,
>;

/// Writes the tab names, the count line and the hints.
fn heads(bench: &Bench, count: (usize, usize), burden: &Burden, texts: &mut Texts) {
    let tally = format!(
        "{} of {} entries{} - {:.1} of {:.1} kg{}",
        count.0,
        count.1,
        bench.sort.label(),
        burden.load,
        burden.capacity,
        match (bench.typing, bench.filter.is_empty()) {
            (true, _) => format!(" - search: {}_", bench.filter),
            (false, false) => format!(" - \"{}\"", bench.filter),
            (false, true) => String::new(),
        }
    );
    for (mut words, mut color, tab, hints) in texts.iter_mut() {
        match (tab, hints) {
            (Some(tab), _) => {
                let on = tab.0 == bench.tab;
                color.set_if_neq(TextColor([theme::MUTED, theme::TEXT][usize::from(on)]));
            }
            (None, true) => words.0 = [PACK_HINTS, MAKE_HINTS][bench.tab as usize].to_string(),
            (None, false) => words.0 = tally.clone(),
        }
    }
}

/// Returns the card of the open tab.
fn card_of(bench: &Bench, shown: &Shown, line: Option<&Line>) -> Card {
    match bench.tab {
        Tab::Pack => pack_card(line, shown),
        Tab::Make => {
            let made = offer(bench, shown.stock, &shown.place);
            make_card(bench, made.as_ref().map_err(|why| *why), shown)
        }
    }
}

/// Spawns `card` under `column`: its title, its subtitle and its rows.
fn card_into(commands: &mut Commands, column: Entity, card: &Card, most: usize) {
    commands.entity(column).despawn_children();
    commands.entity(column).with_children(|parent| {
        parent.spawn((
            text(
                clip(&card.title, most * 7 / 9),
                theme::FONT_BODY,
                theme::TEXT,
            ),
            TextLayout::no_wrap(),
        ));
        parent.spawn((
            text(clip(&card.subtitle, most), theme::FONT_SMALL, theme::MUTED),
            TextLayout::no_wrap(),
        ));
        rows(parent, card, most);
    });
}

/// Redraws the screen when the bench or the pack changes.
pub(super) fn draw(
    (bench, pack, burden, held): (Res<Bench>, Res<Pack>, Res<Burden>, Res<Held>),
    here: Here,
    mut texts: Texts,
    columns: (Query<Entity, With<Rows>>, Query<Entity, With<Facts>>),
    mut commands: Commands,
) {
    let all = arranged(&bench, &pack, &here);
    heads(
        &bench,
        (all.len(), pack.items.len() + pack.stacks.len()),
        &burden,
        &mut texts,
    );
    let place = here.place();
    let treatments = here.planet().map_or_else(Vec::new, |p| {
        Treatment::ALL
            .into_iter()
            .filter(|t| t.possible(&p.record))
            .collect()
    });
    let stock = pack.stock();
    let shown = Shown {
        pack: &pack,
        stock: &stock,
        place,
        burden: *burden,
        held: held.0.map(|h| sheet(&h.build, h.intent, &place)),
        treatments,
    };
    let card = card_of(&bench, &shown, all.get(bench.window.cursor));
    for column in &columns.1 {
        card_into(&mut commands, column, &card, bench.chars);
    }
    for column in &columns.0 {
        rows_into(&mut commands, column, &bench, &all);
    }
}

/// Spawns the drawn rows of `all` under `column`.
fn rows_into(commands: &mut Commands, column: Entity, bench: &Bench, all: &[Line]) {
    commands.entity(column).despawn_children();
    let drawn = (0..bench.rows).filter_map(|slot| bench.window.index(all.len(), slot));
    let built: Vec<_> = drawn
        .map(|i| row(&all[i], i == bench.window.cursor))
        .collect();
    commands.entity(column).with_children(|parent| {
        for line in built {
            parent.spawn(line);
        }
    });
}
