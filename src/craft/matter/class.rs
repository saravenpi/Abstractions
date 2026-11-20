//! Tissue classes: the materials a harvested tissue turns into, each anchored on a measured one.

use serde::{Deserialize, Serialize};

use super::Material;
use crate::life::TissueKind;
use crate::life::body::PartKind;

/// What a harvested tissue is, as a material.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Class {
    /// Bone from a skeleton.
    Bone,
    /// Bone grown as a weapon: horns and tusks of a bony species.
    Antler,
    /// Hardened hide: horn sheaths and scutes.
    Keratin,
    /// The shell of a segmented body.
    Cuticle,
    /// Skin and the sinew under it.
    Hide,
    /// Plant fibre, as wood.
    Wood,
    /// Biogenic silica: glassy crystal tissue.
    Glass,
    /// Resin, as glue.
    Resin,
}

impl Class {
    /// Every class, in a stable order.
    pub const ALL: [Class; 8] = [
        Class::Bone,
        Class::Antler,
        Class::Keratin,
        Class::Cuticle,
        Class::Hide,
        Class::Wood,
        Class::Glass,
        Class::Resin,
    ];

    /// Returns the class a tissue of `kind` harvested from a part of `part` gives, or nothing for
    /// tissues too soft to build with (gel, glands).
    pub fn of(kind: TissueKind, part: PartKind) -> Option<Class> {
        let horn = matches!(part, PartKind::Horn | PartKind::Plate);
        match kind {
            TissueKind::Bone if part == PartKind::Horn => Some(Class::Antler),
            TissueKind::Bone => Some(Class::Bone),
            TissueKind::Hide if horn => Some(Class::Keratin),
            TissueKind::Hide => Some(Class::Hide),
            TissueKind::Shell => Some(Class::Cuticle),
            TissueKind::Fibre => Some(Class::Wood),
            TissueKind::Crystal => Some(Class::Glass),
            TissueKind::Resin => Some(Class::Resin),
            TissueKind::Gel | TissueKind::Gland => None,
        }
    }

    /// Returns the plain name of the class, as explanations print it.
    pub fn name(self) -> &'static str {
        match self {
            Class::Bone => "bone",
            Class::Antler => "antler",
            Class::Keratin => "horn",
            Class::Cuticle => "shell",
            Class::Hide => "hide",
            Class::Wood => "wood",
            Class::Glass => "glass",
            Class::Resin => "resin",
        }
    }

    /// Returns the tissue kind this class is harvested from.
    pub fn kind(self) -> TissueKind {
        match self {
            Class::Bone | Class::Antler => TissueKind::Bone,
            Class::Keratin | Class::Hide => TissueKind::Hide,
            Class::Cuticle => TissueKind::Shell,
            Class::Wood => TissueKind::Fibre,
            Class::Glass => TissueKind::Crystal,
            Class::Resin => TissueKind::Resin,
        }
    }

    /// Returns the measured material the class is anchored on, fresh (wet) as it comes off a body.
    ///
    /// Bone is bovine femur (E 17.7 GPa, 99 MPa, K_IC 6.35), antler wet (7.5 GPa, 108 MPa, K_IC up
    /// to 10.3), keratin wet horn (0.85 GPa, 40 MPa, K_IC 2.56), cuticle insect cuticle (3.05 GPa,
    /// K_IC 4.12) and glass soda-lime glass (K_IC 0.7 to 0.8): https://en.wikipedia.org/wiki/Antler,
    /// https://www.ebi.ac.uk/europepmc/webservices/rest/search?query=EXT_ID:20086133%20AND%20SRC:MED&resultType=core&format=json,
    /// https://www.ebi.ac.uk/europepmc/webservices/rest/search?query=EXT_ID:22496286%20AND%20SRC:MED&resultType=core&format=json,
    /// https://en.wikipedia.org/wiki/Fracture_toughness. Hide (K_IC = √(E·J) with a tearing energy J
    /// near 20 kJ/m²), wood, resin, cuticle strength and every density and conductivity are our
    /// estimates. Hardness is the home hardness of the kind in [`crate::life::tissue::HOME`], except
    /// keratin, which is horn at about 2.5.
    pub fn anchor(self) -> Material {
        let [
            density,
            stiffness,
            strength,
            toughness,
            hardness,
            conductivity,
        ] = match self {
            Class::Bone => [1900.0, 17.7, 99.0, 6.35, 5.0, 0.4],
            Class::Antler => [1800.0, 7.5, 108.0, 10.3, 5.0, 0.4],
            Class::Keratin => [1300.0, 0.85, 40.0, 2.56, 2.5, 0.2],
            Class::Cuticle => [1200.0, 3.05, 70.0, 4.12, 4.0, 0.25],
            Class::Hide => [1100.0, 0.1, 20.0, 1.4, 1.5, 0.2],
            Class::Wood => [600.0, 10.0, 80.0, 5.0, 2.0, 0.15],
            Class::Glass => [2400.0, 70.0, 70.0, 0.75, 7.0, 1.2],
            Class::Resin => [1100.0, 2.0, 40.0, 0.5, 2.5, 0.2],
        };
        Material {
            density,
            stiffness,
            strength,
            toughness,
            hardness,
            conductivity,
        }
    }
}
