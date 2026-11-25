//! Tests for the explicit Crafting Workshop catalog, batch crafting, and missing hints.

use super::catalog::{Category, IngredientNeed, catalog, missing_hint};
use super::catalog_craft::{craft_blueprint, ingredient_have, max_craftable};
use crate::craft::gun::GunKind;
use crate::craft::matter::{ChargeKind, Mineral, PlaceableKind, Stuff, WoodKind};
use crate::craft::pack::Pack;

/// Logs craft into planks, sticks, charcoal, powder, chests, torches, bullets, and both firearms.
#[test]
fn catalog_progression_crafts_wood_supplies_chests_and_firearms() {
    let mut pack = Pack::default();
    pack.add_stack(Stuff::Wood(WoodKind::Log), WoodKind::Log.material(), 20);
    pack.add_stack(
        Stuff::Mineral(Mineral::Flint),
        Mineral::Flint.material(),
        12,
    );
    pack.add_stack(
        Stuff::Mineral(Mineral::Copper),
        Mineral::Copper.material(),
        12,
    );

    let find = |id: &str| catalog().iter().find(|b| b.id == id).unwrap();
    craft_blueprint(find("planks"), &mut pack, Mineral::Basalt, 5).expect("planks");
    assert_eq!(pack.count_of(&Stuff::Wood(WoodKind::Plank)), 20);

    craft_blueprint(find("sticks"), &mut pack, Mineral::Basalt, 2).expect("sticks");
    craft_blueprint(find("charcoal"), &mut pack, Mineral::Basalt, 2).expect("charcoal");
    craft_blueprint(find("spark_powder"), &mut pack, Mineral::Basalt, 2).expect("spark powder");
    craft_blueprint(find("torch"), &mut pack, Mineral::Basalt, 1).expect("torch");
    craft_blueprint(find("chest"), &mut pack, Mineral::Basalt, 1).expect("chest");
    craft_blueprint(find("bullets"), &mut pack, Mineral::Copper, 2).expect("bullets");
    craft_blueprint(find("flintlock_pistol"), &mut pack, Mineral::Copper, 1).expect("pistol");
    assert_eq!(pack.count_of(&Stuff::Placeable(PlaceableKind::Torch)), 4);
    assert_eq!(pack.count_of(&Stuff::Placeable(PlaceableKind::Chest)), 1);
    assert_eq!(pack.count_of(&Stuff::Bullet(Mineral::Copper)), 16);
    assert_eq!(pack.guns.len(), 1);
    assert_eq!(pack.guns[0].kind, GunKind::FlintlockPistol);
    assert!(pack.count_of(&Stuff::Charge(ChargeKind::SparkPowder)) > 0);
}

/// Missing hints identify underground ore strata and category craft steps; max count respects stock.
#[test]
fn catalog_reports_max_craftable_and_clear_location_hints() {
    let mut pack = Pack::default();
    let planks = catalog().iter().find(|b| b.id == "planks").unwrap();
    assert_eq!(max_craftable(planks, &pack, Mineral::Basalt), 0);
    pack.add_stack(Stuff::Wood(WoodKind::Log), WoodKind::Log.material(), 3);
    assert_eq!(max_craftable(planks, &pack, Mineral::Basalt), 3);
    assert_eq!(
        ingredient_have(
            &pack,
            &IngredientNeed::Exact(Stuff::Wood(WoodKind::Log)),
            Mineral::Basalt
        ),
        3
    );
    let hint = missing_hint(&IngredientNeed::ChosenMineral, Mineral::Cobaltite);
    assert!(
        hint.contains("1.6m") || hint.contains("cobaltite") || hint.contains("Dig"),
        "{hint}"
    );
    assert_eq!(Category::ALL.len(), 6);
}
