//! Affordability queries, Have/Need counts, and batch crafting execution for [`Blueprint`]s.

use super::catalog::{Blueprint, BlueprintOutput, IngredientNeed, output_material};
use crate::craft::gun::Gun;
use crate::craft::matter::{ChargeKind, Mineral, Stuff};
use crate::craft::pack::{Pack, SlotEntry};

/// Returns total units held in `pack` satisfying `need` (using `chosen` for bullet slug mineral).
pub fn ingredient_have(pack: &Pack, need: &IngredientNeed, chosen: Mineral) -> u32 {
    match *need {
        IngredientNeed::Exact(stuff) => pack.count_of(&stuff),
        IngredientNeed::ChosenMineral => pack.count_of(&Stuff::Mineral(chosen)),
        IngredientNeed::AnyCharge => {
            pack.count_of(&Stuff::Charge(ChargeKind::SparkPowder))
                + pack.count_of(&Stuff::Charge(ChargeKind::BrimstonePowder))
        }
        IngredientNeed::AnyMineral { allowed, .. } => allowed
            .iter()
            .map(|&m| pack.count_of(&Stuff::Mineral(m)))
            .sum(),
    }
}

/// Returns display label and sample [`Stuff`] for an ingredient slot.
pub fn ingredient_sample(need: &IngredientNeed, pack: &Pack, chosen: Mineral) -> (String, Stuff) {
    match *need {
        IngredientNeed::Exact(s) => (crate::craft::pack::called(&s).to_string(), s),
        IngredientNeed::ChosenMineral => (
            crate::craft::pack::called(&Stuff::Mineral(chosen)).to_string(),
            Stuff::Mineral(chosen),
        ),
        IngredientNeed::AnyCharge => {
            let brim = pack.count_of(&Stuff::Charge(ChargeKind::BrimstonePowder));
            let s = if brim > 0 {
                Stuff::Charge(ChargeKind::BrimstonePowder)
            } else {
                Stuff::Charge(ChargeKind::SparkPowder)
            };
            ("powder (Spark / Brimstone)".to_string(), s)
        }
        IngredientNeed::AnyMineral { label, allowed, .. } => {
            let best = allowed
                .iter()
                .copied()
                .max_by_key(|m| pack.count_of(&Stuff::Mineral(*m)))
                .unwrap_or(allowed[0]);
            (label.to_string(), Stuff::Mineral(best))
        }
    }
}

/// Consumes `want` units from `pack` across acceptable alternatives and returns the dominant mineral used.
fn consume_need(
    pack: &mut Pack,
    need: &IngredientNeed,
    want: u16,
    chosen: Mineral,
) -> Option<Mineral> {
    match *need {
        IngredientNeed::Exact(s) => pack.take_count(&s, want).then_some(match s {
            Stuff::Mineral(m) => m,
            _ => Mineral::Ironstone,
        }),
        IngredientNeed::ChosenMineral => pack
            .take_count(&Stuff::Mineral(chosen), want)
            .then_some(chosen),
        IngredientNeed::AnyCharge => {
            let mut rem = want;
            for c in [ChargeKind::SparkPowder, ChargeKind::BrimstonePowder] {
                let s = Stuff::Charge(c);
                let avail = pack.count_of(&s).min(u32::from(rem)) as u16;
                if avail > 0 && pack.take_count(&s, avail) {
                    rem -= avail;
                }
            }
            (rem == 0).then_some(Mineral::Brimstone)
        }
        IngredientNeed::AnyMineral { allowed, .. } => {
            let mut rem = want;
            let mut primary = allowed[0];
            let mut best = 0;
            for &m in allowed {
                let s = Stuff::Mineral(m);
                let avail = pack.count_of(&s).min(u32::from(rem)) as u16;
                if avail > 0 && pack.take_count(&s, avail) {
                    if avail > best {
                        best = avail;
                        primary = m;
                    }
                    rem -= avail;
                }
            }
            (rem == 0).then_some(primary)
        }
    }
}

/// Returns how many batches of `bp` can currently be crafted given ingredients in `pack`.
pub fn max_craftable(bp: &Blueprint, pack: &Pack, chosen: Mineral) -> u16 {
    let from_inputs = bp
        .inputs
        .iter()
        .map(|i| ingredient_have(pack, &i.need, chosen) / u32::from(i.count.max(1)))
        .min()
        .unwrap_or(0);
    from_inputs.min(99) as u16
}

/// Crafts `times` batches of `bp` inside `pack`, returning status notice or reason on failure.
pub fn craft_blueprint(
    bp: &Blueprint,
    pack: &mut Pack,
    chosen: Mineral,
    times: u16,
) -> Result<String, &'static str> {
    let n = times.clamp(1, max_craftable(bp, pack, chosen).max(1));
    if max_craftable(bp, pack, chosen) < n {
        return Err("Missing ingredients for this recipe.");
    }
    let mut staged = pack.clone();
    let mut barrel = Mineral::Ironstone;
    for (idx, ing) in bp.inputs.iter().enumerate() {
        let total = ing.count.saturating_mul(n);
        let picked = consume_need(&mut staged, &ing.need, total, chosen)
            .ok_or("Missing ingredients for this recipe.")?;
        if idx == 1 {
            barrel = picked;
        }
    }
    match bp.output {
        BlueprintOutput::Stack { stuff, count } => {
            let total = count.saturating_mul(n);
            if staged.room_for(&stuff) < u32::from(total) {
                return Err("Pack full: no free stack room for output.");
            }
            staged.add_stack(stuff, output_material(&stuff), total);
            *pack = staged;
            Ok(format!("Crafted {}x {}.", total, bp.name))
        }
        BlueprintOutput::ChosenBullet(count) => {
            let stuff = Stuff::Bullet(chosen);
            let total = count.saturating_mul(n);
            if staged.room_for(&stuff) < u32::from(total) {
                return Err("Pack full: no free stack room for bullets.");
            }
            staged.add_stack(stuff, output_material(&stuff), total);
            *pack = staged;
            Ok(format!(
                "Crafted {}x {}.",
                total,
                crate::craft::pack::called(&stuff)
            ))
        }
        BlueprintOutput::Firearm(kind) => {
            for _ in 0..n {
                let slot = staged
                    .grid
                    .iter()
                    .position(Option::is_none)
                    .ok_or("Pack full: free a slot for the firearm.")?;
                let id = staged.next;
                staged.next = staged.next.wrapping_add(1);
                staged.guns.push(Gun::new(id, kind, barrel));
                staged.grid[slot] = Some(SlotEntry::Gun(id));
                staged.sync_views();
            }
            *pack = staged;
            Ok(format!("Crafted {}x {}.", n, bp.name))
        }
    }
}
