//! Tests of the pack detail inspector: stone and tissue stacks show their four material bars,
//! tools show their four tool bars with condition, joint life and hotbar slot, and an empty pack
//! yields no detail.

use super::bench::Bench;
use super::detail::Detail;
use super::lines::What;
use super::rows::fact;
use super::scene::{Ground, scene};
use crate::craft::build::{Intent, solve};
use crate::craft::pack::{Burden, Pack};
use crate::craft::stage::{earth, pack};

/// Returns the ground of a pack of `mass` kilograms on Earth.
fn ground(mass: f32) -> Ground<'static> {
    Ground {
        place: earth(),
        life: None,
        burden: Burden::of(mass, 9.81),
    }
}

/// Returns the detail card of the row named `name` in `carried`.
fn row_detail(carried: &Pack, name: &str) -> Detail {
    let base = scene(&Bench::default(), carried, &ground(carried.mass()));
    let at = base.rows.iter().position(|r| r.line.name == name);
    let bench = Bench {
        cursor: at.expect("matching row"),
        ..Bench::default()
    };
    scene(&bench, carried, &ground(carried.mass()))
        .detail
        .expect("row detail")
}

/// Selecting a stone stack gives its name, mass share, icon and four ASCII material bars.
#[test]
fn stone_stack_shows_material_gauges() {
    let carried = pack();
    let detail = row_detail(&carried, "flint");
    assert_eq!(detail.name, "flint");
    assert_eq!(detail.signature, "stone");
    assert!((detail.mass - 1.5).abs() < 1e-5);
    assert!((detail.load_share - 1.5 / carried.mass()).abs() < 1e-5);
    assert!(detail.icon.is_some() && detail.shape.is_none());
    assert!(detail.condition.is_none() && detail.life.is_none() && detail.slot.is_none());
    let pairs: Vec<(&str, &str)> = detail
        .gauges
        .iter()
        .map(|g| (g.label, g.value.as_str()))
        .collect();
    assert_eq!(
        pairs,
        [
            ("Hardness", "7.0 Mohs"),
            ("Toughness", "1.50 MPa"),
            ("Strength", "100 MPa"),
            ("Stiffness", "80.0 GPa"),
        ]
    );
    assert!(
        detail
            .gauges
            .iter()
            .all(|g| (0.0..=1.0).contains(&g.share) && g.label.is_ascii() && g.value.is_ascii())
    );
}

/// Selecting a made tool gives its tool gauges, condition, joint life and hotbar slot, and its row
/// fact leads with `#N`.
#[test]
fn tool_shows_tool_gauges_condition_and_slot() {
    let mut carried = pack();
    let stock = carried.stock();
    let made = solve(Intent::Cut, &stock, &earth()).expect("a cut");
    let id = carried.make(&made.hardest, &stock).expect("made");
    let slot = carried
        .bar
        .slots
        .iter()
        .position(|s| *s == Some(id))
        .expect("slotted")
        + 1;
    let shown = scene(&Bench::default(), &carried, &ground(carried.mass()));
    assert_eq!(shown.rows[0].line.what, What::Item(id));
    assert!(fact(&shown.rows[0]).starts_with(&format!("#{slot}, ")));
    let detail = shown.detail.expect("tool detail");
    assert!(detail.shape.is_some() && detail.icon.is_none());
    assert_eq!(detail.condition, Some(1.0));
    assert!(detail.life.is_some_and(|life| life > 0.0));
    assert_eq!(detail.slot, Some(slot));
    let labels: Vec<&str> = detail.gauges.iter().map(|g| g.label).collect();
    assert_eq!(labels, ["Reach", "Mass", "Edge", "Swing"]);
    assert!(detail.gauges.iter().all(|g| (0.0..=1.0).contains(&g.share)));
}

/// An empty pack produces no detail card.
#[test]
fn empty_pack_has_no_detail() {
    let carried = Pack::default();
    let shown = scene(&Bench::default(), &carried, &ground(0.0));
    assert!(shown.detail.is_none());
}
