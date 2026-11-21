//! The rows of the pack: each tool and each stack of material, with what the list shows of it,
//! grouped by kind.

use std::cmp::Ordering;

use super::kind::{Kind, material_icon};
use crate::craft::build::{Look, Place};
use crate::craft::matter::{Grade, Stuff, Treatment};
use crate::craft::pack::{Pack, Stack, called};
use crate::life::Life;
use crate::ui::Icon;

/// What a row stands for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum What {
    /// The tool with this id.
    Item(u32),
    /// The stack at this index.
    Stack(usize),
}

/// One row of the pack.
#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    /// What it stands for.
    pub what: What,
    /// The group it is shown in.
    pub kind: Kind,
    /// The icon of a material, from the icon font; a tool shows its picture instead.
    pub icon: Option<Icon>,
    /// Its name.
    pub name: String,
    /// What it is made of, or how good it is.
    pub signature: String,
    /// Its mass in kilograms.
    pub mass: f32,
    /// How worn a tool is, 0 new to 1 broken; nothing for a material.
    pub wear: Option<f32>,
    /// The look the picture of a tool is drawn from; nothing for a material.
    pub shape: Option<Look>,
}

/// Returns the name of a treatment as the pack shows it.
pub fn treated(treatment: Treatment) -> &'static str {
    match treatment {
        Treatment::SunDry => "sun-dried",
        Treatment::FireHarden => "fire-hardened",
        Treatment::FreezeSet => "freeze-set",
    }
}

/// Returns the name of a stack: the species and the class of a tissue from the planet of `life`,
/// else the class or the stone.
fn stack_name(stack: &Stack, life: Option<(&Life, u64)>) -> String {
    let kind = called(&stack.stuff);
    let Stuff::Tissue {
        planet, species, ..
    } = stack.stuff
    else {
        return kind.to_string();
    };
    let named = life.filter(|(_, seed)| *seed == planet);
    let species = named.and_then(|(life, _)| life.species.get(usize::from(species)));
    species.map_or_else(|| kind.to_string(), |s| format!("{} {kind}", s.name))
}

/// Returns the signature of a stack: the grade and the treatments of a tissue, or "stone".
fn stack_signature(stuff: &Stuff) -> String {
    let Stuff::Tissue {
        grade, treated: t, ..
    } = stuff
    else {
        return "stone".to_string();
    };
    let grade = match grade {
        Grade::Clean => "clean",
        Grade::Good => "good",
        Grade::Poor => "poor",
    };
    let done = Treatment::ALL
        .into_iter()
        .filter(|x| t.0 & 1 << *x as u8 != 0);
    std::iter::once(grade)
        .chain(done.map(treated))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Returns the order of two rows of one group: tools as they were made, materials heaviest first.
fn order(a: &Line, b: &Line) -> Ordering {
    let tools = matches!(a.what, What::Item(_)) && matches!(b.what, What::Item(_));
    match tools {
        true => Ordering::Equal,
        false => b.mass.total_cmp(&a.mass),
    }
}

/// Returns a row for every tool and stack in `pack`, grouped by kind in [`Kind::ALL`] order, tools
/// first, their wear read at `place`.
pub fn lines(pack: &Pack, place: &Place, life: Option<(&Life, u64)>) -> Vec<Line> {
    let tools = pack.items.iter().map(|item| Line {
        what: What::Item(item.id),
        kind: Kind::Tool,
        icon: None,
        name: item.name(),
        signature: item.signature(),
        mass: item.mass(),
        wear: Some(item.worn(place).0),
        shape: item.shape(),
    });
    let stacks = pack.stacks.iter().enumerate().map(|(at, stack)| Line {
        what: What::Stack(at),
        kind: Kind::of(&stack.stuff),
        icon: Some(material_icon(&stack.stuff)),
        name: stack_name(stack, life),
        signature: stack_signature(&stack.stuff),
        mass: stack.mass,
        wear: None,
        shape: None,
    });
    let mut all: Vec<Line> = tools.chain(stacks).collect();
    all.sort_by(|a, b| a.kind.cmp(&b.kind).then_with(|| order(a, b)));
    all
}
