//! Tests of harvesting: the right tool takes a part whole, a soft edge tears it, the wrong tool
//! ruins it, and a real animal's parts give tissue by class.

use super::*;
use crate::craft::stage::living;

/// The right tool keeps the kill grade and the whole part; too soft an edge drops a grade and half;
/// the wrong tool or bare hands leave a quarter, poor.
#[test]
fn the_right_tool_gives_the_right_yield() {
    for (class, hardness, used, kill, wanted) in cases() {
        assert_eq!(
            fit(class, hardness, used, kill),
            wanted,
            "{class:?} {used:?}"
        );
    }
}

/// A case of [`fit`]: class, tissue hardness, tool, kill grade, and the share and grade wanted.
type Case = (Class, f32, Option<Tool>, Grade, (f32, Grade));

/// Returns the cases of [`fit`]: skinning, cracking, bare hands and drawing resin.
fn cases() -> [Case; 6] {
    let tool = |intent, hardness| Some(Tool { intent, hardness });
    let (knife, club) = (tool(Intent::Cut, 7.0), tool(Intent::Crush, 2.0));
    let soft = tool(Intent::Cut, 1.0);
    let spear = tool(Intent::Throw, 7.0);
    let (clean, good, poor) = (Grade::Clean, Grade::Good, Grade::Poor);
    [
        (Class::Hide, 1.5, knife, clean, (1.0, clean)),
        (Class::Hide, 1.5, soft, clean, (BLUNT, good)),
        (Class::Hide, 1.5, club, clean, (RUINED, poor)),
        (Class::Cuticle, 4.0, club, good, (1.0, good)),
        (Class::Cuticle, 4.0, None, clean, (RUINED, poor)),
        (Class::Resin, 2.5, spear, clean, (1.0, clean)),
    ]
}

/// Returns a clean cut of part `part` of species `species`.
fn cut(species: usize, part: usize) -> Cut {
    Cut {
        species: species as u16,
        part: part as u8,
        kill: Grade::Clean,
    }
}

/// Every part of every animal of a real planet that gives a class gives a positive mass of it,
/// the right tool, harder than any tissue (Mohs 10 is the cap, so 11), four times what bare hands
/// give; a part that is not there gives nothing.
#[test]
fn real_parts_give_tissue() {
    let (life, seed) = living();
    let mut taken = 0;
    for (index, species) in life.species.iter().enumerate() {
        for part in 0..species.body.parts.len() {
            let cut = cut(index, part);
            let Some(bare) = harvest(&life, seed, cut, None) else {
                continue;
            };
            let Stuff::Tissue { class, .. } = bare.stuff else {
                panic!("a tissue");
            };
            let right = Tool {
                intent: need(class),
                hardness: 11.0,
            };
            let whole = harvest(&life, seed, cut, Some(right)).expect("same part");
            assert!(bare.mass > 0.0 && (whole.mass / bare.mass - 4.0).abs() < 1e-3);
            taken += 1;
        }
    }
    assert!(taken > 0);
    let none = Cut {
        species: u16::MAX,
        ..cut(0, 0)
    };
    assert!(harvest(&life, seed, none, None).is_none());
}
