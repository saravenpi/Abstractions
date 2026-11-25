//! Armor slot definitions, body part coverage, and physical/shield stat derivation from minerals.

use serde::{Deserialize, Serialize};

use crate::condition::Part;
use crate::craft::matter::Mineral;
use crate::craft::pack::mineral_name;

/// One of the four wearable armor slots on the robot frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArmorSlot {
    /// Helmet protecting the head and visor.
    Head,
    /// Chestplate protecting torso, arms, and wings.
    Chest,
    /// Greaves protecting both legs.
    Legs,
    /// Armored boots protecting both feet and boosters.
    Boots,
}

impl ArmorSlot {
    /// Every armor slot in top-to-bottom body order.
    pub const ALL: [ArmorSlot; 4] = [
        ArmorSlot::Head,
        ArmorSlot::Chest,
        ArmorSlot::Legs,
        ArmorSlot::Boots,
    ];

    /// Short uppercase slot tag shown in the inventory armor column.
    pub fn label(self) -> &'static str {
        match self {
            ArmorSlot::Head => "HEAD",
            ArmorSlot::Chest => "CHEST",
            ArmorSlot::Legs => "LEGS",
            ArmorSlot::Boots => "BOOTS",
        }
    }

    /// Piece noun used in display names (`Helmet`, `Chestplate`, `Greaves`, `Boots`).
    pub fn noun(self) -> &'static str {
        match self {
            ArmorSlot::Head => "Helmet",
            ArmorSlot::Chest => "Chestplate",
            ArmorSlot::Legs => "Greaves",
            ArmorSlot::Boots => "Boots",
        }
    }

    /// Returns true if this armor slot mitigates impacts landing on `part`.
    pub fn covers(self, part: Part) -> bool {
        match self {
            ArmorSlot::Head => matches!(part, Part::Head),
            ArmorSlot::Chest => matches!(
                part,
                Part::Torso | Part::ArmL | Part::ArmR | Part::WingL | Part::WingR
            ),
            ArmorSlot::Legs => matches!(part, Part::LegL | Part::LegR),
            ArmorSlot::Boots => matches!(part, Part::BootL | Part::BootR),
        }
    }

    /// Fraction of a full Aegis armor set's shield capacity contributed by this slot.
    pub fn shield_share(self) -> f32 {
        match self {
            ArmorSlot::Chest => 0.38,
            ArmorSlot::Legs => 0.26,
            ArmorSlot::Head => 0.20,
            ArmorSlot::Boots => 0.16,
        }
    }

    /// Base carry mass of an armor piece in kilograms.
    pub fn mass_kg(self) -> f32 {
        match self {
            ArmorSlot::Head => 1.8,
            ArmorSlot::Chest => 4.2,
            ArmorSlot::Legs => 3.0,
            ArmorSlot::Boots => 2.0,
        }
    }
}

/// Crafted mineral plate or shielded Aegis armor piece stored in [`crate::craft::pack::Pack`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Armor {
    /// Pack-unique armor identifier.
    pub id: u32,
    /// Body slot this piece equips into.
    pub slot: ArmorSlot,
    /// Primary mineral forming the armor plating.
    pub mineral: Mineral,
    /// True for shielded Aegis armor granting rechargeable energy shield capacity.
    pub shielded: bool,
    /// Accumulated wear fraction in `0.0..=1.0`.
    pub wear: f32,
}

impl Armor {
    /// Display name combining mineral, optional Aegis tag, and piece noun.
    pub fn name(&self) -> String {
        let raw = mineral_name(self.mineral);
        let mut chars = raw.chars();
        let cap = chars
            .next()
            .map(|c| c.to_ascii_uppercase())
            .unwrap_or_default();
        let tag = if self.shielded { " Aegis " } else { " " };
        format!("{cap}{}{tag}{}", chars.as_str(), self.slot.noun())
    }

    /// Physical damage reduction fraction on covered body parts, clamped to `<= 0.84`.
    pub fn defense(&self) -> f32 {
        let m = self.mineral.material();
        let score =
            (m.strength / 920.0) * 0.60 + (m.toughness / 9.0) * 0.20 + (m.hardness / 10.0) * 0.20;
        let intact = (0.05 + 0.775 * score).clamp(0.05, 0.84);
        let wear_factor = 1.0 - self.wear.clamp(0.0, 0.9) * 0.5;
        (intact * wear_factor).clamp(0.0, 0.84)
    }

    /// Rechargeable energy shield joules granted by this piece (`0.0` when not shielded).
    pub fn shield_joules(&self) -> f32 {
        if !self.shielded {
            return 0.0;
        }
        let m = self.mineral.material();
        let str_term = (m.strength / 920.0).clamp(0.0, 1.0);
        let cond_term = (m.conductivity / 380.0).sqrt().clamp(0.0, 1.0);
        let hard_term = (m.hardness / 10.0).clamp(0.0, 1.0);
        let full_set = 40.0 + str_term * 1020.0 + cond_term * 100.0 + hard_term * 260.0;
        full_set * self.slot.shield_share()
    }
}
