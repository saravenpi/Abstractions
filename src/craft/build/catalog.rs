//! Explicit categorized crafting catalog with clear ingredient requirements and gathering hints.

use crate::craft::armor::ArmorSlot;
use crate::craft::gun::GunKind;
use crate::craft::matter::{ChargeKind, Material, Mineral, PlaceableKind, Stuff, WoodKind};

/// Filter tabs shown at the top of the Crafting Workshop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Category {
    /// Every explicit recipe regardless of affordability.
    #[default]
    All,
    /// Only recipes whose ingredients are currently satisfied in the pack.
    Craftable,
    /// Lumber milling, charcoal, and propellant powders.
    Basics,
    /// Firearms and mineral ammunition.
    Weapons,
    /// Wearable mineral plate and shielded Aegis armor sets.
    Armor,
    /// Placeable storage chests and torches.
    Base,
    /// Procedural melee and harvesting tool solver (knives, axes, spears, pickaxes).
    Tools,
}

impl Category {
    /// Ordered filter bar tabs shown in the workshop.
    pub const ALL: [Category; 7] = [
        Category::All,
        Category::Craftable,
        Category::Basics,
        Category::Weapons,
        Category::Armor,
        Category::Base,
        Category::Tools,
    ];

    /// Short display label for the category tab button.
    pub fn label(self) -> &'static str {
        match self {
            Category::All => "All",
            Category::Craftable => "Craftable",
            Category::Basics => "Basics",
            Category::Weapons => "Weapons",
            Category::Armor => "Armor",
            Category::Base => "Base",
            Category::Tools => "Tools",
        }
    }
}

/// One material slot required by an explicit [`Blueprint`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IngredientNeed {
    /// Exact item variant (`WoodKind::Plank`, `Mineral::Flint`, etc.).
    Exact(Stuff),
    /// Any mineral from `allowed` (e.g. barrel ore options).
    AnyMineral {
        /// Human-readable group name shown on the checklist.
        label: &'static str,
        /// Acceptable minerals in preference order.
        allowed: &'static [Mineral],
        /// Gathering hint when player lacks all allowed minerals.
        hint: &'static str,
    },
    /// Either Spark Powder or Brimstone Powder.
    AnyCharge,
    /// Whichever mineral is currently selected in the bullet mineral selector.
    ChosenMineral,
}

/// One ingredient row of a recipe: requirement and unit count per craft.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ingredient {
    /// Required material descriptor.
    pub need: IngredientNeed,
    /// Units consumed per single craft.
    pub count: u16,
}

/// Item produced by one craft of a [`Blueprint`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlueprintOutput {
    /// Fixed discrete stack (`count` units of `stuff`).
    Stack {
        /// Output item variant.
        stuff: Stuff,
        /// Units created per craft.
        count: u16,
    },
    /// Batch of mineral bullets matching the selected bullet mineral.
    ChosenBullet(u16),
    /// Crafted flintlock pistol or mineral rifle with barrel matching primary ore used.
    Firearm(GunKind),
    /// Crafted mineral plate or shielded Aegis armor piece forged from the highest-tier carried ore.
    Armor {
        /// Body slot the crafted piece equips into.
        slot: ArmorSlot,
        /// True when fitted with copper coils and a crystal shield emitter.
        shielded: bool,
    },
}

/// Explicit named crafting recipe displayed in the Crafting Workshop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Blueprint {
    /// Unique stable recipe identifier.
    pub id: &'static str,
    /// Display title.
    pub name: &'static str,
    /// Short utility description for the inspector card.
    pub summary: &'static str,
    /// Workshop category tab.
    pub category: Category,
    /// Required ingredients in display order.
    pub inputs: &'static [Ingredient],
    /// Produced stack or firearm.
    pub output: BlueprintOutput,
}

/// Returns mechanical properties of non-tissue output items.
pub(super) fn output_material(stuff: &Stuff) -> Material {
    match *stuff {
        Stuff::Mineral(m) | Stuff::Bullet(m) => m.material(),
        Stuff::Wood(w) => w.material(),
        Stuff::Charge(c) => c.material(),
        Stuff::Placeable(p) => p.material(),
        Stuff::Tissue { .. } => WoodKind::Log.material(),
    }
}

/// Returns a helpful location hint telling the player where to obtain `need`.
pub fn missing_hint(need: &IngredientNeed, chosen: Mineral) -> String {
    match *need {
        IngredientNeed::Exact(Stuff::Mineral(m)) => m.where_found().to_string(),
        IngredientNeed::ChosenMineral => chosen.where_found().to_string(),
        IngredientNeed::Exact(Stuff::Wood(WoodKind::Log)) => "Chop trees with H".to_string(),
        IngredientNeed::Exact(Stuff::Wood(_)) => "Craft in Basics tab".to_string(),
        IngredientNeed::Exact(Stuff::Charge(_)) | IngredientNeed::AnyCharge => {
            "Craft Spark or Brimstone Powder in Basics".to_string()
        }
        IngredientNeed::Exact(Stuff::Placeable(_)) => "Craft in Base tab".to_string(),
        IngredientNeed::Exact(Stuff::Bullet(_)) => "Craft in Weapons tab".to_string(),
        IngredientNeed::Exact(Stuff::Tissue { .. }) => "Hunt creatures for parts".to_string(),
        IngredientNeed::AnyMineral { hint, .. } => hint.to_string(),
    }
}

const PISTOL_BARRELS: &[Mineral] = &[
    Mineral::Ironstone,
    Mineral::Copper,
    Mineral::Basalt,
    Mineral::Siderite,
];
const RIFLE_BARRELS: &[Mineral] = &[
    Mineral::Cobaltite,
    Mineral::Ilmenite,
    Mineral::Galena,
    Mineral::Ironstone,
    Mineral::Copper,
];
const RIFLE_SIGHTS: &[Mineral] = &[Mineral::Copper, Mineral::Quartz, Mineral::Amethyst];
const CHEST_BINDINGS: &[Mineral] = &[Mineral::Basalt, Mineral::Copper, Mineral::Ironstone];
const SPARK_ORE: &[Mineral] = &[Mineral::Flint, Mineral::Pyrite, Mineral::Quartz];
const BRIM_ORE: &[Mineral] = &[Mineral::Brimstone, Mineral::Pyrite, Mineral::Cinnabar];
const ARMOR_PLATES: &[Mineral] = &[
    Mineral::Lonsdaleite,
    Mineral::Osmium,
    Mineral::Moissanite,
    Mineral::Wolframite,
    Mineral::Corundum,
    Mineral::Chromite,
    Mineral::Ilmenite,
    Mineral::Cobaltite,
    Mineral::Siderite,
    Mineral::Ironstone,
    Mineral::Jade,
    Mineral::Copper,
    Mineral::Basalt,
];
const AEGIS_CORES: &[Mineral] = &[
    Mineral::Lonsdaleite,
    Mineral::Moissanite,
    Mineral::Corundum,
    Mineral::Amethyst,
    Mineral::Quartz,
    Mineral::Cryolite,
];

/// All 18 explicit crafting recipes ordered by category progression.
const CATALOG: &[Blueprint] = &[
    Blueprint {
        id: "planks",
        name: "Wood Planks",
        summary: "Milled lumber boards for chests, grips, and rifle stocks.",
        category: Category::Basics,
        inputs: &[Ingredient {
            need: IngredientNeed::Exact(Stuff::Wood(WoodKind::Log)),
            count: 1,
        }],
        output: BlueprintOutput::Stack {
            stuff: Stuff::Wood(WoodKind::Plank),
            count: 4,
        },
    },
    Blueprint {
        id: "sticks",
        name: "Sticks",
        summary: "Shaped wooden shafts for torches and hand tools.",
        category: Category::Basics,
        inputs: &[Ingredient {
            need: IngredientNeed::Exact(Stuff::Wood(WoodKind::Plank)),
            count: 2,
        }],
        output: BlueprintOutput::Stack {
            stuff: Stuff::Wood(WoodKind::Stick),
            count: 4,
        },
    },
    Blueprint {
        id: "charcoal",
        name: "Charcoal",
        summary: "Slow-charred carbon used in torches and gunpowder.",
        category: Category::Basics,
        inputs: &[Ingredient {
            need: IngredientNeed::Exact(Stuff::Wood(WoodKind::Log)),
            count: 2,
        }],
        output: BlueprintOutput::Stack {
            stuff: Stuff::Wood(WoodKind::Charcoal),
            count: 3,
        },
    },
    Blueprint {
        id: "spark_powder",
        name: "Spark Powder",
        summary: "Fast flint-spark propellant charge for mineral bullets.",
        category: Category::Basics,
        inputs: &[
            Ingredient {
                need: IngredientNeed::Exact(Stuff::Wood(WoodKind::Charcoal)),
                count: 2,
            },
            Ingredient {
                need: IngredientNeed::AnyMineral {
                    label: "Flint, Pyrite or Quartz",
                    allowed: SPARK_ORE,
                    hint: "Break Flint/Quartz rocks or dig Pyrite >= 0.8m",
                },
                count: 1,
            },
        ],
        output: BlueprintOutput::Stack {
            stuff: Stuff::Charge(ChargeKind::SparkPowder),
            count: 4,
        },
    },
    Blueprint {
        id: "brimstone_powder",
        name: "Brimstone Powder",
        summary: "High-yield volcanic gunpowder rich in sulfur.",
        category: Category::Basics,
        inputs: &[
            Ingredient {
                need: IngredientNeed::Exact(Stuff::Wood(WoodKind::Charcoal)),
                count: 2,
            },
            Ingredient {
                need: IngredientNeed::AnyMineral {
                    label: "Brimstone, Pyrite or Cinnabar",
                    allowed: BRIM_ORE,
                    hint: "Mine Lava worlds or dig Cinnabar >= 1.0m",
                },
                count: 1,
            },
        ],
        output: BlueprintOutput::Stack {
            stuff: Stuff::Charge(ChargeKind::BrimstonePowder),
            count: 6,
        },
    },
    Blueprint {
        id: "chest",
        name: "Storage Chest",
        summary: "Placeable 24-slot container shared across players in solo, LAN & online.",
        category: Category::Base,
        inputs: &[
            Ingredient {
                need: IngredientNeed::Exact(Stuff::Wood(WoodKind::Plank)),
                count: 8,
            },
            Ingredient {
                need: IngredientNeed::AnyMineral {
                    label: "Basalt, Copper or Ironstone",
                    allowed: CHEST_BINDINGS,
                    hint: "Mine surface Basalt/Ironstone or dig Copper >= 0.6m",
                },
                count: 2,
            },
        ],
        output: BlueprintOutput::Stack {
            stuff: Stuff::Placeable(PlaceableKind::Chest),
            count: 1,
        },
    },
    Blueprint {
        id: "torch",
        name: "Torch",
        summary: "Flickering campfire torch to light caves, dig sites, and bases.",
        category: Category::Base,
        inputs: &[
            Ingredient {
                need: IngredientNeed::Exact(Stuff::Wood(WoodKind::Stick)),
                count: 1,
            },
            Ingredient {
                need: IngredientNeed::Exact(Stuff::Wood(WoodKind::Charcoal)),
                count: 1,
            },
        ],
        output: BlueprintOutput::Stack {
            stuff: Stuff::Placeable(PlaceableKind::Torch),
            count: 4,
        },
    },
    Blueprint {
        id: "bullets",
        name: "Mineral Bullets",
        summary: "Dense mineral slugs forged with powder for pistol and rifle.",
        category: Category::Weapons,
        inputs: &[
            Ingredient {
                need: IngredientNeed::AnyCharge,
                count: 1,
            },
            Ingredient {
                need: IngredientNeed::ChosenMineral,
                count: 1,
            },
        ],
        output: BlueprintOutput::ChosenBullet(8),
    },
    Blueprint {
        id: "flintlock_pistol",
        name: "Flintlock Pistol",
        summary: "One-handed single-shot mineral sidearm. Reliable starter firearm.",
        category: Category::Weapons,
        inputs: &[
            Ingredient {
                need: IngredientNeed::Exact(Stuff::Wood(WoodKind::Plank)),
                count: 2,
            },
            Ingredient {
                need: IngredientNeed::AnyMineral {
                    label: "Ironstone, Copper, Basalt or Siderite",
                    allowed: PISTOL_BARRELS,
                    hint: "Surface Basalt/Ironstone or dig Copper >= 0.6m",
                },
                count: 2,
            },
            Ingredient {
                need: IngredientNeed::Exact(Stuff::Mineral(Mineral::Flint)),
                count: 1,
            },
        ],
        output: BlueprintOutput::Firearm(GunKind::FlintlockPistol),
    },
    Blueprint {
        id: "mineral_rifle",
        name: "Mineral Rifle",
        summary: "High-velocity 4-round bolt-action rifle with crystal sights.",
        category: Category::Weapons,
        inputs: &[
            Ingredient {
                need: IngredientNeed::Exact(Stuff::Wood(WoodKind::Plank)),
                count: 3,
            },
            Ingredient {
                need: IngredientNeed::AnyMineral {
                    label: "Cobaltite, Ilmenite, Galena, Ironstone or Copper",
                    allowed: RIFLE_BARRELS,
                    hint: "Dig underground seams (>= 0.6-1.8m) for deep ores",
                },
                count: 3,
            },
            Ingredient {
                need: IngredientNeed::AnyMineral {
                    label: "Copper, Quartz or Amethyst",
                    allowed: RIFLE_SIGHTS,
                    hint: "Surface Quartz or dig Copper/Amethyst",
                },
                count: 2,
            },
        ],
        output: BlueprintOutput::Firearm(GunKind::MineralRifle),
    },
    Blueprint {
        id: "armor_helmet",
        name: "Mineral Helmet",
        summary: "Head plating forged from your strongest carried mineral, up to Lonsdaleite.",
        category: Category::Armor,
        inputs: &[Ingredient {
            need: IngredientNeed::AnyMineral {
                label: "Armor ore (best tier used)",
                allowed: ARMOR_PLATES,
                hint: "Mine Basalt, Ironstone, or deeper high-tier ores up to Lonsdaleite",
            },
            count: 4,
        }],
        output: BlueprintOutput::Armor {
            slot: ArmorSlot::Head,
            shielded: false,
        },
    },
    Blueprint {
        id: "armor_chestplate",
        name: "Mineral Chestplate",
        summary: "Heavy torso, arm, and wing armor forged from your strongest carried mineral.",
        category: Category::Armor,
        inputs: &[Ingredient {
            need: IngredientNeed::AnyMineral {
                label: "Armor ore (best tier used)",
                allowed: ARMOR_PLATES,
                hint: "Mine Basalt, Ironstone, or deeper high-tier ores up to Lonsdaleite",
            },
            count: 7,
        }],
        output: BlueprintOutput::Armor {
            slot: ArmorSlot::Chest,
            shielded: false,
        },
    },
    Blueprint {
        id: "armor_greaves",
        name: "Mineral Greaves",
        summary: "Articulated leg plates forged from your strongest carried mineral.",
        category: Category::Armor,
        inputs: &[Ingredient {
            need: IngredientNeed::AnyMineral {
                label: "Armor ore (best tier used)",
                allowed: ARMOR_PLATES,
                hint: "Mine Basalt, Ironstone, or deeper high-tier ores up to Lonsdaleite",
            },
            count: 5,
        }],
        output: BlueprintOutput::Armor {
            slot: ArmorSlot::Legs,
            shielded: false,
        },
    },
    Blueprint {
        id: "armor_boots",
        name: "Mineral Boots",
        summary: "Reinforced booster boots forged from your strongest carried mineral.",
        category: Category::Armor,
        inputs: &[Ingredient {
            need: IngredientNeed::AnyMineral {
                label: "Armor ore (best tier used)",
                allowed: ARMOR_PLATES,
                hint: "Mine Basalt, Ironstone, or deeper high-tier ores up to Lonsdaleite",
            },
            count: 4,
        }],
        output: BlueprintOutput::Armor {
            slot: ArmorSlot::Boots,
            shielded: false,
        },
    },
    Blueprint {
        id: "aegis_helmet",
        name: "Aegis Shielded Helmet",
        summary: "Energy-shielded helmet with copper coil and crystal emitter (up to Lonsdaleite Aegis).",
        category: Category::Armor,
        inputs: &[
            Ingredient {
                need: IngredientNeed::AnyMineral {
                    label: "Armor ore (best tier used)",
                    allowed: ARMOR_PLATES,
                    hint: "Mine higher-tier ores up to Lonsdaleite",
                },
                count: 4,
            },
            Ingredient {
                need: IngredientNeed::Exact(Stuff::Mineral(Mineral::Copper)),
                count: 2,
            },
            Ingredient {
                need: IngredientNeed::AnyMineral {
                    label: "Crystal core (Quartz to Lonsdaleite)",
                    allowed: AEGIS_CORES,
                    hint: "Mine Quartz, Amethyst, Corundum, Moissanite or Lonsdaleite",
                },
                count: 2,
            },
        ],
        output: BlueprintOutput::Armor {
            slot: ArmorSlot::Head,
            shielded: true,
        },
    },
    Blueprint {
        id: "aegis_chestplate",
        name: "Aegis Shielded Chestplate",
        summary: "Primary energy shield reactor chestplate (38% of full Aegis shield capacity).",
        category: Category::Armor,
        inputs: &[
            Ingredient {
                need: IngredientNeed::AnyMineral {
                    label: "Armor ore (best tier used)",
                    allowed: ARMOR_PLATES,
                    hint: "Mine higher-tier ores up to Lonsdaleite",
                },
                count: 7,
            },
            Ingredient {
                need: IngredientNeed::Exact(Stuff::Mineral(Mineral::Copper)),
                count: 3,
            },
            Ingredient {
                need: IngredientNeed::AnyMineral {
                    label: "Crystal core (Quartz to Lonsdaleite)",
                    allowed: AEGIS_CORES,
                    hint: "Mine Quartz, Amethyst, Corundum, Moissanite or Lonsdaleite",
                },
                count: 3,
            },
        ],
        output: BlueprintOutput::Armor {
            slot: ArmorSlot::Chest,
            shielded: true,
        },
    },
    Blueprint {
        id: "aegis_greaves",
        name: "Aegis Shielded Greaves",
        summary: "Shield-linked leg armor projecting an electric cyan field around the lower frame.",
        category: Category::Armor,
        inputs: &[
            Ingredient {
                need: IngredientNeed::AnyMineral {
                    label: "Armor ore (best tier used)",
                    allowed: ARMOR_PLATES,
                    hint: "Mine higher-tier ores up to Lonsdaleite",
                },
                count: 5,
            },
            Ingredient {
                need: IngredientNeed::Exact(Stuff::Mineral(Mineral::Copper)),
                count: 2,
            },
            Ingredient {
                need: IngredientNeed::AnyMineral {
                    label: "Crystal core (Quartz to Lonsdaleite)",
                    allowed: AEGIS_CORES,
                    hint: "Mine Quartz, Amethyst, Corundum, Moissanite or Lonsdaleite",
                },
                count: 2,
            },
        ],
        output: BlueprintOutput::Armor {
            slot: ArmorSlot::Legs,
            shielded: true,
        },
    },
    Blueprint {
        id: "aegis_boots",
        name: "Aegis Shielded Boots",
        summary: "Shield-grounded booster boots completing the rechargeable Aegis armor set.",
        category: Category::Armor,
        inputs: &[
            Ingredient {
                need: IngredientNeed::AnyMineral {
                    label: "Armor ore (best tier used)",
                    allowed: ARMOR_PLATES,
                    hint: "Mine higher-tier ores up to Lonsdaleite",
                },
                count: 4,
            },
            Ingredient {
                need: IngredientNeed::Exact(Stuff::Mineral(Mineral::Copper)),
                count: 2,
            },
            Ingredient {
                need: IngredientNeed::AnyMineral {
                    label: "Crystal core (Quartz to Lonsdaleite)",
                    allowed: AEGIS_CORES,
                    hint: "Mine Quartz, Amethyst, Corundum, Moissanite or Lonsdaleite",
                },
                count: 2,
            },
        ],
        output: BlueprintOutput::Armor {
            slot: ArmorSlot::Boots,
            shielded: true,
        },
    },
];

/// Returns all explicit blueprints in the Crafting Workshop catalog.
pub fn catalog() -> &'static [Blueprint] {
    CATALOG
}
