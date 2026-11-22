//! What the open screen shows, as plain data: the rows of the pack with the ones lit, the tool
//! cards of the workshop or what is missing, the six slots of the hotbar and the load. The paint
//! system draws it; tests read it without building the screen.

use super::bench::{Bench, Pane};
use super::detail::{Detail, detail};
use super::lines::{Line, What, lines};
use super::offer::{KINDS, offer, parts, title};
use super::reason::{Reason, missing, reason};
use super::sheet::{Gauge, gauges, sheet};
use crate::craft::build::{Look, Place, Proposal, Stock};
use crate::craft::pack::{Burden, Pack, SLOTS, cost};
use crate::life::Life;
use crate::ui::units::fixed;

/// The look the picture of a tool is drawn from.
pub type Shape = Option<Look>;

/// One row of the pack as the screen shows it.
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    /// The entry.
    pub line: Line,
    /// Whether it is lit: the cursor of the pack, or a part of the focused tool card.
    pub lit: bool,
    /// The kilograms the focused tool card takes from it, in the workshop pane.
    pub uses: Option<f32>,
}

/// One tool card of the workshop.
#[derive(Debug, Clone, PartialEq)]
pub struct Tool {
    /// Which of the three it is: lightest, hardest hitting, most durable.
    pub kind: &'static str,
    /// Its name.
    pub name: String,
    /// The one reason that matters.
    pub reason: Reason,
    /// What it takes from the pack, in words.
    pub parts: String,
    /// Its four bars.
    pub gauges: [Gauge; 4],
    /// Its silhouette.
    pub shape: Shape,
    /// Whether it is the focused card.
    pub focused: bool,
}

/// One slot of the hotbar as the screen shows it.
#[derive(Debug, Clone, PartialEq)]
pub struct Slot {
    /// The word for its tool, such as "spear", or nothing for an empty slot.
    pub name: Option<String>,
    /// The silhouette of its tool.
    pub shape: Shape,
    /// How worn its tool is, 0 new to 1 broken.
    pub worn: f32,
    /// Whether it is the held slot.
    pub held: bool,
}

/// Where the screen is open: the world a tool is used on, the life that names tissues and what
/// the pack weighs here.
pub struct Ground<'a> {
    /// The world.
    pub place: Place,
    /// The life underfoot and the terrain seed of its planet.
    pub life: Option<(&'a Life, u64)>,
    /// What the pack weighs here.
    pub burden: Burden,
}

/// Everything the screen shows.
#[derive(Debug, Clone, PartialEq)]
pub struct Scene {
    /// The rows of the pack, grouped by kind.
    pub rows: Vec<Row>,
    /// The focused pack row's details, when the pack is non-empty and the cursor is in bounds.
    pub detail: Option<Detail>,
    /// The three tool cards, or what the pack lacks for the intent.
    pub tools: Result<Vec<Tool>, String>,
    /// The six slots of the hotbar.
    pub slots: Vec<Slot>,
    /// What the pack weighs on this world.
    pub burden: Burden,
    /// The load in words.
    pub status: String,
}

/// Returns the load in words: the mass carried against what the frame carries here, and the pace.
pub fn status(burden: &Burden) -> String {
    format!(
        "{} of {} kg carried, pace {} %",
        fixed(burden.load, 1),
        fixed(burden.capacity, 1),
        fixed(burden.pace * 100.0, 0)
    )
}

/// Returns the look of the tool `proposal` makes from `stock`: the same look the tool has once
/// made, in the pack and on the hotbar.
pub fn look(proposal: &Proposal, stock: &[Stock]) -> Look {
    let stuff = |at: usize| {
        stock
            .get(usize::from(proposal.design.of[at]))
            .map(|s| &s.stuff)
    };
    let intent = proposal.design.intent;
    Look::of(&proposal.build, intent, stuff(1), stuff(0))
}

/// Returns the card of `proposal`, the one at `at`, focused or not.
fn tool(proposal: &Proposal, at: usize, focused: bool, stock: &[Stock], place: &Place) -> Tool {
    let build = &proposal.build;
    Tool {
        kind: KINDS[at],
        name: title(proposal, stock),
        reason: reason(proposal, at, place),
        parts: parts(proposal, stock),
        gauges: gauges(
            &sheet(build, proposal.design.intent, place),
            proposal.design.intent,
        ),
        shape: Some(look(proposal, stock)),
        focused,
    }
}

/// The cards a bench offers, or why it offers none, and what the focused card takes from the pack.
type Offer = (Result<Vec<Tool>, String>, Vec<(usize, f32)>);

/// Returns the three cards for the bench and what the focused one takes from `stock`, or what the
/// pack lacks.
fn tools(bench: &Bench, stock: &[Stock], place: &Place) -> Offer {
    let Ok(offered) = offer(bench, stock, place) else {
        return (Err(missing(bench.intent, stock)), Vec::new());
    };
    let pick = bench.pick.min(2);
    let mut cards: Vec<Tool> = (0..3)
        .map(|at| tool(&offered.three[at], at, at == pick, stock, place))
        .collect();
    let used = match &offered.chosen {
        Some(chosen) => {
            cards[pick] = tool(chosen, pick, true, stock, place);
            cost(chosen, stock)
        }
        None => {
            cards[pick].reason = Reason {
                bad: true,
                words: "Not enough of the swapped material".to_string(),
            };
            Vec::new()
        }
    };
    (Ok(cards), used)
}

/// Returns the rows of `all`, lit by the cursor in the pack pane, and in the workshop pane by the
/// parts the focused card takes, `used`, with the kilograms it takes.
fn rows(bench: &Bench, all: Vec<Line>, used: &[(usize, f32)]) -> Vec<Row> {
    let taken = |line: &Line| match line.what {
        What::Stack(at) => used.iter().find(|(i, _)| *i == at).map(|(_, kg)| *kg),
        What::Item(_) => None,
    };
    all.into_iter()
        .enumerate()
        .map(|(at, line)| {
            let (lit, uses) = match bench.pane {
                Pane::Pack => (at == bench.cursor, None),
                Pane::Workshop => (taken(&line).is_some(), taken(&line)),
            };
            Row { line, lit, uses }
        })
        .collect()
}

/// Returns the last word of the name of a tool: what it is, its picture showing what of.
fn word(name: &str) -> String {
    name.rsplit(' ').next().unwrap_or(name).to_string()
}

/// Returns the six slots of the hotbar of `pack`, wear read at `place`.
fn slots(pack: &Pack, place: &Place) -> Vec<Slot> {
    (0..SLOTS)
        .map(|at| {
            let item = pack.bar.slots[at].and_then(|id| pack.item(id));
            Slot {
                name: item.map(|i| word(&i.name())),
                shape: item.and_then(|i| i.shape()),
                worn: item.map_or(0.0, |i| i.worn(place).0),
                held: at == pack.bar.held,
            }
        })
        .collect()
}

/// Returns what the screen shows for `bench` and `pack` on `ground`.
pub fn scene(bench: &Bench, pack: &Pack, ground: &Ground) -> Scene {
    let stock = pack.stock();
    let (tools, used) = tools(bench, &stock, &ground.place);
    let all = lines(pack, &ground.place, ground.life);
    let detail = detail(pack, &all, bench.cursor, ground);
    Scene {
        rows: rows(bench, all, &used),
        detail,
        tools,
        slots: slots(pack, &ground.place),
        burden: ground.burden,
        status: status(&ground.burden),
    }
}
