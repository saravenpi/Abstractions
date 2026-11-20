//! The rows of the pack: each tool and each stack of material, with what the list shows of it, and
//! the filter and order that arrange them.

use super::bench::Sort;
use crate::computer::sift::matches;
use crate::craft::build::{Form, Place};
use crate::craft::matter::{Grade, Stuff, Treatment};
use crate::craft::pack::{Pack, Stack, called};
use crate::life::Life;

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
    /// Its name.
    pub name: String,
    /// What it is made of, or how good it is.
    pub signature: String,
    /// Its mass in kilograms.
    pub mass: f32,
    /// How worn a tool is, 0 new to 1 broken; nothing for a material.
    pub wear: Option<f32>,
    /// Shaft length, head size and form of a tool, in metres.
    pub shape: Option<(f32, f32, Form)>,
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

/// Returns a row for every tool and stack in `pack`, tools first, their wear read at `place`.
pub fn lines(pack: &Pack, place: &Place, life: Option<(&Life, u64)>) -> Vec<Line> {
    let tools = pack.items.iter().map(|item| Line {
        what: What::Item(item.id),
        name: item.name(),
        signature: item.signature(),
        mass: item.mass(),
        wear: Some(item.worn(place).0),
        shape: item.shape(),
    });
    let stacks = pack.stacks.iter().enumerate().map(|(at, stack)| Line {
        what: What::Stack(at),
        name: stack_name(stack, life),
        signature: stack_signature(&stack.stuff),
        mass: stack.mass,
        wear: None,
        shape: None,
    });
    tools.chain(stacks).collect()
}

/// Returns the rows whose name or signature holds `filter`, in the order `sort` asks for.
pub fn arrange(mut lines: Vec<Line>, filter: &str, sort: Sort) -> Vec<Line> {
    lines.retain(|line| matches(&line.name, filter) || matches(&line.signature, filter));
    match sort {
        Sort::Kept => {}
        Sort::Name => lines.sort_by_key(|line| line.name.to_lowercase()),
        Sort::Mass => lines.sort_by(|a, b| b.mass.total_cmp(&a.mass)),
    }
    lines
}
