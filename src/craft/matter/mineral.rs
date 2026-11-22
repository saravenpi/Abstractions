//! Minerals: the stones a planet's geology leaves on the ground. The world names them, since a rock
//! shows the stone it is made of; this file gives each its measured numbers.

use super::Material;
pub use crate::world::rocks::Mineral;

impl Mineral {
    /// Returns the measured numbers of the stone.
    ///
    /// Obsidian: hardness 5 to 5.5 (https://en.wikipedia.org/wiki/Obsidian), K_IC of glass 0.7 to
    /// 0.8 (https://en.wikipedia.org/wiki/Fracture_toughness). The rest are textbook values of the
    /// rock and our estimates: basalt E 60 GPa, flexural 25 MPa, K_IC 2; flint E 80 GPa, 100 MPa,
    /// K_IC 1.5, Mohs 7; ice E 9 GPa, 1.5 MPa, K_IC 0.1; quartz Mohs 7; nephrite jade K_IC 4.2;
    /// ironstone hematite E 160 GPa; malachite Mohs 4.5; brimstone Mohs 2.5; siderite nickel-iron.
    pub fn material(self) -> Material {
        let [
            density,
            stiffness,
            strength,
            toughness,
            hardness,
            conductivity,
        ] = match self {
            Mineral::Basalt => [3000.0, 60.0, 25.0, 2.0, 6.0, 1.7],
            Mineral::Obsidian => [2400.0, 70.0, 60.0, 0.75, 5.5, 1.2],
            Mineral::Flint => [2600.0, 80.0, 100.0, 1.5, 7.0, 3.0],
            Mineral::Ice => [917.0, 9.0, 1.5, 0.1, 1.5, 2.2],
            Mineral::Quartz => [2650.0, 95.0, 130.0, 1.8, 7.0, 3.2],
            Mineral::Jade => [3300.0, 110.0, 185.0, 4.2, 6.5, 2.8],
            Mineral::Ironstone => [5200.0, 160.0, 210.0, 3.2, 6.5, 12.0],
            Mineral::Malachite => [3950.0, 85.0, 115.0, 2.4, 4.5, 8.0],
            Mineral::Brimstone => [2070.0, 28.0, 45.0, 0.8, 2.5, 0.4],
            Mineral::Siderite => [7800.0, 200.0, 380.0, 6.5, 8.0, 45.0],
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
